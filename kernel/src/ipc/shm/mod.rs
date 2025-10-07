/*!
 * Shared Memory Module
 * Zero-copy data sharing between processes
 */

pub mod manager;
pub mod segment;
pub mod traits;
pub mod types;

// Re-export public API
pub use manager::ShmManager;
pub use types::{ShmError, ShmPermission, ShmStats};
