/*!
 * Adaptive I/O Dispatcher
 *
 * Phase 3: Intelligent selection between tokio::fs and io_uring
 * based on operation characteristics
 *
 * ## Strategy
 *
 * - **Small files (<64KB)**: tokio::fs (lower overhead, simpler)
 * - **Large files (≥64KB)**: io_uring (batching, zero-copy)
 * - **Sequential batches**: io_uring (amortized submission cost)
 * - **Single operations**: tokio::fs (no queue overhead)
 */

use super::io::AsyncFileOps;
use super::ipc::AsyncIpcOps;
use crate::core::types::Pid;
use crate::syscalls::iouring::{IoUringManager, SyscallOpType, SyscallSubmissionEntry};
use crate::syscalls::types::{Syscall, SyscallResult};
use std::sync::Arc;
use tracing::{debug, info};

/// Thresholds for adaptive dispatch
const LARGE_FILE_THRESHOLD: u64 = 64 * 1024; // 64KB
const BATCH_SIZE_THRESHOLD: usize = 4; // 4+ operations benefit from io_uring

/// Adaptive I/O dispatcher
///
/// Automatically chooses the best execution path based on:
/// 1. Operation size (small vs large)
/// 2. Batch size (single vs multiple)
/// 3. Operation type (file vs IPC)
pub struct AdaptiveDispatcher {
    /// tokio::fs operations for small/single operations
    file_ops: Arc<AsyncFileOps>,

    /// Async IPC operations
    ipc_ops: Arc<AsyncIpcOps>,

    /// io_uring for large/batched operations
    iouring_manager: Option<Arc<IoUringManager>>,

    /// Enable adaptive dispatch (can be disabled for testing)
    adaptive_enabled: bool,
}

impl AdaptiveDispatcher {
    /// Create new adaptive dispatcher
    pub fn new(
        file_ops: Arc<AsyncFileOps>,
        ipc_ops: Arc<AsyncIpcOps>,
        iouring_manager: Option<Arc<IoUringManager>>,
    ) -> Self {
        info!("Adaptive dispatcher initialized (io_uring: {})", iouring_manager.is_some());
        Self {
            file_ops,
            ipc_ops,
            iouring_manager,
            adaptive_enabled: true,
        }
    }

    /// Disable adaptive dispatch (always use tokio::fs)
    pub fn disable_adaptive(&mut self) {
        self.adaptive_enabled = false;
        debug!("Adaptive dispatch disabled");
    }

    /// Execute single syscall with adaptive path selection
    pub async fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        // If io_uring is not available, always use tokio::fs
        if !self.adaptive_enabled || self.iouring_manager.is_none() {
            return self.execute_tokio(pid, syscall).await;
        }

