/*!
 * Global Garbage Collector
 * System-wide garbage collection and memory cleanup
 */

use super::super::manager::MemoryManager;
use crate::core::serialization::serde::{is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, Size};
use ahash::HashMap;
use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Global garbage collection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GcStrategy {
    /// Collect from all processes
    Global,
    /// Collect only from processes over threshold
    Threshold { threshold: Size },
    /// Collect from specific processes
    Targeted { pid: Pid },
    /// Collect unreferenced memory blocks
    Unreferenced,
}

/// Garbage collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GcStats {
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub freed_bytes: Size,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub freed_blocks: usize,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub processes_cleaned: usize,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub duration_ms: u64,
}

impl Default for GcStats {
    fn default() -> Self {
        Self {
            freed_bytes: 0,
            freed_blocks: 0,
            processes_cleaned: 0,
            duration_ms: 0,
        }
    }
}

impl GcStats {
    /// Create new empty GC stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate bytes freed per process
    pub fn bytes_per_process(&self) -> Size {
        if self.processes_cleaned == 0 {
            0
        } else {
            self.freed_bytes / self.processes_cleaned
        }
    }

    /// Calculate blocks freed per process
    pub fn blocks_per_process(&self) -> f64 {
        if self.processes_cleaned == 0 {
            0.0
        } else {
            self.freed_blocks as f64 / self.processes_cleaned as f64
        }
    }

    /// Check if any memory was freed
    pub fn freed_any(&self) -> bool {
        self.freed_bytes > 0 || self.freed_blocks > 0
    }
}

/// Global garbage collector
pub struct GlobalGarbageCollector {
    memory_manager: Arc<RwLock<MemoryManager>>,
    /// Threshold for automatic GC trigger (bytes)
    threshold: Arc<RwLock<Size>>,
    /// Track last GC time
    last_gc: Arc<RwLock<std::time::Instant>>,
    /// Minimum time between GCs (seconds)
    min_interval: u64,
}

impl GlobalGarbageCollector {
    /// Create a new global garbage collector
    pub fn new(memory_manager: MemoryManager) -> Self {
        let threshold = crate::core::limits::DEFAULT_GC_THRESHOLD;
        info!(
            "Global garbage collector initialized with threshold: {} bytes",
            threshold
        );

        Self {
            memory_manager: Arc::new(RwLock::new(memory_manager)),
            threshold: Arc::new(RwLock::new(threshold)),
            last_gc: Arc::new(RwLock::new(std::time::Instant::now())),
            min_interval: 5, // 5 seconds minimum between GCs
        }
    }

    /// Run garbage collection with specified strategy
    pub fn collect(&self, strategy: GcStrategy) -> GcStats {
        let start = std::time::Instant::now();
        info!(
            "Starting global garbage collection with strategy: {:?}",
            strategy
        );

        let mut stats = GcStats::new();

        match strategy {
            GcStrategy::Global => {
                self.collect_global(&mut stats);
            }
            GcStrategy::Threshold { threshold } => {
                self.collect_threshold(threshold, &mut stats);
            }
            GcStrategy::Targeted { pid } => {
                self.collect_targeted(pid, &mut stats);
            }
            GcStrategy::Unreferenced => {
                self.collect_unreferenced(&mut stats);
            }
        }

        // Update last GC time
        *self.last_gc.write() = std::time::Instant::now();

        stats.duration_ms = start.elapsed().as_millis() as u64;
        info!(
            "Global GC completed: freed {} bytes ({} blocks) from {} processes in {}ms",
            stats.freed_bytes, stats.freed_blocks, stats.processes_cleaned, stats.duration_ms
        );

        stats
    }

    /// Collect from all processes
    fn collect_global(&self, stats: &mut GcStats) {
        let mm = self.memory_manager.read();

        // First, run the memory manager's internal GC to clean deallocated blocks
        let freed_blocks = mm.force_collect();
        stats.freed_blocks = freed_blocks;

        info!("Global GC: cleaned {} deallocated blocks", freed_blocks);
    }

    /// Collect from processes exceeding threshold
    fn collect_threshold(&self, threshold: Size, stats: &mut GcStats) {
        let mm = self.memory_manager.read();

        // Get all process allocations
        let process_memory = self.get_process_memory_map(&mm);

        // Identify processes over threshold
        let targets: Vec<Pid> = process_memory
            .iter()
            .filter(|(_, &size)| size > threshold)
            .map(|(&pid, _)| pid)
            .collect();

        info!(
            "Threshold GC: found {} processes over {} bytes threshold",
            targets.len(),
            threshold
        );

        for pid in targets {
            let freed = mm.free_process_memory(pid);
            if freed > 0 {
                stats.freed_bytes += freed;
                stats.processes_cleaned += 1;
                warn!(
                    "Threshold GC: freed {} bytes from PID {} (exceeded threshold)",
                    freed, pid
                );
            }
        }

        // Also clean up unreferenced blocks
        let freed_blocks = mm.force_collect();
        stats.freed_blocks = freed_blocks;
    }

