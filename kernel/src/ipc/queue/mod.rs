/*!
 * Queue Module
 * Async message queues with multiple queue types
 */

pub mod fifo;
pub mod manager;
pub mod priority;
pub mod pubsub;
pub mod types;

// Re-export public API
pub use manager::QueueManager;
pub use types::{QueueMessage, QueueStats};
