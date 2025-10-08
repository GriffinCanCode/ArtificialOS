/*!
 * Permission Types Module
 * Core types and traits for permission system
 */

mod core;
mod traits;

pub use core::{
    Action, PermissionError, PermissionRequest, PermissionResponse, PermissionResult, Resource,
    ResourceType,
};
pub use traits::{PermissionChecker, PermissionProvider, PermissionSystem};

