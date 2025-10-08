/*!
 * Core Module
 * Fundamental kernel types and error handling
 */

pub mod bincode;
pub mod const_generics;
pub mod errors;
pub mod hints;
pub mod json;
pub mod serde;
pub mod sync;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use errors::*;
pub use hints::*;
pub use traits::*;
pub use types::*;
