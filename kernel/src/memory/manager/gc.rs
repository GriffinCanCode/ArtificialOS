/*!
 * Memory Manager Garbage Collection
 * Internal GC for cleaning up deallocated memory blocks
 */

use super::MemoryManager;
use crate::core::types::{Address, Size};
use log::info;
use std::sync::atomic::Ordering;

impl MemoryManager {
    /// Garbage collect deallocated memory blocks
    /// Removes deallocated blocks from the HashMap to prevent unbounded growth
    /// Note: Free blocks remain in the segregated free list for address recycling
    pub fn collect(&self) -> Size {
        let initial_count = self.blocks.len();

        // Collect addresses of deallocated blocks before removing them
        let deallocated_addrs: Vec<Address> = self
            .blocks
            .iter()
            .filter(|entry| !entry.value().allocated)
            .map(|entry| *entry.key())
            .collect();

        // Remove deallocated blocks from the blocks map
        for addr in &deallocated_addrs {
            self.blocks.remove(addr);
        }

        let removed_count = deallocated_addrs.len();

        // Clean up storage for deallocated blocks
        if !deallocated_addrs.is_empty() {
            for addr in &deallocated_addrs {
                self.memory_storage.remove(addr);
            }
        }

        // Reset deallocated counter
        self.deallocated_count.store(0, Ordering::SeqCst);

        // Note: We intentionally keep blocks in the segregated free list for address recycling
        // The segregated free list allows these addresses to be reused in O(1) or O(log n) time

        if removed_count > 0 {
            // Shrink DashMap capacity after bulk deletion to reclaim memory
            self.blocks.shrink_to_fit();
            self.memory_storage.shrink_to_fit();

            let free_list_size = self.free_list.lock().unwrap().len();
            info!(
                "Garbage collection complete: removed {} deallocated blocks and their storage, {} blocks remain, {} blocks in segregated free list for O(1)/O(log n) recycling (maps shrunk to fit)",
                removed_count,
                initial_count - removed_count,
                free_list_size
            );
        }

        removed_count
    }

    /// Force garbage collection (for testing or manual cleanup)
    pub fn force_collect(&self) -> Size {
        info!("Forcing garbage collection...");
        self.collect()
    }

    /// Check if GC should run
    pub fn should_collect(&self) -> bool {
        let dealloc_count = self.deallocated_count.load(Ordering::SeqCst);
        dealloc_count >= self.gc_threshold as u64
    }

    /// Set GC threshold
    pub fn set_threshold(&mut self, _threshold: Size) {
        // Note: MemoryManager uses Arc internally, so this would need refactoring
        // to support mutable threshold changes. For now, threshold is set at construction.
    }
}
