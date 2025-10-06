/*!
 * Pipe Module
 * Unix-style pipes for streaming data between processes
 */

pub mod manager;
pub mod pipe;
pub mod types;

// Re-export public API
pub use manager::PipeManager;
pub use types::{PipeError, PipeStats};
