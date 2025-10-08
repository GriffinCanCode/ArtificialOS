/*!
 * Syscall Submission Queue
 * io_uring-style submission queue for syscalls with lock-free ring buffer
 */

use super::IoUringError;
use crate::core::types::{Fd, Pid, Size, SockFd};
use crate::ipc::utils::lockfree_ring::LockFreeRing;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Submission queue for syscall operations (lock-free)
///
/// # Performance
/// - Lock-free ring buffer for zero-contention submissions
/// - Optimized for high-frequency syscall batching
#[repr(C, align(64))]
pub struct SyscallSubmissionQueue {
    ring: LockFreeRing<SyscallSubmissionEntry>,
    seq_counter: AtomicU64,
}

impl SyscallSubmissionQueue {
    /// Create a new submission queue (lock-free)
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: LockFreeRing::new(capacity),
            seq_counter: AtomicU64::new(0),
        }
    }

    /// Push an entry to the queue (lock-free)
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation
    pub fn push(&self, mut entry: SyscallSubmissionEntry) -> Result<u64, IoUringError> {
        let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
        entry.seq = seq;

        self.ring
            .push(entry)
            .map(|_| seq)
            .map_err(|_| IoUringError::SubmissionQueueFull)
    }

    /// Pop an entry from the queue (lock-free)
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation
    pub fn pop(&self) -> Option<SyscallSubmissionEntry> {
        self.ring.pop()
    }

    /// Pop multiple entries (for batch processing, lock-free)
    pub fn pop_batch(&self, max: usize) -> Vec<SyscallSubmissionEntry> {
        let mut batch = Vec::with_capacity(max);
        for _ in 0..max {
            if let Some(entry) = self.ring.pop() {
                batch.push(entry);
            } else {
                break;
            }
        }
        batch
    }

    /// Check if queue is empty (lock-free)
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Get available space (approximate, lock-free)
    pub fn available(&self) -> usize {
        self.ring.available()
    }

    /// Get pending count (approximate, lock-free)
    pub fn pending(&self) -> usize {
        self.ring.len()
    }
}

/// Submission entry for a syscall operation
#[derive(Debug, Clone)]
pub struct SyscallSubmissionEntry {
    /// Sequence number (assigned on submission)
    pub seq: u64,
    /// Process ID
    pub pid: Pid,
    /// Operation type
    pub op: SyscallOpType,
    /// User data (for correlation)
    pub user_data: u64,
}

impl SyscallSubmissionEntry {
    /// Create a new submission entry
    pub fn new(pid: Pid, op: SyscallOpType, user_data: u64) -> Self {
        Self {
            seq: 0, // Will be set when pushed
            pid,
            op,
            user_data,
        }
    }

    // Convenience constructors for common operations

    /// Create a read file operation
    pub fn read_file(pid: Pid, path: PathBuf, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::ReadFile { path }, user_data)
    }

    /// Create a write file operation
    pub fn write_file(pid: Pid, path: PathBuf, data: Vec<u8>, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::WriteFile { path, data }, user_data)
    }

    /// Create an open file operation
    pub fn open(pid: Pid, path: PathBuf, flags: u32, mode: u32, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::Open { path, flags, mode }, user_data)
    }

    /// Create a close file operation
    pub fn close(pid: Pid, fd: Fd, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::Close { fd }, user_data)
    }

    /// Create an fsync operation
    pub fn fsync(pid: Pid, fd: Fd, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::Fsync { fd }, user_data)
    }

    /// Create a socket send operation
    pub fn send(pid: Pid, sockfd: SockFd, data: Vec<u8>, flags: u32, user_data: u64) -> Self {
        Self::new(
            pid,
            SyscallOpType::Send {
                sockfd,
                data,
                flags,
            },
            user_data,
        )
    }

    /// Create a socket recv operation
    pub fn recv(pid: Pid, sockfd: SockFd, size: Size, flags: u32, user_data: u64) -> Self {
        Self::new(
            pid,
            SyscallOpType::Recv {
                sockfd,
                size,
                flags,
            },
            user_data,
        )
    }

    /// Create an accept operation
    pub fn accept(pid: Pid, sockfd: SockFd, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::Accept { sockfd }, user_data)
    }

    /// Create a connect operation
    pub fn connect(pid: Pid, sockfd: SockFd, address: String, user_data: u64) -> Self {
        Self::new(pid, SyscallOpType::Connect { sockfd, address }, user_data)
    }
}

