/*!
 * Process Memory Tracking
 * Per-process memory usage statistics
 */

use crate::core::types::Size;

/// Per-process memory tracking
#[derive(Debug, Clone)]
pub(super) struct ProcessMemoryTracking {
    pub current_bytes: Size,
    pub peak_bytes: Size,
    pub allocation_count: usize,
}

impl ProcessMemoryTracking {
    pub fn new() -> Self {
        Self {
            current_bytes: 0,
            peak_bytes: 0,
            allocation_count: 0,
        }
    }

    pub fn add_allocation(&mut self, size: Size) {
        self.current_bytes += size;
        self.allocation_count += 1;
        if self.current_bytes > self.peak_bytes {
            self.peak_bytes = self.current_bytes;
        }
    }

    pub fn remove_allocation(&mut self, size: Size) {
        self.current_bytes = self.current_bytes.saturating_sub(size);
    }
}
