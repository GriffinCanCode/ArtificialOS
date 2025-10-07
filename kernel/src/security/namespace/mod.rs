/*!
 * Network Namespace Isolation Module
 * Cross-platform network isolation with platform-specific implementations
 */

mod bridge;
mod linux;
mod macos;
mod manager;
mod simulation;
mod traits;
mod types;
mod veth;

pub use bridge::BridgeManager;
pub use manager::NamespaceManager;
pub use traits::*;
pub use types::*;
pub use veth::VethManager;
