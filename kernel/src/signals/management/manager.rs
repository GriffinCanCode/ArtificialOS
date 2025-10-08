/*!
 * Signal Manager
 * Central manager for process signals with queuing and delivery
 */

use crate::core::types::Pid;
use crate::signals::core::internal_types::{PrioritySignal, ProcessSignals, MAX_HANDLERS_PER_PROCESS};
use crate::signals::core::traits::*;
use crate::signals::core::types::*;
use crate::signals::handler::{CallbackRegistry, SignalHandler, SignalOutcome};
use crate::core::{ShardManager, WorkloadProfile};
use ahash::RandomState;
use dashmap::DashMap;
use log::{debug, info, warn};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Signal manager implementation
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic handler ID counter
#[repr(C, align(64))]
#[derive(Clone)]
pub struct SignalManagerImpl {
    processes: Arc<DashMap<Pid, ProcessSignals, RandomState>>,
    handler: Arc<SignalHandler>,
    callbacks: Arc<CallbackRegistry>,
    next_handler_id: Arc<AtomicU64>,
    stats: Arc<RwLock<SignalStats>>,
}

impl SignalManagerImpl {
    pub fn new() -> Self {
        let callbacks = Arc::new(CallbackRegistry::new());
        info!("Signal manager initialized");
        Self {
            // CPU-topology-aware shard counts for optimal concurrent performance
            processes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::HighContention), // signal delivery: high contention
            )),
            handler: Arc::new(SignalHandler::new(callbacks.clone())),
            callbacks,
            next_handler_id: Arc::new(AtomicU64::new(1)),
            stats: Arc::new(RwLock::new(SignalStats {
                total_signals_sent: 0,
                total_signals_delivered: 0,
                total_signals_blocked: 0,
                total_signals_queued: 0,
                pending_signals: 0,
                handlers_registered: 0,
            })),
        }
    }

    /// Get callback registry for registering handlers
    pub fn callbacks(&self) -> Arc<CallbackRegistry> {
        self.callbacks.clone()
    }

    /// Cleanup all signal resources for a terminated process
    pub fn cleanup_process_signals(&self, pid: Pid) -> usize {
        let mut count = 0;

        // Remove process signal state (handlers, pending signals, blocked signals)
        if let Some((_, proc)) = self.processes.remove(&pid) {
            count += proc.handlers.len();
            count += proc.pending.len();

            // Update stats
            let mut stats = self.stats.write();
            stats.handlers_registered = stats
                .handlers_registered
                .saturating_sub(proc.handlers.len());
            stats.pending_signals = stats.pending_signals.saturating_sub(proc.pending.len());
        }

        if count > 0 {
            info!(
                "Cleaned {} signal resources for terminated PID {}",
                count, pid
            );
        }

        count
    }

    /// Check if process has any signal resources
    pub fn has_process_signals(&self, pid: Pid) -> bool {
        self.processes.get(&pid).is_some()
    }

    /// Get current timestamp
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Allocate handler ID
    #[allow(dead_code)]
    fn next_handler_id(&self) -> u64 {
        self.next_handler_id.fetch_add(1, AtomicOrdering::SeqCst)
    }

    /// Queue signal for delivery with priority
    fn queue_signal(&self, sender_pid: Pid, target_pid: Pid, signal: Signal) -> SignalResult<()> {
        let mut proc = self
            .processes
            .get_mut(&target_pid)
            .ok_or(SignalError::ProcessNotFound(target_pid))?;

        // Check queue capacity
        if !proc.can_queue() {
            warn!("Signal queue full for PID {}", target_pid);
            return Err(SignalError::QueueFull(target_pid));
        }

        // Check if signal is blocked
        if proc.blocked.contains(&signal) && signal.can_catch() {
            debug!("Signal {:?} blocked for PID {}", signal, target_pid);
            self.stats.write().total_signals_blocked += 1;
            return Err(SignalError::SignalBlocked(signal));
        }

        // Queue with priority (RT signals get higher priority)
        proc.pending.push(PrioritySignal {
            signal: PendingSignal {
                signal,
                sender_pid,
                timestamp: Self::timestamp(),
            },
            priority: signal.priority(),
        });

        let mut stats = self.stats.write();
        stats.total_signals_queued += 1;
        stats.pending_signals += 1;

        info!(
            "Queued signal {:?} from PID {} to PID {} (priority: {})",
            signal,
            sender_pid,
            target_pid,
            signal.priority()
        );
        Ok(())
    }

    /// Process and deliver a signal immediately
    fn process_signal(
        &self,
        pid: Pid,
        signal: Signal,
        action: SignalAction,
    ) -> SignalResult<SignalOutcome> {
        self.handler.execute(pid, signal, action)
    }
}

impl Default for SignalManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

