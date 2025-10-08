/*!
 * Async Syscall Traits (Rust 1.75+)
 *
 * Modern async trait definitions using native async in traits (stabilized in Rust 1.75).
 *
 * ## Design Philosophy
 *
 * These traits provide zero-cost async abstractions for syscalls. Unlike the old
 * approach of using `tokio::spawn_blocking`, these traits allow:
 *
 * 1. **True async I/O** - Operations yield to executor instead of blocking threads
 * 2. **Zero-cost fast path** - Fast syscalls can still be synchronous
 * 3. **Composable futures** - Natural async/await syntax
 * 4. **Better backpressure** - Async naturally handles flow control
 *
 * ## Usage Pattern
 *
 * ```ignore
 * impl AsyncFileSystemSyscalls for MyExecutor {
 *     async fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
 *         // True async I/O - no spawn_blocking needed
 *         let content = tokio::fs::read(path).await?;
 *         SyscallResult::success_with_data(content)
 *     }
 * }
 * ```
 *
 * ## Performance Characteristics
 *
 * - **Synchronous syscalls**: Inlined, < 100ns overhead
 * - **Async syscalls**: ~1-10μs dispatch, but non-blocking
 * - **Memory**: Futures are stack-allocated when possible
 */

use crate::core::types::Pid;
use crate::syscalls::types::*;
use std::path::PathBuf;

// ============================================================================
// File System - Async I/O Operations
// ============================================================================

/// Async file system syscall operations
///
/// These operations involve kernel I/O and should use true async I/O
/// (tokio::fs, io_uring, etc.) instead of spawn_blocking.
pub trait AsyncFileSystemSyscalls: Send + Sync {
    /// Read a file asynchronously
    async fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Write a file asynchronously
    async fn write_file(&self, pid: Pid, path: &PathBuf, data: &[u8]) -> SyscallResult;

