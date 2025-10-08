/*!
 * Syscall Classification System
 *
 * Compile-time classification of syscalls into fast-path (sync) and slow-path (async).
 *
 * ## Design Philosophy
 *
 * Not all syscalls are created equal. Some complete in nanoseconds (getpid, memory reads),
 * while others can block for milliseconds or longer (file I/O, network, sleep).
 *
 * By classifying syscalls at compile time, we can:
 * 1. Execute fast syscalls synchronously with zero async overhead
 * 2. Execute blocking syscalls asynchronously without blocking thread pool
 * 3. Maintain backward compatibility with existing code
 *
 * ## Performance Impact
 *
 * - Fast-path: Direct function call (< 100ns)
 * - Slow-path: Async dispatch (~1-10μs overhead, but non-blocking)
 */

use crate::syscalls::types::Syscall;

/// Classification of syscall execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallClass {
    /// Fast synchronous operations (< 100ns typical)
    /// - In-memory operations
    /// - Simple arithmetic/checks
    /// - No I/O or blocking
    Fast,

    /// Blocking operations that should run async
    /// - File I/O
    /// - Network operations
    /// - IPC operations
    /// - Sleep/wait operations
    Blocking,
}

impl Syscall {
    /// Classify a syscall for optimal execution strategy
    ///
    /// This classification is based on empirical performance characteristics
    /// and whether the operation can block.
    ///
    /// # Fast Syscalls (Synchronous)
    ///
    /// Operations that complete in < 100ns and never block:
    /// - Memory stats (in-memory DashMap lookup)
    /// - Process state queries (cached data)
    /// - Simple checks (file_exists via VFS cache)
    /// - System info (cached or simple calculations)
    ///
    /// # Blocking Syscalls (Async)
    ///
    /// Operations that involve I/O or can block:
    /// - File operations (read, write, stat) - kernel I/O
    /// - Network operations (socket, send, recv) - network latency
    /// - IPC operations (pipe, shm, queue) - inter-process coordination
    /// - Sleep operations - time-based blocking
    /// - Process spawn - subprocess creation
    #[inline]
    pub fn classify(&self) -> SyscallClass {
        match self {
            // ================================================================
            // Fast Path: In-memory operations
            // ================================================================

            // Memory management (DashMap lookups, atomic counters)
            Syscall::GetMemoryStats | Syscall::GetProcessMemoryStats { .. } => SyscallClass::Fast,

            // Process state queries (cached in ProcessManager)
            Syscall::GetProcessInfo { .. }
            | Syscall::GetProcessList
            | Syscall::GetProcessState { .. }
            | Syscall::GetProcessStats { .. } => SyscallClass::Fast,

            // System info (uptime calculation, cached data)
            Syscall::GetSystemInfo | Syscall::GetCurrentTime | Syscall::GetUptime => {
                SyscallClass::Fast
            }

            // Environment variables (HashMap lookup)
            Syscall::GetEnvironmentVar { .. } => SyscallClass::Fast,

            // File descriptor operations (in-memory registry)
            Syscall::Dup { .. } | Syscall::Dup2 { .. } | Syscall::Fcntl { .. } => {
                SyscallClass::Fast
            }

            // IPC stats (in-memory counter reads)
            Syscall::PipeStats { .. } | Syscall::ShmStats { .. } | Syscall::QueueStats { .. } => {
                SyscallClass::Fast
            }

            // Socket stats (in-memory counter reads)
            // GetSocketInfo removed - use network syscalls instead

            // Scheduler queries (cached state)
            Syscall::GetSchedulingPolicy
            | Syscall::GetSchedulerStats
            | Syscall::GetTimeQuantum
            | Syscall::GetProcessSchedulerStats { .. }
            | Syscall::GetAllProcessSchedulerStats => SyscallClass::Fast,

            // Working directory (cached per-process)
            Syscall::GetWorkingDirectory => SyscallClass::Fast,

            // ================================================================
            // Slow Path: Blocking I/O operations
            // ================================================================

            // File I/O (kernel syscalls, can block on slow storage)
            Syscall::ReadFile { .. }
            | Syscall::WriteFile { .. }
            | Syscall::CreateFile { .. }
            | Syscall::DeleteFile { .. }
            | Syscall::ListDirectory { .. }
            | Syscall::FileExists { .. }
            | Syscall::FileStat { .. }
            | Syscall::MoveFile { .. }
            | Syscall::CopyFile { .. }
            | Syscall::CreateDirectory { .. }
            | Syscall::RemoveDirectory { .. }
            | Syscall::TruncateFile { .. }
            | Syscall::Open { .. }
            | Syscall::Close { .. }
            | Syscall::Lseek { .. } => SyscallClass::Blocking,

            // Directory operations
            Syscall::SetWorkingDirectory { .. } => SyscallClass::Blocking,

            // Network operations (I/O, can block)
            Syscall::Socket { .. }
            | Syscall::Bind { .. }
            | Syscall::Listen { .. }
            | Syscall::Accept { .. }
            | Syscall::Connect { .. }
            | Syscall::Send { .. }
            | Syscall::Recv { .. }
            | Syscall::SendTo { .. }
            | Syscall::RecvFrom { .. }
            | Syscall::CloseSocket { .. }
            | Syscall::SetSockOpt { .. }
            | Syscall::GetSockOpt { .. }
            | Syscall::NetworkRequest { .. } => SyscallClass::Blocking,

            // IPC operations (can block on full/empty buffers)
            Syscall::CreatePipe { .. }
            | Syscall::WritePipe { .. }
            | Syscall::ReadPipe { .. }
            | Syscall::ClosePipe { .. }
            | Syscall::DestroyPipe { .. } => SyscallClass::Blocking,

            // Shared memory operations (potential page faults)
            Syscall::CreateShm { .. }
            | Syscall::AttachShm { .. }
            | Syscall::DetachShm { .. }
            | Syscall::WriteShm { .. }
            | Syscall::ReadShm { .. }
            | Syscall::DestroyShm { .. } => SyscallClass::Blocking,

            // Queue operations (can block on full/empty)
            Syscall::CreateQueue { .. }
            | Syscall::SendQueue { .. }
            | Syscall::ReceiveQueue { .. }
            | Syscall::SubscribeQueue { .. }
            | Syscall::UnsubscribeQueue { .. }
            | Syscall::CloseQueue { .. }
            | Syscall::DestroyQueue { .. } => SyscallClass::Blocking,

            // Process management (fork/exec are expensive)
            Syscall::SpawnProcess { .. }
            | Syscall::KillProcess { .. }
            | Syscall::WaitProcess { .. } => SyscallClass::Blocking,

            // Scheduler operations (can trigger context switch)
            Syscall::SetSchedulingPolicy { .. }
            | Syscall::SetTimeQuantum { .. }
            | Syscall::BoostPriority { .. }
            | Syscall::LowerPriority { .. }
            | Syscall::YieldProcess
            | Syscall::ScheduleNext
            | Syscall::GetCurrentScheduled => SyscallClass::Blocking,

            // Signal operations (delivery can trigger handlers)
            Syscall::SendSignal { .. }
            | Syscall::RegisterSignalHandler { .. }
            | Syscall::BlockSignal { .. }
            | Syscall::UnblockSignal { .. }
            | Syscall::GetPendingSignals
            | Syscall::GetSignalStats
            | Syscall::WaitForSignal { .. }
            | Syscall::GetSignalState { .. } => SyscallClass::Blocking,

            // Time operations (blocking by definition)
            Syscall::Sleep { .. } => SyscallClass::Blocking,

            // Memory management operations (potential GC)
            Syscall::TriggerGC { .. } => SyscallClass::Blocking,

            // Environment modification (can trigger side effects)
            Syscall::SetEnvironmentVar { .. } => SyscallClass::Blocking,

            // Mmap operations (page table modifications)
            Syscall::Mmap { .. }
            | Syscall::MmapRead { .. }
            | Syscall::MmapWrite { .. }
            | Syscall::Msync { .. }
            | Syscall::Munmap { .. }
            | Syscall::MmapStats { .. } => SyscallClass::Blocking,

            // Catch-all for future syscalls: default to blocking (safe)
            _ => SyscallClass::Blocking,
        }
    }

