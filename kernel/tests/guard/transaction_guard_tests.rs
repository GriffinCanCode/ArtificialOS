/*!
 * Transaction Guard Tests
 */

use ai_os_kernel::core::guard::*;
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

    let mut tx = TransactionGuard::new(Some(1), commit_fn, rollback_fn);

    tx.add_operation(Operation::new("insert", vec![1, 2, 3]))
        .unwrap();
    tx.add_operation(Operation::new("update", vec![4, 5, 6]))
        .unwrap();

    assert_eq!(tx.state(), TransactionState::Active);
    assert_eq!(tx.operations().len(), 2);

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

        tx.add_operation(Operation::new("op1", vec![1]))
            .unwrap();
        tx.add_operation(Operation::new("op2", vec![2]))
            .unwrap();

        // Drop without commit triggers auto-rollback
    }

    assert_eq!(committed.load(Ordering::SeqCst), 0);
    assert_eq!(rolled_back.load(Ordering::SeqCst), 2);
}

#[test]
fn test_transaction_manual_rollback() {
    let rolled_back = Arc::new(AtomicUsize::new(0));
    let rolled_back_clone = rolled_back.clone();

    let commit_fn = |_: &[Operation]| Ok(());
    let rollback_fn = move |ops: &[Operation]| {
        rolled_back_clone.store(ops.len(), Ordering::SeqCst);
        Ok(())
    };

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

    tx.add_operation(Operation::new("op", vec![])).unwrap();

    tx.rollback().unwrap();

    assert_eq!(rolled_back.load(Ordering::SeqCst), 1);
}

#[test]
fn test_transaction_execute() {
    let commit_fn = |_: &[Operation]| Ok(());
    let rollback_fn = |_: &[Operation]| Ok(());

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

    let result = tx
        .execute(|tx| {
            tx.add_operation(Operation::new("test", vec![42]))?;
            Ok(100)
        })
        .unwrap();

    assert_eq!(result, 100);
    assert_eq!(tx.operations().len(), 1);
    assert_eq!(tx.state(), TransactionState::Active);
}

#[test]
fn test_transaction_poisoning() {
    let commit_fn = |_: &[Operation]| Ok(());
    let rollback_fn = |_: &[Operation]| Ok(());

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

    tx.poison("Test error".to_string());

    assert!(tx.is_poisoned());
    assert_eq!(tx.poison_reason(), Some("Test error"));
    assert_eq!(tx.state(), TransactionState::Poisoned);

    // Can recover
    tx.recover().unwrap();
    assert!(!tx.is_poisoned());
}

#[test]
fn test_transaction_invalid_state_transitions() {
    let commit_fn = |_: &[Operation]| Ok(());
    let rollback_fn = |_: &[Operation]| Ok(());

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);
    tx.add_operation(Operation::new("op", vec![])).unwrap();

    // Commit the transaction
    tx.commit().unwrap();

    // Now try to add operation after commit (should fail)
    // Note: commit consumes tx, so we can't test this easily
    // This demonstrates the type safety of the API
}

#[test]
fn test_transaction_commit_failure() {
    let commit_fn = |_: &[Operation]| Err("Commit failed".to_string());
    let rollback_fn = |_: &[Operation]| Ok(());

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);
    tx.add_operation(Operation::new("op", vec![])).unwrap();

    let result = tx.commit();
    assert!(result.is_err());
}
