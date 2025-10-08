/*!
 * Core Syscall Execution Infrastructure
 *
 * Provides the foundational components for syscall execution:
 * - Executor: Main syscall dispatcher with type-state pattern
 * - Handler: Trait and registry for syscall handlers
 * - Handlers: Category-specific handler implementations
 */

pub mod executor;
pub mod handler;
pub mod handlers;

// Re-export commonly used types
pub use executor::{IpcManagers, OptionalManagers, SyscallExecutorWithIpc, SYSTEM_START};
pub use handler::{SyscallHandler, SyscallHandlerRegistry};

