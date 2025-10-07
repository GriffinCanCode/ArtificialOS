/*!
 * Syscall Traits
 * Syscall execution abstractions
 */

use super::types::*;
use crate::core::types::Pid;
use std::path::PathBuf;

/// File system syscall operations
pub trait FileSystemSyscalls: Send + Sync {
    /// Read a file
    fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Write a file
    fn write_file(&self, pid: Pid, path: &PathBuf, data: &[u8]) -> SyscallResult;

    /// Create a file
    fn create_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Delete a file
    fn delete_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// List directory contents
    fn list_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Check if file exists
    fn file_exists(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get file metadata
    fn file_stat(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Move/rename file
    fn move_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Copy file
    fn copy_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Create directory
    fn create_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Remove directory
    fn remove_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get working directory
    fn get_working_directory(&self, pid: Pid) -> SyscallResult;

    /// Set working directory
    fn set_working_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Truncate file to specified size
    fn truncate_file(&self, pid: Pid, path: &PathBuf, size: u64) -> SyscallResult;
}

/// Process management syscalls
pub trait ProcessSyscalls: Send + Sync {
    /// Spawn a new process
    fn spawn_process(&self, pid: Pid, command: &str, args: &[String]) -> SyscallResult;

    /// Kill a process
    fn kill_process(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process information
    fn get_process_info(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get list of all processes
    fn get_process_list(&self, pid: Pid) -> SyscallResult;

    /// Set process priority
    fn set_process_priority(&self, pid: Pid, target_pid: Pid, priority: u8) -> SyscallResult;

    /// Get process state
    fn get_process_state(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process statistics
    fn get_process_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Wait for process to complete
    fn wait_process(&self, pid: Pid, target_pid: Pid, timeout_ms: Option<u64>) -> SyscallResult;
}

/// Inter-process communication syscalls
pub trait IpcSyscalls: Send + Sync {
    /// Create a pipe
    fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult;

    /// Write to a pipe
    fn write_pipe(&self, pid: Pid, pipe_id: u32, data: &[u8]) -> SyscallResult;

    /// Read from a pipe
    fn read_pipe(&self, pid: Pid, pipe_id: u32, size: usize) -> SyscallResult;

    /// Close a pipe end
    fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Destroy a pipe
    fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Get pipe statistics
    fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Create shared memory segment
    fn create_shm(&self, pid: Pid, size: usize) -> SyscallResult;

    /// Attach to shared memory segment
    fn attach_shm(&self, pid: Pid, segment_id: u32, read_only: bool) -> SyscallResult;

    /// Detach from shared memory segment
    fn detach_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Write to shared memory
    fn write_shm(&self, pid: Pid, segment_id: u32, offset: usize, data: &[u8]) -> SyscallResult;

    /// Read from shared memory
    fn read_shm(&self, pid: Pid, segment_id: u32, offset: usize, size: usize) -> SyscallResult;

    /// Destroy shared memory segment
    fn destroy_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Get shared memory statistics
    fn shm_stats(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Create async queue
    fn create_queue(&self, pid: Pid, queue_type: &str, capacity: Option<usize>) -> SyscallResult;

    /// Send message to queue
    fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult;

    /// Receive message from queue
    fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Subscribe to PubSub queue
    fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Unsubscribe from PubSub queue
    fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Close queue
    fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Destroy queue
    fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Get queue statistics
    fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult;
}

/// Network syscalls
pub trait NetworkSyscalls: Send + Sync {
    /// Create a socket
    fn socket(&self, pid: Pid, domain: u32, socket_type: u32, protocol: u32) -> SyscallResult;

    /// Bind socket to address
    fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Listen on socket
    fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult;

    /// Accept incoming connection
    fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult;

    /// Connect to remote address
    fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Send data on socket
    fn send(&self, pid: Pid, sockfd: u32, data: &[u8], flags: u32) -> SyscallResult;

    /// Receive data from socket
    fn recv(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult;

    /// Send data to specific address
    fn sendto(
        &self,
        pid: Pid,
        sockfd: u32,
        data: &[u8],
        address: &str,
        flags: u32,
    ) -> SyscallResult;

    /// Receive data with source address
    fn recvfrom(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult;

    /// Close socket
    fn close_socket(&self, pid: Pid, sockfd: u32) -> SyscallResult;

    /// Set socket option
    fn setsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
        optval: &[u8],
    ) -> SyscallResult;

    /// Get socket option
    fn getsockopt(&self, pid: Pid, sockfd: u32, level: u32, optname: u32) -> SyscallResult;

    /// Make network request (higher-level)
    fn network_request(&self, pid: Pid, url: &str) -> SyscallResult;
}

/// File descriptor syscalls
pub trait FileDescriptorSyscalls: Send + Sync {
    /// Open file and return file descriptor
    fn open(&self, pid: Pid, path: &PathBuf, flags: u32, mode: u32) -> SyscallResult;

    /// Close file descriptor
    fn close(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor
    fn dup(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor to specific number
    fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult;

    /// Seek within file
    fn lseek(&self, pid: Pid, fd: u32, offset: i64, whence: u32) -> SyscallResult;

    /// File control operations
    fn fcntl(&self, pid: Pid, fd: u32, cmd: u32, arg: u32) -> SyscallResult;
}

/// Memory management syscalls
pub trait MemorySyscalls: Send + Sync {
    /// Get system memory statistics
    fn get_memory_stats(&self, pid: Pid) -> SyscallResult;

    /// Get process memory statistics
    fn get_process_memory_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Trigger garbage collection
    fn trigger_gc(&self, pid: Pid, target_pid: Option<u32>) -> SyscallResult;
}

/// Scheduler syscalls (delegated to scheduler module)
pub use crate::scheduler::{
    PriorityControl, SchedulerControl, SchedulerPolicy, SchedulerStats, SchedulerSyscalls,
};

/// Signal syscalls (delegated to signals module)
pub trait SignalSyscalls: Send + Sync {
    /// Send signal to process
    fn send_signal(&self, pid: Pid, target_pid: Pid, signal: u32) -> SyscallResult;

    /// Register signal handler
    fn register_signal_handler(&self, pid: Pid, signal: u32, handler_id: u64) -> SyscallResult;

    /// Block signal
    fn block_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Unblock signal
    fn unblock_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Get pending signals
    fn get_pending_signals(&self, pid: Pid) -> SyscallResult;
}

/// System information syscalls
pub trait SystemInfoSyscalls: Send + Sync {
    /// Get system information
    fn get_system_info(&self, pid: Pid) -> SyscallResult;

    /// Get current time
    fn get_current_time(&self, pid: Pid) -> SyscallResult;

    /// Get environment variable
    fn get_env_var(&self, pid: Pid, key: &str) -> SyscallResult;

    /// Set environment variable
    fn set_env_var(&self, pid: Pid, key: &str, value: &str) -> SyscallResult;
}

/// Time-related syscalls
pub trait TimeSyscalls: Send + Sync {
    /// Sleep for specified duration
    fn sleep(&self, pid: Pid, duration_ms: u64) -> SyscallResult;

    /// Get system uptime
    fn get_uptime(&self, pid: Pid) -> SyscallResult;
}

/// Complete syscall executor trait combining all categories
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
    /// Execute a syscall by dispatching to appropriate handler
    fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult;
}
