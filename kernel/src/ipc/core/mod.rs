/*!
 * IPC Core Module
 * Core IPC types, traits, and manager
 */

pub mod manager;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use manager::IPCManager;
pub use traits::*;
pub use types::*;
