/*!
 * Timeout-aware IPC Operations
 *
 * Blocking IPC operations with configurable timeouts for pipes and queues.
 */

use super::super::pipe::{PipeError, PipeManager};
use super::super::queue::{QueueManager, QueueMessage};
use super::super::core::types::{IpcError, IpcResult, PipeId, QueueId};
use crate::core::guard::{TimeoutPolicy, TimeoutPolicyExt};
use crate::core::sync::{WaitError, WaitQueue};
use crate::core::types::{Pid, Priority, Size};
use std::sync::Arc;
use std::time::Instant;

/// Timeout-aware pipe operations
pub struct TimeoutPipeOps {
    manager: Arc<PipeManager>,
    wait_queue: Arc<WaitQueue<PipeId>>,
}

impl TimeoutPipeOps {
    /// Create new timeout-aware pipe operations
    ///
    /// # Performance
    ///
    /// Uses the PipeManager's shared wait queue for optimal coordination.
    /// The manager will automatically wake this wrapper when data/space becomes available.
    pub fn new(manager: Arc<PipeManager>) -> Self {
        Self {
            wait_queue: manager.wait_queue(), // ✅ Use shared wait queue from manager
            manager,
        }
    }

    /// Read from pipe with timeout
    ///
    /// Blocks until data is available or timeout expires.
    pub fn read_timeout(
        &self,
        pipe_id: PipeId,
        pid: Pid,
        size: Size,
        timeout: TimeoutPolicy,
    ) -> Result<Vec<u8>, PipeError> {
        let start = Instant::now();

        loop {
            // Try non-blocking read
            match self.manager.read(pipe_id, pid, size) {
                Ok(data) => return Ok(data),
                Err(PipeError::WouldBlock(_)) => {
                    // Calculate remaining timeout
                    let elapsed = start.elapsed();
                    let remaining = timeout
                        .to_duration_opt()
                        .and_then(|d| d.checked_sub(elapsed));

                    if remaining.is_none() || remaining == Some(std::time::Duration::ZERO) {
                        return Err(PipeError::Timeout {
                            elapsed_ms: elapsed.as_millis() as u64,
                            timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                        });
                    }

                    // Wait for pipe to have data
                    match self.wait_queue.wait(pipe_id, remaining) {
                        Ok(()) => continue, // Retry read
                        Err(WaitError::Timeout) => {
                            return Err(PipeError::Timeout {
                                elapsed_ms: start.elapsed().as_millis() as u64,
                                timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                            });
                        }
                        Err(_) => {
                            return Err(PipeError::InvalidOperation("Wait cancelled".to_string()))
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Write to pipe with timeout
    ///
    /// Blocks until space is available or timeout expires.
    pub fn write_timeout(
        &self,
        pipe_id: PipeId,
        pid: Pid,
        data: &[u8],
        timeout: TimeoutPolicy,
    ) -> Result<Size, PipeError> {
        let start = Instant::now();

        loop {
            // Try non-blocking write
            match self.manager.write(pipe_id, pid, data) {
                Ok(written) => return Ok(written),
                Err(PipeError::WouldBlock(_)) => {
                    // Calculate remaining timeout
                    let elapsed = start.elapsed();
                    let remaining = timeout
                        .to_duration_opt()
                        .and_then(|d| d.checked_sub(elapsed));

                    if remaining.is_none() || remaining == Some(std::time::Duration::ZERO) {
                        return Err(PipeError::Timeout {
                            elapsed_ms: elapsed.as_millis() as u64,
                            timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                        });
                    }

                    // Wait for pipe to have space
                    match self.wait_queue.wait(pipe_id, remaining) {
                        Ok(()) => continue, // Retry write
                        Err(WaitError::Timeout) => {
                            return Err(PipeError::Timeout {
                                elapsed_ms: start.elapsed().as_millis() as u64,
                                timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                            });
                        }
                        Err(_) => {
                            return Err(PipeError::InvalidOperation("Wait cancelled".to_string()))
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Notify waiters that pipe has data/space available
    ///
    /// # Note
    ///
    /// This method is now redundant - the PipeManager automatically wakes waiters
    /// after successful read/write operations. Kept for backward compatibility.
    #[deprecated(note = "PipeManager now automatically wakes waiters. This call is redundant.")]
    pub fn notify(&self, pipe_id: PipeId) {
        self.wait_queue.wake_all(pipe_id);
    }
}

/// Timeout-aware queue operations
pub struct TimeoutQueueOps {
    manager: Arc<QueueManager>,
    wait_queue: Arc<WaitQueue<QueueId>>,
}

impl TimeoutQueueOps {
    /// Create new timeout-aware queue operations
    pub fn new(manager: Arc<QueueManager>) -> Self {
        Self {
            manager,
            wait_queue: Arc::new(WaitQueue::with_defaults()),
        }
    }

    /// Receive from queue with timeout
    ///
    /// Blocks until a message is available or timeout expires.
    pub fn receive_timeout(
        &self,
        queue_id: QueueId,
        pid: Pid,
        timeout: TimeoutPolicy,
    ) -> IpcResult<QueueMessage> {
        let start = Instant::now();

        loop {
            // Try non-blocking receive
            match self.manager.receive(queue_id, pid)? {
                Some(msg) => return Ok(msg),
                None => {
                    // Calculate remaining timeout
                    let elapsed = start.elapsed();
                    let remaining = timeout
                        .to_duration_opt()
                        .and_then(|d| d.checked_sub(elapsed));

                    if remaining.is_none() || remaining == Some(std::time::Duration::ZERO) {
                        return Err(IpcError::Timeout {
                            elapsed_ms: elapsed.as_millis() as u64,
                            timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                        });
                    }

                    // Wait for message
                    match self.wait_queue.wait(queue_id, remaining) {
                        Ok(()) => continue, // Retry receive
                        Err(WaitError::Timeout) => {
                            return Err(IpcError::Timeout {
                                elapsed_ms: start.elapsed().as_millis() as u64,
                                timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                            });
                        }
                        Err(_) => {
                            return Err(IpcError::InvalidOperation("Wait cancelled".to_string()))
                        }
                    }
                }
            }
        }
    }

    /// Send to queue (delegates to manager, queues typically don't block on send)
    pub fn send(
        &self,
        queue_id: QueueId,
        from_pid: Pid,
        data: Vec<u8>,
        priority: Option<Priority>,
    ) -> IpcResult<()> {
        let result = self.manager.send(queue_id, from_pid, data, priority);

        // Notify waiters that a message is available
        if result.is_ok() {
            self.notify(queue_id);
        }

        result
    }

    /// Notify waiters that a message is available
    ///
    /// # Usage
    ///
    /// Should be called after successful send operations. Unlike pipes,
    /// QueueManager doesn't auto-notify, so this call is still necessary.
    pub fn notify(&self, queue_id: QueueId) {
        self.wait_queue.wake_all(queue_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryManager;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_pipe_read_timeout() {
        let memory_manager = MemoryManager::new();
        let pipe_manager = Arc::new(PipeManager::new(memory_manager));
        let timeout_ops = Arc::new(TimeoutPipeOps::new(pipe_manager.clone()));

        let pipe_id = pipe_manager.create(1, 2, None).unwrap();

        let timeout_ops_clone = timeout_ops.clone();
        let handle = thread::spawn(move || {
            let timeout = TimeoutPolicy::Ipc(Duration::from_millis(100));
            timeout_ops_clone.read_timeout(pipe_id, 1, 100, timeout)
        });

        // Let it timeout
        let result = handle.join().unwrap();
        assert!(matches!(result, Err(PipeError::Timeout { .. })));
    }

    #[test]
    fn test_pipe_read_with_data() {
        let memory_manager = MemoryManager::new();
        let pipe_manager = Arc::new(PipeManager::new(memory_manager));
        let timeout_ops = Arc::new(TimeoutPipeOps::new(pipe_manager.clone()));

        let pipe_id = pipe_manager.create(1, 2, None).unwrap();

        // Write data - PipeManager will automatically wake readers
        pipe_manager.write(pipe_id, 2, b"hello").unwrap();
        // No need to call timeout_ops.notify() - PipeManager does it automatically

        // Read should succeed
        let timeout = TimeoutPolicy::Ipc(Duration::from_secs(1));
        let data = timeout_ops.read_timeout(pipe_id, 1, 100, timeout).unwrap();
        assert_eq!(data, b"hello");
    }
}
