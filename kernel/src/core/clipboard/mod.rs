/*!
 * Clipboard Module
 * Multi-format clipboard with history, subscriptions, and per-process isolation
 */

pub mod manager;
pub mod types;

pub use manager::ClipboardManager;
pub use types::{
    ClipboardData, ClipboardEntry, ClipboardError, ClipboardFormat, ClipboardResult,
    ClipboardStats, ClipboardSubscription,
};

