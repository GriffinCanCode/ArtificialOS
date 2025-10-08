/*!
 * Process Memory Tracking
 * Per-process memory usage statistics
 */

use crate::core::types::Size;

/// Per-process memory tracking
///
/// # Performance
/// - Cache-line aligned to prevent false sharing in concurrent memory operations
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct ProcessMemoryTracking {
    pub current_bytes: Size,
    pub peak_bytes: Size,
    pub allocation_count: usize,
}

impl ProcessMemoryTracking {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current_bytes: 0,
            peak_bytes: 0,
            allocation_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn add_allocation(&mut self, size: Size) {
        self.current_bytes += size;
        self.allocation_count += 1;
        if self.current_bytes > self.peak_bytes {
            self.peak_bytes = self.current_bytes;
        }
    }

    #[allow(dead_code)]
    pub fn remove_allocation(&mut self, size: Size) {
        self.current_bytes = self.current_bytes.saturating_sub(size);
    }
}
