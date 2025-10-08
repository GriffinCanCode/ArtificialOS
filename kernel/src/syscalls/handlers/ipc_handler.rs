/*!
 * IPC Syscall Handler
 * Handles all inter-process communication syscalls (pipes, shm, queues)
 */

use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for IPC syscalls
pub struct IpcHandler {
    executor: SyscallExecutor,
}

impl IpcHandler {
    #[inline]
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for IpcHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            // Pipe operations
            Syscall::CreatePipe {
                reader_pid,
                writer_pid,
                capacity,
            } => Some(
                self.executor
                    .create_pipe(pid, *reader_pid, *writer_pid, *capacity),
            ),
            Syscall::WritePipe { pipe_id, ref data } => {
                Some(self.executor.write_pipe(pid, *pipe_id, data))
            }
            Syscall::ReadPipe { pipe_id, size } => {
                Some(self.executor.read_pipe(pid, *pipe_id, *size))
            }
            Syscall::ClosePipe { pipe_id } => Some(self.executor.close_pipe(pid, *pipe_id)),
            Syscall::DestroyPipe { pipe_id } => Some(self.executor.destroy_pipe(pid, *pipe_id)),
            Syscall::PipeStats { pipe_id } => Some(self.executor.pipe_stats(pid, *pipe_id)),

            // Shared memory operations
            Syscall::CreateShm { size } => Some(self.executor.create_shm(pid, *size)),
            Syscall::AttachShm {
                segment_id,
                read_only,
            } => Some(self.executor.attach_shm(pid, *segment_id, *read_only)),
            Syscall::DetachShm { segment_id } => Some(self.executor.detach_shm(pid, *segment_id)),
            Syscall::WriteShm {
                segment_id,
                offset,
                ref data,
            } => Some(self.executor.write_shm(pid, *segment_id, *offset, data)),
            Syscall::ReadShm {
                segment_id,
                offset,
                size,
            } => Some(self.executor.read_shm(pid, *segment_id, *offset, *size)),
            Syscall::DestroyShm { segment_id } => Some(self.executor.destroy_shm(pid, *segment_id)),
            Syscall::ShmStats { segment_id } => Some(self.executor.shm_stats(pid, *segment_id)),

            // Queue operations
            Syscall::CreateQueue {
                ref queue_type,
                capacity,
            } => Some(self.executor.create_queue(pid, queue_type, *capacity)),
            Syscall::SendQueue {
                queue_id,
                ref data,
                priority,
            } => Some(self.executor.send_queue(pid, *queue_id, data, *priority)),
            Syscall::ReceiveQueue { queue_id } => Some(self.executor.receive_queue(pid, *queue_id)),
            Syscall::SubscribeQueue { queue_id } => {
                Some(self.executor.subscribe_queue(pid, *queue_id))
            }
            Syscall::UnsubscribeQueue { queue_id } => {
                Some(self.executor.unsubscribe_queue(pid, *queue_id))
            }
            Syscall::CloseQueue { queue_id } => Some(self.executor.close_queue(pid, *queue_id)),
            Syscall::DestroyQueue { queue_id } => Some(self.executor.destroy_queue(pid, *queue_id)),
            Syscall::QueueStats { queue_id } => Some(self.executor.queue_stats(pid, *queue_id)),

            _ => None, // Not an IPC syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "ipc_handler"
    }
}
