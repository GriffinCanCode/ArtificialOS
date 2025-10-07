/*!
 * Internal Signal Manager Types
 * Types used internally by the signal manager
 */

use super::types::{PendingSignal, Signal, SignalAction};
use crate::core::types::Pid;
use ahash::HashMap;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

// Configuration constants
pub(super) const MAX_PENDING_SIGNALS: usize = 128;
pub(super) const MAX_HANDLERS_PER_PROCESS: usize = 32;

/// Pending signal with priority (for heap ordering)
#[derive(Debug, Clone)]
pub(super) struct PrioritySignal {
    pub signal: PendingSignal,
    pub priority: u32,
}

impl PartialEq for PrioritySignal {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritySignal {}

impl PartialOrd for PrioritySignal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritySignal {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older timestamp
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.signal.timestamp.cmp(&self.signal.timestamp))
    }
}

/// Process signal information
#[derive(Debug, Clone)]
pub(super) struct ProcessSignals {
    pub pid: Pid,
    pub pending: BinaryHeap<PrioritySignal>,
    pub blocked: HashSet<Signal>,
    pub handlers: HashMap<Signal, SignalAction>,
}

impl ProcessSignals {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            pending: BinaryHeap::new(),
            blocked: HashSet::new(),
            handlers: HashMap::default(),
        }
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn can_queue(&self) -> bool {
        self.pending.len() < MAX_PENDING_SIGNALS
    }
}