        // Classify operation for adaptive dispatch
        match self.classify_for_dispatch(&syscall) {
            DispatchPath::TokioFs => self.execute_tokio(pid, syscall).await,
            DispatchPath::IoUring => self.execute_iouring(pid, syscall).await,
        }
    }

    /// Execute batch of syscalls with adaptive path selection
    ///
    /// Large batches benefit significantly from io_uring's submission queue
    pub async fn execute_batch(&self, pid: Pid, syscalls: Vec<Syscall>) -> Vec<SyscallResult> {
        // For small batches, use tokio::fs concurrently
        if syscalls.len() < BATCH_SIZE_THRESHOLD || !self.adaptive_enabled || self.iouring_manager.is_none() {
            return self.execute_tokio_batch(pid, syscalls).await;
        }

        // For large batches, prefer io_uring for batching benefits
        self.execute_iouring_batch(pid, syscalls).await
    }

    // ========================================================================
    // Classification Logic
    // ========================================================================

    /// Classify operation for dispatch path
    fn classify_for_dispatch(&self, syscall: &Syscall) -> DispatchPath {
        match syscall {
            // File operations - check size
            Syscall::ReadFile { path } => {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() >= LARGE_FILE_THRESHOLD {
                        return DispatchPath::IoUring;
                    }
                }
                DispatchPath::TokioFs
            }

            Syscall::WriteFile { path: _, data } => {
                if data.len() as u64 >= LARGE_FILE_THRESHOLD {
                    return DispatchPath::IoUring;
                }
                DispatchPath::TokioFs
            }

            Syscall::CopyFile { source, .. } => {
                if let Ok(metadata) = std::fs::metadata(source) {
                    if metadata.len() >= LARGE_FILE_THRESHOLD {
                        return DispatchPath::IoUring;
                    }
                }
                DispatchPath::TokioFs
            }

            // Small operations always use tokio::fs
            Syscall::CreateFile { .. }
            | Syscall::DeleteFile { .. }
            | Syscall::FileExists { .. }
            | Syscall::FileStat { .. }
            | Syscall::MoveFile { .. }
            | Syscall::CreateDirectory { .. }
            | Syscall::RemoveDirectory { .. }
            | Syscall::TruncateFile { .. } => DispatchPath::TokioFs,

            // IPC operations - always tokio (flume async)
            Syscall::WritePipe { .. }
            | Syscall::ReadPipe { .. }
            | Syscall::SendQueue { .. }
            | Syscall::ReceiveQueue { .. } => DispatchPath::TokioFs,

            // Everything else uses tokio
            _ => DispatchPath::TokioFs,
        }
    }

    // ========================================================================
    // tokio::fs Execution Path
    // ========================================================================

    /// Execute using tokio::fs (true async I/O)
    async fn execute_tokio(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        match syscall {
            // File operations
            Syscall::ReadFile { path } => self.file_ops.read(pid, &path).await,
            Syscall::WriteFile { path, data } => self.file_ops.write(pid, &path, &data).await,
            Syscall::CreateFile { path } => self.file_ops.create(pid, &path).await,
            Syscall::DeleteFile { path } => self.file_ops.delete(pid, &path).await,
            Syscall::FileStat { path } => self.file_ops.metadata(pid, &path).await,
            Syscall::CopyFile { source, destination } => {
                self.file_ops.copy(pid, &source, &destination).await
            }
            Syscall::MoveFile { source, destination } => {
                self.file_ops.rename(pid, &source, &destination).await
            }
            Syscall::ListDirectory { path } => self.file_ops.read_dir(pid, &path).await,
            Syscall::CreateDirectory { path } => self.file_ops.create_dir(pid, &path).await,
            Syscall::RemoveDirectory { path } => self.file_ops.remove_dir(pid, &path).await,

            // IPC operations
            Syscall::WritePipe {
                pipe_id,
                data,
            } => {
                self.ipc_ops.pipe_write(pipe_id as u64, pid, &data, None).await
            }
            Syscall::ReadPipe {
                pipe_id,
                size,
            } => {
                self.ipc_ops.pipe_read(pipe_id as u64, pid, size, None).await
            }
            Syscall::SendQueue {
                queue_id,
                data,
                priority,
            } => {
                self.ipc_ops.queue_send(queue_id as u64, pid, data, priority, None).await
            }
            Syscall::ReceiveQueue { queue_id } => {
                self.ipc_ops.queue_receive(queue_id as u64, pid, None).await
            }

            // Not yet implemented - return error
            _ => SyscallResult::error(format!(
                "Syscall {} not implemented in tokio path",
                syscall.name()
            )),
        }
    }

    /// Execute batch using tokio::fs concurrently
    async fn execute_tokio_batch(&self, pid: Pid, syscalls: Vec<Syscall>) -> Vec<SyscallResult> {
        let futures: Vec<_> = syscalls
            .into_iter()
            .map(|syscall| self.execute_tokio(pid, syscall))
            .collect();

        futures::future::join_all(futures).await
    }

    // ========================================================================
    // io_uring Execution Path
    // ========================================================================

    /// Execute using io_uring (batched, zero-copy)
    async fn execute_iouring(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        let manager = match &self.iouring_manager {
            Some(m) => m,
            None => return SyscallResult::error("io_uring not available"),
        };

        // Convert syscall to io_uring operation
        let op = match self.syscall_to_iouring_op(&syscall) {
            Some(op) => op,
            None => {
                debug!("Syscall {} cannot use io_uring, falling back to tokio", syscall.name());
                return self.execute_tokio(pid, syscall).await;
            }
        };

        // Submit operation
        let entry = SyscallSubmissionEntry {
            seq: 0, // Will be assigned by manager
            pid,
            op,
            user_data: 0,
        };

        let seq = match manager.submit(pid, entry) {
            Ok(s) => s,
            Err(e) => return SyscallResult::error(format!("io_uring submit failed: {}", e)),
        };

        // Wait for completion
        match manager.wait_completion(pid, seq) {
            Ok(completion) => completion.result,
            Err(e) => SyscallResult::error(format!("io_uring wait failed: {}", e)),
        }
    }

    /// Execute batch using io_uring
    async fn execute_iouring_batch(&self, pid: Pid, syscalls: Vec<Syscall>) -> Vec<SyscallResult> {
        let manager = match &self.iouring_manager {
            Some(m) => m,
            None => return self.execute_tokio_batch(pid, syscalls).await,
        };

        // Convert syscalls to io_uring operations
        let entries: Vec<_> = syscalls
            .into_iter()
            .filter_map(|syscall| {
                self.syscall_to_iouring_op(&syscall).map(|op| SyscallSubmissionEntry {
                    seq: 0,
                    pid,
                    op,
                    user_data: 0,
                })
            })
            .collect();

        if entries.is_empty() {
            return vec![];
        }

        // Submit batch
        let seqs = match manager.submit_batch(pid, entries) {
            Ok(s) => s,
            Err(e) => return vec![SyscallResult::error(format!("Batch submit failed: {}", e))],
        };

        // Reap completions
        let mut results = Vec::with_capacity(seqs.len());
        for seq in seqs {
            let result = match manager.wait_completion(pid, seq) {
                Ok(completion) => completion.result,
                Err(e) => SyscallResult::error(format!("Completion failed: {}", e)),
            };
            results.push(result);
        }

        results
    }

    /// Convert Syscall to io_uring operation type
    fn syscall_to_iouring_op(&self, syscall: &Syscall) -> Option<SyscallOpType> {
        match syscall {
            Syscall::ReadFile { path } => Some(SyscallOpType::ReadFile { path: path.clone() }),
            Syscall::WriteFile { path, data } => Some(SyscallOpType::WriteFile {
                path: path.clone(),
                data: data.clone(),
            }),
            Syscall::Open { path, flags, mode } => Some(SyscallOpType::Open {
                path: path.clone(),
                flags: *flags,
                mode: *mode,
            }),
            Syscall::Close { fd } => Some(SyscallOpType::Close { fd: *fd }),
            // Network operations
            Syscall::Send {
                sockfd,
                data,
                flags,
            } => Some(SyscallOpType::Send {
                sockfd: *sockfd,
                data: data.clone(),
                flags: *flags,
            }),
            Syscall::Recv {
                sockfd,
                size,
                flags,
            } => Some(SyscallOpType::Recv {
                sockfd: *sockfd,
                size: *size,
                flags: *flags,
            }),
            // Not supported by io_uring yet
            _ => None,
        }
    }
}

