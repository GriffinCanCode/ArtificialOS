/*!
 * Observable VFS - Event System for File Watching
 * Minimal trait-based approach for file change notifications
 */

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;

/// File system events that can be observed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEvent {
    /// File or directory was created
    Created { path: PathBuf },

    /// File contents were modified
    Modified { path: PathBuf },

    /// File or directory was deleted
    Deleted { path: PathBuf },

    /// File or directory was renamed/moved
    Renamed { from: PathBuf, to: PathBuf },
}

impl FileEvent {
    /// Get the primary path involved in this event
    pub fn path(&self) -> &Path {
        match self {
            FileEvent::Created { path } => path,
            FileEvent::Modified { path } => path,
            FileEvent::Deleted { path } => path,
            FileEvent::Renamed { from, .. } => from,
        }
    }
}

/// Observable filesystem trait - opt-in for filesystems that support events
pub trait Observable: Send + Sync {
    /// Subscribe to file events
    /// Returns a receiver that will get all future events
    fn subscribe(&self) -> broadcast::Receiver<FileEvent>;

    /// Emit an event to all subscribers
    fn emit(&self, event: FileEvent);
}

/// Event broadcaster implementation
/// Uses tokio broadcast channel for lock-free MPMC
#[derive(Clone)]
pub struct EventBroadcaster {
    sender: Arc<broadcast::Sender<FileEvent>>,
}

impl EventBroadcaster {
    /// Create new broadcaster with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<FileEvent> {
        self.sender.subscribe()
    }

    /// Emit event to all subscribers
    pub fn emit(&self, event: FileEvent) {
        // Ignore errors - if no subscribers, that's fine
        let _ = self.sender.send(event);
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new(1024) // Default buffer: 1024 events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_broadcast() {
        let broadcaster = EventBroadcaster::new(10);

        // Subscribe before emitting
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        // Emit event
        broadcaster.emit(FileEvent::Created {
            path: PathBuf::from("/test.txt"),
        });

        // Both subscribers should receive it
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        assert_eq!(event1, event2);
        assert_eq!(event1.path(), Path::new("/test.txt"));
    }

    #[tokio::test]
    async fn test_late_subscriber_misses_events() {
        let broadcaster = EventBroadcaster::new(10);

        // Emit before subscribing
        broadcaster.emit(FileEvent::Modified {
            path: PathBuf::from("/test.txt"),
        });

        // Late subscriber
        let mut rx = broadcaster.subscribe();

        // Should timeout (no event)
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            rx.recv()
        ).await;

        assert!(result.is_err()); // Timeout
    }
}