    /// Collect from a specific process
    fn collect_targeted(&self, pid: Pid, stats: &mut GcStats) {
        let mm = self.memory_manager.read();

        let freed = mm.free_process_memory(pid);
        if freed > 0 {
            stats.freed_bytes = freed;
            stats.processes_cleaned = 1;
            info!("Targeted GC: freed {} bytes from PID {}", freed, pid);
        } else {
            info!("Targeted GC: no memory to free from PID {}", pid);
        }

        // Clean up unreferenced blocks
        let freed_blocks = mm.force_collect();
        stats.freed_blocks = freed_blocks;
    }

    /// Collect only unreferenced/deallocated blocks
    fn collect_unreferenced(&self, stats: &mut GcStats) {
        let mm = self.memory_manager.read();

        let freed_blocks = mm.force_collect();
        stats.freed_blocks = freed_blocks;

        info!(
            "Unreferenced GC: cleaned {} deallocated blocks",
            freed_blocks
        );
    }

    /// Check if GC should run based on time interval
    pub fn should_collect(&self) -> bool {
        let last_gc = self.last_gc.read();
        let elapsed = last_gc.elapsed().as_secs();

        elapsed >= self.min_interval
    }

    /// Check if GC should run based on memory pressure
    pub fn should_collect_pressure(&self) -> bool {
        let mm = self.memory_manager.read();
        let (total, used, _available) = mm.info();

        let usage_ratio = used as f64 / total as f64;

        // Trigger if over 80% memory usage
        usage_ratio >= 0.80
    }

    /// Auto-collect based on conditions
    pub fn auto_collect(&self) -> Option<GcStats> {
        if !self.should_collect() {
            return None;
        }

        if self.should_collect_pressure() {
            info!("Auto-GC triggered due to memory pressure");
            Some(self.collect(GcStrategy::Global))
        } else {
            // Just clean up unreferenced blocks
            Some(self.collect(GcStrategy::Unreferenced))
        }
    }

    /// Set the GC threshold
    pub fn set_threshold(&self, threshold: Size) {
        *self.threshold.write() = threshold;
        info!("Global GC threshold set to {} bytes", threshold);
    }

    /// Get current threshold
    pub fn get_threshold(&self) -> Size {
        *self.threshold.read()
    }

    /// Get memory statistics
    pub fn stats(&self) -> (Size, Size, Size) {
        let mm = self.memory_manager.read();
        mm.info()
    }

    /// Get process memory map
    fn get_process_memory_map(&self, mm: &MemoryManager) -> HashMap<Pid, Size> {
        let mut map = HashMap::default();

        // Get all allocated blocks and group by process
        let allocations = mm.process_allocations(0); // This would need to be refactored

        for block in allocations {
            if let Some(pid) = block.owner_pid {
                *map.entry(pid).or_insert(0) += block.size;
            }
        }

        map
    }

    /// Force immediate garbage collection
    pub fn force_collect(&self, strategy: GcStrategy) -> GcStats {
        info!("Force collecting with strategy: {:?}", strategy);
        self.collect(strategy)
    }

    /// Get time since last GC
    pub fn time_since_last_gc(&self) -> u64 {
        self.last_gc.read().elapsed().as_secs()
    }

    /// Check if memory manager should collect
    pub fn check_memory_manager(&self) -> bool {
        let mm = self.memory_manager.read();
        mm.should_collect()
    }
}

impl Clone for GlobalGarbageCollector {
    fn clone(&self) -> Self {
        Self {
            memory_manager: Arc::clone(&self.memory_manager),
            threshold: Arc::clone(&self.threshold),
            last_gc: Arc::clone(&self.last_gc),
            min_interval: self.min_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_creation() {
        let mm = MemoryManager::new();
        let gc = GlobalGarbageCollector::new(mm);
        assert_eq!(gc.get_threshold(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_gc_threshold() {
        let mm = MemoryManager::new();
        let gc = GlobalGarbageCollector::new(mm);

        gc.set_threshold(50 * 1024 * 1024);
        assert_eq!(gc.get_threshold(), 50 * 1024 * 1024);
    }

    #[test]
    fn test_unreferenced_collection() {
        let mm = MemoryManager::new();
        let gc = GlobalGarbageCollector::new(mm);

        let stats = gc.collect(GcStrategy::Unreferenced);
        assert_eq!(stats.freed_bytes, 0);
    }
}