// Implement SignalDelivery trait
impl SignalDelivery for SignalManagerImpl {
    fn send(&self, sender_pid: Pid, target_pid: Pid, signal: Signal) -> SignalResult<()> {
        debug!(
            "Sending signal {:?} from PID {} to PID {}",
            signal, sender_pid, target_pid
        );

        // Validate signal
        self.handler.validate_signal(signal)?;

        // SIGKILL and SIGSTOP cannot be caught, blocked, or ignored
        if matches!(signal, Signal::SIGKILL | Signal::SIGSTOP) {
            let action = self.handler.default_action(signal);
            let outcome = self.process_signal(target_pid, signal, action)?;

            let mut stats = self.stats.write();
            stats.total_signals_sent += 1;
            stats.total_signals_delivered += 1;

            info!(
                "Delivered uncatchable signal {:?} to PID {} with outcome {:?}",
                signal, target_pid, outcome
            );
            return Ok(());
        }

        // Queue signal for delivery
        self.queue_signal(sender_pid, target_pid, signal)?;

        self.stats.write().total_signals_sent += 1;

        Ok(())
    }

    fn broadcast(&self, sender_pid: Pid, signal: Signal) -> SignalResult<u32> {
        debug!("Broadcasting signal {:?} from PID {}", signal, sender_pid);

        let target_pids: Vec<Pid> = self.processes.iter().map(|entry| *entry.key()).collect();

        let mut delivered = 0;
        for target_pid in target_pids {
            if target_pid != sender_pid {
                if self.send(sender_pid, target_pid, signal).is_ok() {
                    delivered += 1;
                }
            }
        }

        info!("Broadcast signal {:?} to {} processes", signal, delivered);
        Ok(delivered)
    }

    fn deliver_pending(&self, pid: Pid) -> SignalResult<usize> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        let pending_count = proc.pending.len();
        if pending_count == 0 {
            return Ok(0);
        }

        // Collect pending signals by priority (heap drains in priority order)
        let mut signals_to_deliver: Vec<PendingSignal> = Vec::new();
        while let Some(ps) = proc.pending.pop() {
            signals_to_deliver.push(ps.signal);
        }
        let handlers = proc.handlers.clone();

        drop(proc);

        let mut delivered = 0;
        for pending in signals_to_deliver {
            let action = handlers
                .get(&pending.signal)
                .cloned()
                .unwrap_or_else(|| self.handler.default_action(pending.signal));

            match self.process_signal(pid, pending.signal, action) {
                Ok(outcome) => {
                    debug!(
                        "Delivered signal {:?} to PID {} with outcome {:?}",
                        pending.signal, pid, outcome
                    );
                    delivered += 1;
                    self.stats.write().total_signals_delivered += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to deliver signal {:?} to PID {}: {}",
                        pending.signal, pid, e
                    );
                }
            }
        }

        let mut stats = self.stats.write();
        stats.pending_signals = stats.pending_signals.saturating_sub(delivered);
        drop(stats);

        info!(
            "Delivered {} pending signals to PID {} (priority order)",
            delivered, pid
        );
        Ok(delivered)
    }
}

// Implement SignalHandlerRegistry trait
impl SignalHandlerRegistry for SignalManagerImpl {
    fn register_handler(&self, pid: Pid, signal: Signal, action: SignalAction) -> SignalResult<()> {
        // Cannot catch or ignore SIGKILL or SIGSTOP
        if !signal.can_catch() {
            return Err(SignalError::PermissionDenied(format!(
                "Signal {:?} cannot be caught",
                signal
            )));
        }

        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        if proc.handlers.len() >= MAX_HANDLERS_PER_PROCESS {
            return Err(SignalError::OperationFailed(
                "Too many handlers registered".to_string(),
            ));
        }

        proc.handlers.insert(signal, action);
        self.stats.write().handlers_registered += 1;

        info!("Registered handler for signal {:?} on PID {}", signal, pid);
        Ok(())
    }

    fn unregister_handler(&self, pid: Pid, signal: Signal) -> SignalResult<()> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        if let Some(action) = proc.handlers.remove(&signal) {
            // Cleanup callback from registry if it was a handler callback
            if let SignalAction::Handler(handler_id) = action {
                self.callbacks.unregister(handler_id);
            }

            let mut stats = self.stats.write();
            stats.handlers_registered = stats.handlers_registered.saturating_sub(1);
            drop(stats);
            info!(
                "Unregistered handler for signal {:?} on PID {}",
                signal, pid
            );
        }

        Ok(())
    }

    fn get_handler(&self, pid: Pid, signal: Signal) -> Option<SignalAction> {
        self.processes
            .get(&pid)
            .and_then(|proc| proc.handlers.get(&signal).cloned())
    }

    fn reset_handlers(&self, pid: Pid) -> SignalResult<()> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        let count = proc.handlers.len();

        // Cleanup callback handlers from registry to prevent leak
        let mut callback_ids_cleaned = 0;
        for action in proc.handlers.values() {
            if let SignalAction::Handler(handler_id) = action {
                if self.callbacks.unregister(*handler_id) {
                    callback_ids_cleaned += 1;
                }
            }
        }

        proc.handlers.clear();

        let mut stats = self.stats.write();
        stats.handlers_registered = stats.handlers_registered.saturating_sub(count);
        drop(stats);

        info!(
            "Reset all handlers for PID {} ({} callbacks cleaned)",
            pid, callback_ids_cleaned
        );
        Ok(())
    }
}

