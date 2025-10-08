/*!
 * Timeout Management for Blocking Syscalls
 *
 * Provides timeout enforcement for syscalls that can block:
 * - Config: Per-operation timeout policies
 * - Executor: Retry logic with adaptive backoff
 */

pub mod config;
pub mod executor;

// Re-export commonly used types
pub use config::SyscallTimeoutConfig;
pub use executor::{TimeoutError, TimeoutExecutor};

// Re-export TimeoutPolicy from core
pub use crate::core::guard::TimeoutPolicy;
