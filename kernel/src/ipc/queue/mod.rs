/*!
 * Queue Module
 * Async message queues with multiple queue types
 */

pub mod fifo;
pub mod lifecycle;
pub mod manager;
pub mod operations;
pub mod priority;
pub mod pubsub;
pub mod subscription;
pub mod types;

// Re-export public API
pub use manager::QueueManager;
pub use types::{QueueMessage, QueueStats};