// Implement SignalMasking trait
impl SignalMasking for SignalManagerImpl {
    fn block_signal(&self, pid: Pid, signal: Signal) -> SignalResult<()> {
        if !signal.can_catch() {
            return Err(SignalError::PermissionDenied(format!(
                "Signal {:?} cannot be blocked",
                signal
            )));
        }

        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        proc.blocked.insert(signal);
        debug!("Blocked signal {:?} for PID {}", signal, pid);
        Ok(())
    }

    fn unblock_signal(&self, pid: Pid, signal: Signal) -> SignalResult<()> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        proc.blocked.remove(&signal);
        debug!("Unblocked signal {:?} for PID {}", signal, pid);
        Ok(())
    }

    fn is_blocked(&self, pid: Pid, signal: Signal) -> bool {
        self.processes
            .get(&pid)
            .map(|proc| proc.blocked.contains(&signal))
            .unwrap_or(false)
    }

    fn get_blocked(&self, pid: Pid) -> Vec<Signal> {
        self.processes
            .get(&pid)
            .map(|proc| proc.blocked.iter().copied().collect())
            .unwrap_or_default()
    }

    fn set_mask(&self, pid: Pid, signals: Vec<Signal>) -> SignalResult<()> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        // Validate signals can be blocked
        for signal in &signals {
            if !signal.can_catch() {
                return Err(SignalError::PermissionDenied(format!(
                    "Signal {:?} cannot be blocked",
                    signal
                )));
            }
        }

        proc.blocked = signals.into_iter().collect();
        info!("Set signal mask for PID {}", pid);
        Ok(())
    }
}

// Implement SignalQueue trait
impl SignalQueue for SignalManagerImpl {
    fn pending_signals(&self, pid: Pid) -> Vec<Signal> {
        self.processes
            .get(&pid)
            .map(|proc| proc.pending.iter().map(|ps| ps.signal.signal).collect())
            .unwrap_or_default()
    }

    fn has_pending(&self, pid: Pid) -> bool {
        self.processes
            .get(&pid)
            .map(|proc| proc.has_pending())
            .unwrap_or(false)
    }

    fn clear_pending(&self, pid: Pid) -> SignalResult<usize> {
        let mut proc = self
            .processes
            .get_mut(&pid)
            .ok_or(SignalError::ProcessNotFound(pid))?;

        let count = proc.pending.len();
        proc.pending.clear();

        let mut stats = self.stats.write();
        stats.pending_signals = stats.pending_signals.saturating_sub(count);
        drop(stats);

        info!("Cleared {} pending signals for PID {}", count, pid);
        Ok(count)
    }

    fn pending_count(&self, pid: Pid) -> usize {
        self.processes
            .get(&pid)
            .map(|proc| proc.pending.len())
            .unwrap_or(0)
    }
}

// Implement SignalStateManager trait
impl SignalStateManager for SignalManagerImpl {
    fn get_state(&self, pid: Pid) -> Option<ProcessSignalState> {
        self.processes.get(&pid).map(|proc| ProcessSignalState {
            pid: proc.pid,
            pending_signals: proc.pending.iter().map(|ps| ps.signal.clone()).collect(),
            blocked_signals: proc.blocked.iter().copied().collect(),
            handlers: proc.handlers.iter().map(|(s, a)| (*s, a.clone())).collect(),
        })
    }

    fn initialize_process(&self, pid: Pid) -> SignalResult<()> {
        if self.processes.contains_key(&pid) {
            return Err(SignalError::OperationFailed(format!(
                "Process {} already initialized",
                pid
            )));
        }

        self.processes.insert(pid, ProcessSignals::new(pid));
        info!("Initialized signal state for PID {}", pid);
        Ok(())
    }

    fn cleanup_process(&self, pid: Pid) -> SignalResult<()> {
        if let Some((_, proc)) = self.processes.remove(&pid) {
            let pending_count = proc.pending.len();
            let handler_count = proc.handlers.len();

            // Extract and cleanup handler IDs from CallbackRegistry to prevent leak
            // Zero-overhead: only iterate handlers during cleanup (infrequent operation)
            let mut callback_ids_cleaned = 0;
            for action in proc.handlers.values() {
                if let SignalAction::Handler(handler_id) = action {
                    if self.callbacks.unregister(*handler_id) {
                        callback_ids_cleaned += 1;
                    }
                }
            }

            let mut stats = self.stats.write();
            stats.pending_signals = stats.pending_signals.saturating_sub(pending_count);
            stats.handlers_registered = stats.handlers_registered.saturating_sub(handler_count);

            info!(
                "Cleaned up signal state for PID {} ({} pending, {} handlers, {} callbacks)",
                pid, pending_count, handler_count, callback_ids_cleaned
            );
        }

        Ok(())
    }

    fn stats(&self) -> SignalStats {
        self.stats.read().clone()
    }
}
