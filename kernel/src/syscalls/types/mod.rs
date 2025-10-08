/*!
 * Syscall Types Module
 * Defines syscall enum, errors, results, and helper types with modern serde patterns
 */

mod errors;
mod process_types;
mod results;
mod syscall;

// Re-export all public types
pub use errors::SyscallError;
pub use process_types::{ProcessOutput, SystemInfo};
pub use results::SyscallResult;
pub use syscall::search::SearchResult;
pub use syscall::Syscall;
