/*!
 * Signal Core - Types and Traits
 * Fundamental types, internal types, and trait definitions for signal handling
 */

pub mod atomic_stats;
pub(crate) mod internal_types; // Internal types accessible within signals module
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use atomic_stats::AtomicSignalStats;
pub use traits::*;
pub use types::*;