    /// Create a file asynchronously
    async fn create_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Delete a file asynchronously
    async fn delete_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// List directory contents asynchronously
    async fn list_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Check if file exists (may use VFS cache or async check)
    async fn file_exists(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get file metadata asynchronously
    async fn file_stat(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Move/rename file asynchronously
    async fn move_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Copy file asynchronously
    async fn copy_file(&self, pid: Pid, source: &PathBuf, destination: &PathBuf) -> SyscallResult;

    /// Create directory asynchronously
    async fn create_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Remove directory asynchronously
    async fn remove_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Get working directory (fast, may be synchronous internally)
    async fn get_working_directory(&self, pid: Pid) -> SyscallResult;

    /// Set working directory asynchronously
    async fn set_working_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult;

    /// Truncate file to specified size asynchronously
    async fn truncate_file(&self, pid: Pid, path: &PathBuf, size: u64) -> SyscallResult;
}

// ============================================================================
// Process Management - Mixed Fast/Slow Operations
// ============================================================================

/// Async process management syscalls
///
/// Most process queries are fast (in-memory), but spawn/wait are blocking.
pub trait AsyncProcessSyscalls: Send + Sync {
    /// Spawn a new process asynchronously (blocks on fork/exec)
    async fn spawn_process(&self, pid: Pid, command: &str, args: &[String]) -> SyscallResult;

    /// Kill a process (fast permission check + signal)
    async fn kill_process(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process information (fast, in-memory)
    async fn get_process_info(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get list of all processes (fast, in-memory)
    async fn get_process_list(&self, pid: Pid) -> SyscallResult;

    /// Set process priority (fast, in-memory)
    async fn set_process_priority(&self, pid: Pid, target_pid: Pid, priority: u8) -> SyscallResult;

    /// Get process state (fast, in-memory)
    async fn get_process_state(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get process statistics (fast, in-memory)
    async fn get_process_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Wait for process to complete (blocking)
    async fn wait_process(
        &self,
        pid: Pid,
        target_pid: Pid,
        timeout_ms: Option<u64>,
    ) -> SyscallResult;
}

// ============================================================================
// IPC - Naturally Async Operations
// ============================================================================

/// Async inter-process communication syscalls
///
/// IPC operations are naturally async - they coordinate between processes
/// and can block on buffer availability.
pub trait AsyncIpcSyscalls: Send + Sync {
    /// Create a pipe (fast allocation)
    async fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult;

    /// Write to a pipe (can block on full buffer)
    async fn write_pipe(&self, pid: Pid, pipe_id: u32, data: &[u8]) -> SyscallResult;

    /// Read from a pipe (can block on empty buffer)
    async fn read_pipe(&self, pid: Pid, pipe_id: u32, size: usize) -> SyscallResult;

    /// Close a pipe end (fast cleanup)
    async fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Destroy a pipe (fast cleanup)
    async fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Get pipe statistics (fast, in-memory)
    async fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult;

    /// Create shared memory segment (kernel allocation)
    async fn create_shm(&self, pid: Pid, size: usize) -> SyscallResult;

    /// Attach to shared memory segment (page table update)
    async fn attach_shm(&self, pid: Pid, segment_id: u32, read_only: bool) -> SyscallResult;

    /// Detach from shared memory segment (page table update)
    async fn detach_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Write to shared memory (potential page fault)
    async fn write_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        data: &[u8],
    ) -> SyscallResult;

    /// Read from shared memory (potential page fault)
    async fn read_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        size: usize,
    ) -> SyscallResult;

    /// Destroy shared memory segment (kernel deallocation)
    async fn destroy_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Get shared memory statistics (fast, in-memory)
    async fn shm_stats(&self, pid: Pid, segment_id: u32) -> SyscallResult;

    /// Create async queue
    async fn create_queue(
        &self,
        pid: Pid,
        queue_type: &str,
        capacity: Option<usize>,
    ) -> SyscallResult;

    /// Send message to queue (can block on full)
    async fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult;

    /// Receive message from queue (can block on empty)
    async fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Subscribe to PubSub queue
    async fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Unsubscribe from PubSub queue
    async fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Close queue
    async fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Destroy queue
    async fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult;

    /// Get queue statistics (fast, in-memory)
    async fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult;
}

// ============================================================================
// Network - Naturally Async I/O
// ============================================================================

/// Async network syscalls
///
/// Network operations are inherently async due to latency and TCP semantics.
pub trait AsyncNetworkSyscalls: Send + Sync {
    /// Create a socket
    async fn socket(&self, pid: Pid, domain: u32, socket_type: u32, protocol: u32)
        -> SyscallResult;

    /// Bind socket to address
    async fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Listen on socket
    async fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult;

    /// Accept incoming connection (blocking)
    async fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult;

    /// Connect to remote address (blocking)
    async fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult;

    /// Send data on socket (can block)
    async fn send(&self, pid: Pid, sockfd: u32, data: &[u8], flags: u32) -> SyscallResult;

    /// Receive data from socket (can block)
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

    /// Make network request (higher-level, blocks on I/O)
    async fn network_request(&self, pid: Pid, url: &str) -> SyscallResult;
}

// ============================================================================
// File Descriptors - Fast Operations
// ============================================================================

/// Async file descriptor syscalls
///
/// Most FD operations are fast (in-memory table updates), but open/close
/// may involve I/O.
pub trait AsyncFileDescriptorSyscalls: Send + Sync {
    /// Open file and return file descriptor (kernel I/O)
    async fn open(&self, pid: Pid, path: &PathBuf, flags: u32, mode: u32) -> SyscallResult;

    /// Close file descriptor (kernel call)
    async fn close(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor (fast, in-memory)
    async fn dup(&self, pid: Pid, fd: u32) -> SyscallResult;

    /// Duplicate file descriptor to specific number (fast, in-memory)
    async fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult;

    /// Seek within file (fast if cached, slow if seeks on disk)
    async fn lseek(&self, pid: Pid, fd: u32, offset: i64, whence: u32) -> SyscallResult;

    /// File control operations (fast, in-memory)
    async fn fcntl(&self, pid: Pid, fd: u32, cmd: u32, arg: u32) -> SyscallResult;
}

// ============================================================================
// Memory - Fast Operations
// ============================================================================

/// Async memory management syscalls
///
/// Most memory operations are fast (DashMap lookups), but GC can block.
pub trait AsyncMemorySyscalls: Send + Sync {
    /// Get system memory statistics (fast, in-memory)
    async fn get_memory_stats(&self, pid: Pid) -> SyscallResult;

    /// Get process memory statistics (fast, in-memory)
    async fn get_process_memory_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Trigger garbage collection (can block)
    async fn trigger_gc(&self, pid: Pid, target_pid: Option<u32>) -> SyscallResult;
}

// ============================================================================
// Scheduler - Fast with Occasional Blocking
// ============================================================================

/// Async scheduler syscalls (delegated to scheduler module)
///
/// Most scheduler operations are fast state updates, but yield can block.
// pub use crate::scheduler::AsyncSchedulerSyscalls;

// ============================================================================
// Signals - Fast Delivery
// ============================================================================

/// Async signal syscalls
///
/// Signal operations are typically fast, but delivery can trigger handlers.
pub trait AsyncSignalSyscalls: Send + Sync {
    /// Send signal to process (fast delivery + handler trigger)
    async fn send_signal(&self, pid: Pid, target_pid: Pid, signal: u32) -> SyscallResult;

    /// Register signal handler (fast, in-memory)
    async fn register_signal_handler(
        &self,
        pid: Pid,
        signal: u32,
        handler_id: u64,
    ) -> SyscallResult;

    /// Block signal (fast, in-memory)
    async fn block_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Unblock signal (fast, in-memory)
    async fn unblock_signal(&self, pid: Pid, signal: u32) -> SyscallResult;

    /// Get pending signals (fast, in-memory)
    async fn get_pending_signals(&self, pid: Pid) -> SyscallResult;
}

// ============================================================================
// System Info - All Fast Operations
// ============================================================================

/// Async system information syscalls
///
/// All system info operations are fast (cached or simple calculations).
pub trait AsyncSystemInfoSyscalls: Send + Sync {
    /// Get system information (fast, cached)
    async fn get_system_info(&self, pid: Pid) -> SyscallResult;

    /// Get current time (fast, syscall)
    async fn get_current_time(&self, pid: Pid) -> SyscallResult;

    /// Get environment variable (fast, HashMap lookup)
    async fn get_env_var(&self, pid: Pid, key: &str) -> SyscallResult;

    /// Set environment variable (fast, HashMap insert)
    async fn set_env_var(&self, pid: Pid, key: &str, value: &str) -> SyscallResult;
}

// ============================================================================
// Time - Blocking Operations
// ============================================================================

/// Async time-related syscalls
///
/// Sleep operations are inherently blocking and benefit from true async.
pub trait AsyncTimeSyscalls: Send + Sync {
    /// Sleep for specified duration (truly async, doesn't block thread)
    async fn sleep(&self, pid: Pid, duration_ms: u64) -> SyscallResult;

    /// Get system uptime (fast, calculation)
    async fn get_uptime(&self, pid: Pid) -> SyscallResult;
}

// ============================================================================
// Combined Async Executor Trait
// ============================================================================

/// Complete async syscall executor trait combining all categories
///
/// This trait represents a fully async syscall executor. Implementations
/// can choose to make fast operations synchronous internally while exposing
/// an async interface for consistency.
pub trait AsyncSyscallExecutorTrait:
    AsyncFileSystemSyscalls
    + AsyncProcessSyscalls
    + AsyncIpcSyscalls
    + AsyncNetworkSyscalls
    + AsyncFileDescriptorSyscalls
    + AsyncMemorySyscalls
    // + AsyncSchedulerSyscalls
    + AsyncSignalSyscalls
    + AsyncSystemInfoSyscalls
    + AsyncTimeSyscalls
    + Clone
    + Send
    + Sync
{
    /// Execute a syscall asynchronously by dispatching to appropriate handler
    ///
    /// This is the main entry point for async syscall execution. The executor
    /// should use classification to determine whether to use fast-path (sync)
    /// or slow-path (async) execution.
    async fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult;
}
