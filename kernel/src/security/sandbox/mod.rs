/*!
 * Modular Sandbox System
 *
 * Addresses architectural concerns:
 * - Granular capabilities with path-specific permissions
 * - TOCTOU-safe path handling with canonicalized handles
 * - Fine-grained network control with host/port/CIDR rules
 */

pub mod capability;
pub mod config;
pub mod manager;
pub mod network;
pub mod path;

pub use manager::SandboxManager;
