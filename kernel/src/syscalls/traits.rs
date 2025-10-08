/*!
 * Syscall Traits (Async - Rust 1.75+)
 * Modern async trait definitions using native async in traits
 */

use super::types::*;
use crate::core::types::Pid;
use std::path::PathBuf;

/// File system syscall operations (async)
/// Uses native async traits (stabilized in Rust 1.75)
pub trait FileSystemSyscalls: Send + Sync {
    /// Read a file
    async fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Write a file
    async fn write_file(&self, pid: Pid, path: &PathBuf, data: &[u8]) -> SyscallResult;

    /// Create a file
    async fn create_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Delete a file
    async fn delete_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// List directory contents
    async fn list_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Check if file exists
    async fn file_exists(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get file metadata
    async fn file_stat(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Move/rename file
    async fn move_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Copy file
    async fn copy_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Create directory
    async fn create_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Remove directory
    async fn remove_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get working directory
    async fn get_working_directory(&self, pid: Pid) -> SyscallResult;

    /// Set working directory
    async fn set_working_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Truncate file to specified size
    async fn truncate_file(&self, pid: Pid, path: &PathBuf, size: u64) -> SyscallResult;
}

/// Process management syscalls (async)
pub trait ProcessSyscalls: Send + Sync {
    /// Spawn a new process
    async fn spawn_process(&self, pid: Pid, command: &str, args: &[String]) -> SyscallResult;

    /// Kill a process
    async fn kill_process(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process information
    async fn get_process_info(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get list of all processes
    async fn get_process_list(&self, pid: Pid) -> SyscallResult;

    /// Set process priority
    async fn set_process_priority(&self, pid: Pid, target_pid: Pid, priority: u8) -> SyscallResult;

    /// Get process state
    async fn get_process_state(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process statistics
    async fn get_process_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Wait for process to complete
    async fn wait_process(
        &self,
        pid: Pid,
        target_pid: Pid,
        timeout_ms: Option<u64>,
    ) -> SyscallResult;
}

/// Inter-process communication syscalls (async)
pub trait IpcSyscalls: Send + Sync {
    /// Create a pipe
    async fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult;

    /// Write to a pipe
    async fn write_pipe(&self, pid: Pid, pipe_id: u32, data: &[u8]) -> SyscallResult;

    /// Read from a pipe
    async fn read_pipe(&self, pid: Pid, pipe_id: u32, size: usize) -> SyscallResult;

    /// Close a pipe end
    async fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Destroy a pipe
    async fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Get pipe statistics
    async fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Create shared memory segment
    async fn create_shm(&self, pid: Pid, size: usize) -> SyscallResult;

    /// Attach to shared memory segment
    async fn attach_shm(&self, pid: Pid, segment_id: u32, read_only: bool) -> SyscallResult;

    /// Detach from shared memory segment
    async fn detach_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Write to shared memory
    async fn write_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        data: &[u8],
    ) -> SyscallResult;

    /// Read from shared memory
    async fn read_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        size: usize,
    ) -> SyscallResult;

    /// Destroy shared memory segment
    async fn destroy_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Get shared memory statistics
    async fn shm_stats(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Create async queue
    async fn create_queue(
        &self,
        pid: Pid,
        queue_type: &str,
        capacity: Option<usize>,
    ) -> SyscallResult;

    /// Send message to queue
    async fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult;

    /// Receive message from queue
    async fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Subscribe to PubSub queue
    async fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Unsubscribe from PubSub queue
    async fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Close queue
    async fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Destroy queue
    async fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Get queue statistics
    async fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult;
}

/// Network syscalls (async)
pub trait NetworkSyscalls: Send + Sync {
    /// Create a socket
    async fn socket(&self, pid: Pid, domain: u32, socket_type: u32, protocol: u32)
        -> SyscallResult;

    /// Bind socket to address
    async fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Listen on socket
    async fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult;

    /// Accept incoming connection
    async fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult;

    /// Connect to remote address
    async fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Send data on socket
    async fn send(&self, pid: Pid, sockfd: u32, data: &[u8], flags: u32) -> SyscallResult;

    /// Receive data from socket
    async fn recv(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult;

    /// Send data to specific address
    async fn sendto(
        &self,
        pid: Pid,
        sockfd: u32,
        data: &[u8],
        address: &str,
        flags: u32,
    ) -> SyscallResult;

    /// Receive data with source address
    async fn recvfrom(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult;

    /// Close socket
    async fn close_socket(&self, pid: Pid, sockfd: u32) -> SyscallResult;

    /// Set socket option
    async fn setsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
        optval: &[u8],
    ) -> SyscallResult;

    /// Get socket option
    async fn getsockopt(&self, pid: Pid, sockfd: u32, level: u32, optname: u32) -> SyscallResult;

    /// Make network request (higher-level)
    async fn network_request(&self, pid: Pid, url: &str) -> SyscallResult;
}

/// File descriptor syscalls (async)
pub trait FileDescriptorSyscalls: Send + Sync {
    /// Open file and return file descriptor
    async fn open(&self, pid: Pid, path: &PathBuf, flags: u32, mode: u32) -> SyscallResult;

    /// Close file descriptor
    async fn close(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor
    async fn dup(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor to specific number
    async fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult;

    /// Seek within file
    async fn lseek(&self, pid: Pid, fd: u32, offset: i64, whence: u32) -> SyscallResult;

    /// File control operations
    async fn fcntl(&self, pid: Pid, fd: u32, cmd: u32, arg: u32) -> SyscallResult;
}

/// Memory management syscalls (async)
pub trait MemorySyscalls: Send + Sync {
    /// Get system memory statistics
    async fn get_memory_stats(&self, pid: Pid) -> SyscallResult;

    /// Get process memory statistics
    async fn get_process_memory_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Trigger garbage collection
    async fn trigger_gc(&self, pid: Pid, target_pid: Option<u32>) -> SyscallResult;
}

/// Scheduler syscalls (delegated to scheduler module)
pub use crate::scheduler::{
    PriorityControl, SchedulerControl, SchedulerPolicy, SchedulerStats, SchedulerSyscalls,
};

/// Signal syscalls (async)
pub trait SignalSyscalls: Send + Sync {
    /// Send signal to process
    async fn send_signal(&self, pid: Pid, target_pid: Pid, signal: u32) -> SyscallResult;

    /// Register signal handler
    async fn register_signal_handler(
        &self,
        pid: Pid,
        signal: u32,
        handler_id: u64,
    ) -> SyscallResult;

    /// Block signal
    async fn block_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Unblock signal
    async fn unblock_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Get pending signals
    async fn get_pending_signals(&self, pid: Pid) -> SyscallResult;
}

/// System information syscalls (async)
pub trait SystemInfoSyscalls: Send + Sync {
    /// Get system information
    async fn get_system_info(&self, pid: Pid) -> SyscallResult;

    /// Get current time
    async fn get_current_time(&self, pid: Pid) -> SyscallResult;

    /// Get environment variable
    async fn get_env_var(&self, pid: Pid, key: &str) -> SyscallResult;

    /// Set environment variable
    async fn set_env_var(&self, pid: Pid, key: &str, value: &str) -> SyscallResult;
}

/// Time-related syscalls (async)
pub trait TimeSyscalls: Send + Sync {
    /// Sleep for specified duration
    async fn sleep(&self, pid: Pid, duration_ms: u64) -> SyscallResult;

    /// Get system uptime
    async fn get_uptime(&self, pid: Pid) -> SyscallResult;
}

/// Complete async syscall executor trait combining all categories
pub trait SyscallExecutorTrait:
    FileSystemSyscalls
    + ProcessSyscalls
    + IpcSyscalls
    + NetworkSyscalls
    + FileDescriptorSyscalls
    + MemorySyscalls
    + SchedulerSyscalls
    + SignalSyscalls
    + SystemInfoSyscalls
    + TimeSyscalls
    + Clone
    + Send
    + Sync
{
    /// Execute a syscall asynchronously by dispatching to appropriate handler
    async fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult;
}
