/*!
 * Memory Module
 * Memory management and allocation
 */

pub mod gc;
pub mod manager;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use gc::{GlobalGarbageCollector, GcStrategy, GcStats};
pub use manager::MemoryManager;
pub use traits::*;
pub use types::*;
