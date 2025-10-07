/*!
 * Core Module
 * Fundamental kernel types and error handling
 */

pub mod bincode;
pub mod errors;
pub mod json;
pub mod serde;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use errors::*;
pub use traits::*;
pub use types::*;