/// Syscall operation types supported by io_uring-style completion
///
/// These are the operations that benefit most from async submission/completion:
/// - File I/O operations
/// - Network I/O operations
/// - Operations that can be batched effectively
#[derive(Debug, Clone)]
pub enum SyscallOpType {
    // File I/O operations
    ReadFile {
        path: PathBuf,
    },
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
    },
    Open {
        path: PathBuf,
        flags: u32,
        mode: u32,
    },
    Close {
        fd: Fd,
    },
    Fsync {
        fd: Fd,
    },
    Lseek {
        fd: Fd,
        offset: i64,
        whence: u32,
    },

    // Network I/O operations
    Send {
        sockfd: SockFd,
        data: Vec<u8>,
        flags: u32,
    },
    Recv {
        sockfd: SockFd,
        size: Size,
        flags: u32,
    },
    Accept {
        sockfd: SockFd,
    },
    Connect {
        sockfd: SockFd,
        address: String,
    },
    SendTo {
        sockfd: SockFd,
        data: Vec<u8>,
        address: String,
        flags: u32,
    },
    RecvFrom {
        sockfd: SockFd,
        size: Size,
        flags: u32,
    },

    // IPC operations (integrates with existing zero-copy IPC)
    IpcSend {
        target_pid: Pid,
        data: Vec<u8>,
    },
    IpcRecv {
        size: Size,
    },
}

impl SyscallOpType {
    /// Check if this operation is I/O bound and benefits from async completion
    pub fn is_io_bound(&self) -> bool {
        matches!(
            self,
            SyscallOpType::ReadFile { .. }
                | SyscallOpType::WriteFile { .. }
                | SyscallOpType::Open { .. }
                | SyscallOpType::Fsync { .. }
                | SyscallOpType::Send { .. }
                | SyscallOpType::Recv { .. }
                | SyscallOpType::Accept { .. }
                | SyscallOpType::Connect { .. }
                | SyscallOpType::SendTo { .. }
                | SyscallOpType::RecvFrom { .. }
        )
    }

    /// Check if this operation can be batched efficiently
    pub fn is_batchable(&self) -> bool {
        matches!(
            self,
            SyscallOpType::ReadFile { .. }
                | SyscallOpType::WriteFile { .. }
                | SyscallOpType::Send { .. }
                | SyscallOpType::Recv { .. }
        )
    }

    /// Get operation name for logging
    pub fn name(&self) -> &'static str {
        match self {
            SyscallOpType::ReadFile { .. } => "read_file",
            SyscallOpType::WriteFile { .. } => "write_file",
            SyscallOpType::Open { .. } => "open",
            SyscallOpType::Close { .. } => "close",
            SyscallOpType::Fsync { .. } => "fsync",
            SyscallOpType::Lseek { .. } => "lseek",
            SyscallOpType::Send { .. } => "send",
            SyscallOpType::Recv { .. } => "recv",
            SyscallOpType::Accept { .. } => "accept",
            SyscallOpType::Connect { .. } => "connect",
            SyscallOpType::SendTo { .. } => "sendto",
            SyscallOpType::RecvFrom { .. } => "recvfrom",
            SyscallOpType::IpcSend { .. } => "ipc_send",
            SyscallOpType::IpcRecv { .. } => "ipc_recv",
        }
    }
}
