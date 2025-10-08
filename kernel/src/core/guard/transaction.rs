/*!
 * Transaction Guards
 *
 * RAII guards for atomic operations with automatic rollback
 */

use super::traits::{Guard, GuardDrop, Recoverable};
use super::{GuardError, GuardMetadata, GuardResult};
use crate::core::types::Pid;
use std::sync::Arc;

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction in progress
    Active,
    /// Transaction committed successfully
    Committed,
    /// Transaction rolled back
    RolledBack,
    /// Transaction poisoned (panic during execution)
    Poisoned,
}

/// Transaction guard with automatic rollback
///
/// # Example
///
/// ```ignore
/// let tx = TransactionGuard::new(
///     pid,
///     |ops| {
///         // Commit operations
///         Ok(())
///     },
///     |ops| {
///         // Rollback operations
///         Ok(())
///     },
/// );
///
/// tx.execute(|| {
///     // Do work...
/// })?;
///
/// tx.commit()?; // Explicit commit
/// // Or auto-rollback on drop
/// ```
pub struct TransactionGuard {
    state: TransactionState,
    operations: Vec<Operation>,
    commit_fn: Arc<dyn Fn(&[Operation]) -> Result<(), String> + Send + Sync>,
    rollback_fn: Arc<dyn Fn(&[Operation]) -> Result<(), String> + Send + Sync>,
    metadata: GuardMetadata,
    poison_reason: Option<String>,
}

/// A single operation in a transaction
#[derive(Debug, Clone)]
pub struct Operation {
    pub name: String,
    pub data: Vec<u8>,
}

impl Operation {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

impl TransactionGuard {
    /// Create a new transaction guard
    pub fn new<C, R>(pid: Option<Pid>, commit_fn: C, rollback_fn: R) -> Self
    where
        C: Fn(&[Operation]) -> Result<(), String> + Send + Sync + 'static,
        R: Fn(&[Operation]) -> Result<(), String> + Send + Sync + 'static,
    {
        let mut metadata = GuardMetadata::new("transaction");
        if let Some(pid) = pid {
            metadata = metadata.with_pid(pid);
        }

        Self {
            state: TransactionState::Active,
            operations: Vec::new(),
            commit_fn: Arc::new(commit_fn),
            rollback_fn: Arc::new(rollback_fn),
            metadata,
            poison_reason: None,
        }
    }

    /// Add an operation to the transaction
    pub fn add_operation(&mut self, operation: Operation) -> GuardResult<()> {
        if self.state != TransactionState::Active {
            return Err(GuardError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: "Active".to_string(),
            });
        }

        self.operations.push(operation);
        Ok(())
    }

    /// Execute a function within the transaction
    ///
    /// If the function panics, transaction is marked as poisoned
    pub fn execute<F, R>(&mut self, f: F) -> GuardResult<R>
    where
        F: FnOnce(&mut Self) -> GuardResult<R>,
    {
        if self.state != TransactionState::Active {
            return Err(GuardError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: "Active".to_string(),
            });
        }

        // Catch panics and poison the transaction
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));

        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(e),
            Err(panic) => {
                self.poison(format!("Panic during transaction: {:?}", panic));
                Err(GuardError::Poisoned(
                    self.poison_reason.clone().unwrap_or_default(),
                ))
            }
        }
    }

    /// Commit the transaction
    pub fn commit(mut self) -> GuardResult<()> {
        if self.state != TransactionState::Active {
            return Err(GuardError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: "Committed".to_string(),
            });
        }

        (self.commit_fn)(&self.operations).map_err(|e| GuardError::OperationFailed(e))?;

        self.state = TransactionState::Committed;
        std::mem::forget(self); // Prevent drop from rolling back
        Ok(())
    }

    /// Manually rollback the transaction
    pub fn rollback(mut self) -> GuardResult<()> {
        if self.state != TransactionState::Active {
            return Err(GuardError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: "RolledBack".to_string(),
            });
        }

        self.do_rollback()?;
        std::mem::forget(self); // Prevent drop from rolling back again
        Ok(())
    }

    /// Internal rollback implementation
    fn do_rollback(&mut self) -> GuardResult<()> {
        (self.rollback_fn)(&self.operations).map_err(|e| GuardError::OperationFailed(e))?;

        self.state = TransactionState::RolledBack;
        Ok(())
    }

    /// Get current transaction state
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// Get operations in this transaction
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl Guard for TransactionGuard {
    fn resource_type(&self) -> &'static str {
        "transaction"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }

    fn release(&mut self) -> GuardResult<()> {
        if self.state == TransactionState::Active {
            self.do_rollback()
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }
}

