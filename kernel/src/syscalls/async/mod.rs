/*!
 * Async Syscall Execution Layer
 *
 * Provides async execution capabilities with intelligent dispatch:
 * - Executor: Dual-mode execution (fast-path sync, slow-path async)
 * - Classification: Compile-time syscall classification
 * - Legacy Traits: Deprecated async trait definitions
 */

pub mod classification;
pub mod executor;
pub mod legacy_traits;

// Re-export commonly used types
pub use classification::SyscallClass;
pub use executor::{AsyncExecutorStats, AsyncSyscallExecutor};

