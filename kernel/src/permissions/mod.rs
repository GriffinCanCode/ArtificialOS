/*!
 * Permissions Module
 * Centralized permission checking, policy evaluation, and audit logging
 *
 * This module provides a unified entry point for all permission checks across
 * the kernel, replacing scattered permission logic with a single source of truth.
 *
 * ## Features
 * - Unified permission model (capabilities, paths, network, IPC)
 * - Policy-based access control (PBAC)
 * - Comprehensive audit trail
 * - Permission check caching for performance
 * - Strong typing and extensibility
 *
 * ## Usage
 * ```rust
 * use ai_os_kernel::permissions::{PermissionManager, PermissionRequest, Action};
 *
 * let manager = PermissionManager::new();
 *
 * // Check file read permission
 * let request = PermissionRequest::file_read(pid, Path::new("/tmp/test.txt"));
 * if manager.check(&request).is_allowed() {
 *     // Perform operation
 * }
 *
 * // Check with audit
 * let result = manager.check_and_audit(&request);
 * if !result.is_allowed() {
 *     eprintln!("Denied: {}", result.reason());
 * }
 * ```
 */

mod audit;
mod cache;
mod context;
mod manager;
mod policy;
mod traits;
mod types;

pub use audit::{AuditEvent, AuditLogger, AuditSeverity};
pub use cache::PermissionCache;
pub use context::{EvaluationContext, RequestContext};
pub use manager::PermissionManager;
pub use policy::{Policy, PolicyDecision, PolicyEngine};
pub use traits::{PermissionChecker, PermissionProvider};
pub use types::{
    Action, PermissionRequest, PermissionResponse, PermissionResult, Resource, ResourceType,
};
