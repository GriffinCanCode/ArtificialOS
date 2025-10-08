/*!
 * Garbage Collection
 * Global and system-wide memory cleanup
 */

pub mod collector;

pub use collector::{GcStats, GcStrategy, GlobalGarbageCollector};

