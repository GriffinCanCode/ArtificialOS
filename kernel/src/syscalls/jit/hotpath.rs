/*!
 * Hot Path Detection
 * Identifies frequently executed syscalls for JIT compilation
 */

use super::types::SyscallPattern;
use crate::core::types::Pid;
use crate::syscalls::types::Syscall;
use ahash::RandomState;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Threshold for considering a syscall pattern as "hot"
const HOT_THRESHOLD: u64 = 100;

/// Detection window for hot path analysis
#[allow(dead_code)]
const DETECTION_WINDOW: u64 = 1000;

/// Hot path detector
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic counters
#[repr(C, align(64))]
pub struct HotpathDetector {
    /// Per-process syscall counts
    process_counts: Arc<DashMap<(Pid, SyscallPattern), AtomicU64, RandomState>>,
    /// Global syscall counts
    global_counts: Arc<DashMap<SyscallPattern, AtomicU64, RandomState>>,
    /// Total syscalls processed
    total_syscalls: AtomicU64,
}

impl HotpathDetector {
    /// Create a new hot path detector
    pub fn new() -> Self {
        Self {
            process_counts: Arc::new(DashMap::with_hasher(RandomState::new())),
            global_counts: Arc::new(DashMap::with_hasher(RandomState::new())),
            total_syscalls: AtomicU64::new(0),
        }
    }

    /// Record a syscall execution
    pub fn record(&self, pid: Pid, syscall: &Syscall) {
        let pattern = SyscallPattern::from_syscall(syscall);

        // Increment process-specific count
        self.process_counts
            .entry((pid, pattern.clone()))
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Increment global count
        self.global_counts
            .entry(pattern)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Increment total
        self.total_syscalls.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if a syscall is hot for a specific process
    pub fn is_hot(&self, pid: Pid, syscall: &Syscall) -> bool {
        let pattern = SyscallPattern::from_syscall(syscall);

        if let Some(count) = self.process_counts.get(&(pid, pattern)) {
            count.load(Ordering::Relaxed) >= HOT_THRESHOLD
        } else {
            false
        }
    }

    /// Check if a pattern is globally hot
    pub fn is_globally_hot(&self, pattern: &SyscallPattern) -> bool {
        if let Some(count) = self.global_counts.get(pattern) {
            count.load(Ordering::Relaxed) >= HOT_THRESHOLD
        } else {
            false
        }
    }

    /// Get all hot patterns that should be compiled
    pub fn get_hot_patterns(&self) -> Vec<SyscallPattern> {
        let mut hot_patterns = Vec::new();

        for entry in self.global_counts.iter() {
            let pattern = entry.key();
            let count = entry.value().load(Ordering::Relaxed);

            if count >= HOT_THRESHOLD {
                hot_patterns.push(pattern.clone());
            }
        }

        // Sort by frequency (most frequent first)
        hot_patterns.sort_by_key(|pattern| {
            let count = self
                .global_counts
                .get(pattern)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0);
            std::cmp::Reverse(count)
        });

        hot_patterns
    }

    /// Get count for a specific pattern
    pub fn get_count(&self, pid: Pid, pattern: &SyscallPattern) -> u64 {
        self.process_counts
            .get(&(pid, pattern.clone()))
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get global count for a pattern
    pub fn get_global_count(&self, pattern: &SyscallPattern) -> u64 {
        self.global_counts
            .get(pattern)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.process_counts.clear();
        self.global_counts.clear();
        self.total_syscalls.store(0, Ordering::Relaxed);
    }

    /// Get total syscall count
    pub fn total_syscalls(&self) -> u64 {
        self.total_syscalls.load(Ordering::Relaxed)
    }
}

impl Default for HotpathDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotpath_detection() {
        let detector = HotpathDetector::new();
        let syscall = Syscall::GetProcessList;

        // Initially not hot
        assert!(!detector.is_hot(1, &syscall));

        // Record many times
        for _ in 0..HOT_THRESHOLD {
            detector.record(1, &syscall);
        }

        // Now should be hot
        assert!(detector.is_hot(1, &syscall));
    }

    #[test]
    fn test_get_hot_patterns() {
        let detector = HotpathDetector::new();

        // Record various syscalls
        for _ in 0..HOT_THRESHOLD + 10 {
            detector.record(1, &Syscall::GetProcessList);
        }
        for _ in 0..HOT_THRESHOLD + 5 {
            detector.record(1, &Syscall::GetProcessInfo { target_pid: 1 });
        }

        let hot_patterns = detector.get_hot_patterns();
        assert_eq!(hot_patterns.len(), 2);

        // Should be sorted by frequency
        assert_eq!(
            hot_patterns[0],
            SyscallPattern::Simple(super::super::types::SimpleSyscallType::GetProcessList)
        );
    }
}