    /// Check if syscall is in the fast path (synchronous)
    #[inline(always)]
    pub fn is_fast(&self) -> bool {
        matches!(self.classify(), SyscallClass::Fast)
    }

    /// Check if syscall is in the slow path (async)
    #[inline(always)]
    pub fn is_blocking(&self) -> bool {
        matches!(self.classify(), SyscallClass::Blocking)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_fast_syscalls() {
        // Memory operations should be fast
        assert!(Syscall::GetMemoryStats.is_fast());
        assert!(Syscall::GetProcessMemoryStats { target_pid: 1 }.is_fast());

        // System info should be fast
        assert!(Syscall::GetSystemInfo.is_fast());
        assert!(Syscall::GetUptime.is_fast());
        assert!(Syscall::GetCurrentTime.is_fast());

        // Process queries should be fast
        assert!(Syscall::GetProcessList.is_fast());
    }

    #[test]
    fn test_blocking_syscalls() {
        // File I/O should be blocking
        assert!(Syscall::ReadFile {
            path: PathBuf::from("/test")
        }
        .is_blocking());
        assert!(Syscall::WriteFile {
            path: PathBuf::from("/test"),
            data: vec![]
        }
        .is_blocking());

        // Network should be blocking
        assert!(Syscall::Socket {
            domain: 0,
            socket_type: 0,
            protocol: 0
        }
        .is_blocking());

        // IPC should be blocking
        assert!(Syscall::CreatePipe {
            reader_pid: 1,
            writer_pid: 2,
            capacity: None
        }
        .is_blocking());

        // Sleep is obviously blocking
        assert!(Syscall::Sleep { duration_ms: 100 }.is_blocking());
    }

    #[test]
    fn test_classification_performance() {
        // Classification should be extremely fast (single match statement)
        let syscall = Syscall::GetMemoryStats;
        let start = std::time::Instant::now();
        for _ in 0..1_000_000 {
            let _ = syscall.classify();
        }
        let elapsed = start.elapsed();

        // Should take < 10ms for 1M classifications
        assert!(
            elapsed.as_millis() < 10,
            "Classification too slow: {:?}",
            elapsed
        );
    }
}
