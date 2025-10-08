/*!
 * Data Structures
 *
 * Specialized data structures for kernel operations:
 * - Const generics utilities for compile-time sizing
 * - Inline strings for stack-allocated small strings
 * - Epoch-based file descriptor table for lock-free FD management
 *
 * # Performance
 *
 * - Const generics: Zero runtime overhead, compile-time validation
 * - Inline strings: Avoids heap allocation for strings ≤15 bytes
 * - Epoch FD table: Lock-free reads, generational safety
 *
 * # Use Cases
 *
 * - **Const generics**: Fixed-size arrays, compile-time bounds
 * - **Inline strings**: Short-lived strings, path components
 * - **Epoch FD table**: High-throughput FD operations
 */

mod const_generics;
mod epoch_fd;
mod inline_string;

pub use epoch_fd::EpochFdTable;
pub use inline_string::InlineString;

// Re-export const generics utilities
pub use const_generics::*;
