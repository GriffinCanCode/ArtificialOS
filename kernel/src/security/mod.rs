/*!
 * Security Module
 * Sandboxing and resource limits with granular capabilities
 */

pub mod ebpf;
pub mod limits;
pub mod namespace;
pub mod sandbox;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use ebpf::EbpfManagerImpl;
pub use limits::LimitManager;
pub use namespace::NamespaceManager;
pub use sandbox::SandboxManager;
pub use traits::*;
pub use types::*;

// Re-export ResourceLimits from core
pub use crate::core::types::ResourceLimits;
