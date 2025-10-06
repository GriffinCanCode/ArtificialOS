/*!
 * Security Module
 * Sandboxing and resource limits
 */

pub mod limits;
pub mod sandbox;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use limits::LimitManager;
pub use sandbox::SandboxManager;
pub use traits::*;
pub use types::*;

// Re-export ResourceLimits from core
pub use crate::core::types::ResourceLimits;