impl GuardDrop for TransactionGuard {
    fn on_drop(&mut self) {
        if self.state == TransactionState::Active {
            log::info!(
                "Transaction auto-rolling back {} operations",
                self.operations.len()
            );
            if let Err(e) = self.do_rollback() {
                log::error!("Transaction rollback failed: {}", e);
            }
        }
    }
}

impl Recoverable for TransactionGuard {
    fn is_poisoned(&self) -> bool {
        self.state == TransactionState::Poisoned
    }

    fn recover(&mut self) -> GuardResult<()> {
        if self.state != TransactionState::Poisoned {
            return Ok(());
        }

        // Attempt rollback to recover
        self.do_rollback()?;
        self.poison_reason = None;
        Ok(())
    }

    fn poison_reason(&self) -> Option<&str> {
        self.poison_reason.as_deref()
    }

    fn poison(&mut self, reason: String) {
        self.state = TransactionState::Poisoned;
        self.poison_reason = Some(reason);
    }
}

impl Drop for TransactionGuard {
    fn drop(&mut self) {
        self.on_drop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_transaction_commit() {
        let committed = Arc::new(AtomicUsize::new(0));
        let rolled_back = Arc::new(AtomicUsize::new(0));

        let committed_clone = committed.clone();
        let rolled_back_clone = rolled_back.clone();

        let commit_fn = move |ops: &[Operation]| {
            committed_clone.store(ops.len(), Ordering::SeqCst);
            Ok(())
        };

        let rollback_fn = move |ops: &[Operation]| {
            rolled_back_clone.store(ops.len(), Ordering::SeqCst);
            Ok(())
        };

        let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

        tx.add_operation(Operation::new("op1", vec![1, 2, 3]))
            .unwrap();
        tx.add_operation(Operation::new("op2", vec![4, 5, 6]))
            .unwrap();

        tx.commit().unwrap();

        assert_eq!(committed.load(Ordering::SeqCst), 2);
        assert_eq!(rolled_back.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_transaction_auto_rollback() {
        let committed = Arc::new(AtomicUsize::new(0));
        let rolled_back = Arc::new(AtomicUsize::new(0));

        let committed_clone = committed.clone();
        let rolled_back_clone = rolled_back.clone();

        let commit_fn = move |ops: &[Operation]| {
            committed_clone.store(ops.len(), Ordering::SeqCst);
            Ok(())
        };

        let rollback_fn = move |ops: &[Operation]| {
            rolled_back_clone.store(ops.len(), Ordering::SeqCst);
            Ok(())
        };

        {
            let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

            tx.add_operation(Operation::new("op1", vec![1, 2, 3]))
                .unwrap();

            // Drop without commit triggers rollback
        }

        assert_eq!(committed.load(Ordering::SeqCst), 0);
        assert_eq!(rolled_back.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_transaction_execute() {
        let tx_result = Arc::new(AtomicUsize::new(0));
        let tx_result_clone = tx_result.clone();

        let commit_fn = |_: &[Operation]| Ok(());
        let rollback_fn = |_: &[Operation]| Ok(());

        let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

        let result = tx
            .execute(|tx| {
                tx.add_operation(Operation::new("test", vec![]))?;
                tx_result_clone.store(42, Ordering::SeqCst);
                Ok(100)
            })
            .unwrap();

        assert_eq!(result, 100);
        assert_eq!(tx_result.load(Ordering::SeqCst), 42);
        assert_eq!(tx.operations().len(), 1);
    }
}