/// Dispatch path classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchPath {
    /// Use tokio::fs (small operations, better latency)
    TokioFs,
    /// Use io_uring (large operations, better throughput)
    IoUring,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification() {
        // Small file -> tokio::fs
        let syscall = Syscall::WriteFile {
            path: PathBuf::from("/test.txt"),
            data: vec![0u8; 1024], // 1KB
        };

        let dispatcher = create_test_dispatcher();
        assert_eq!(dispatcher.classify_for_dispatch(&syscall), DispatchPath::TokioFs);

        // Large file -> io_uring
        let syscall = Syscall::WriteFile {
            path: PathBuf::from("/large.txt"),
            data: vec![0u8; 128 * 1024], // 128KB
        };

        assert_eq!(dispatcher.classify_for_dispatch(&syscall), DispatchPath::IoUring);
    }

    #[tokio::test]
    async fn test_batch_threshold() {
        let dispatcher = create_test_dispatcher();

        // Small batch (< 4) should use tokio::fs
        let syscalls = vec![
            Syscall::CreateFile { path: PathBuf::from("/test1.txt") },
            Syscall::CreateFile { path: PathBuf::from("/test2.txt") },
        ];

        // This should use tokio batch path
        let _results = dispatcher.execute_batch(1, syscalls).await;
    }

    fn create_test_dispatcher() -> AdaptiveDispatcher {
        use crate::memory::MemoryManager;
        use crate::security::SandboxManager;
        use crate::vfs::MountManager;

        let sandbox = Arc::new(SandboxManager::new());
        let mount_manager = Arc::new(MountManager::new());
        let file_ops = Arc::new(AsyncFileOps::new(sandbox, mount_manager));

        let memory_manager = MemoryManager::new();
        let pipe_manager = crate::ipc::PipeManager::new(memory_manager.clone());
        let shm_manager = crate::ipc::ShmManager::new(memory_manager.clone());
        let queue_manager = crate::ipc::QueueManager::new(memory_manager);
        let ipc_ops = Arc::new(AsyncIpcOps::new(pipe_manager, queue_manager, shm_manager));

        AdaptiveDispatcher::new(file_ops, ipc_ops, None)
    }
}

