/*!
 * Syscall Type Extensions
 * Additional functionality for syscall types including tracing support
 */

use super::types::Syscall;

impl Syscall {
    /// Get the name of this syscall for tracing and logging
    pub fn name(&self) -> &'static str {
        match self {
            // File System Operations
            Syscall::ReadFile { .. } => "read_file",
            Syscall::WriteFile { .. } => "write_file",
            Syscall::CreateFile { .. } => "create_file",
            Syscall::DeleteFile { .. } => "delete_file",
            Syscall::ListDirectory { .. } => "list_directory",
            Syscall::FileExists { .. } => "file_exists",
            Syscall::FileStat { .. } => "file_stat",
            Syscall::MoveFile { .. } => "move_file",
            Syscall::CopyFile { .. } => "copy_file",
            Syscall::CreateDirectory { .. } => "create_directory",

            // File Descriptor Operations
            Syscall::Open { .. } => "open",
            Syscall::Close { .. } => "close",
            Syscall::Lseek { .. } => "lseek",
            Syscall::Dup { .. } => "dup",
            Syscall::Dup2 { .. } => "dup2",
            Syscall::Fcntl { .. } => "fcntl",

            // IPC - Pipes
            Syscall::CreatePipe { .. } => "create_pipe",
            Syscall::ReadPipe { .. } => "read_pipe",
            Syscall::WritePipe { .. } => "write_pipe",
            Syscall::ClosePipe { .. } => "close_pipe",
            Syscall::DestroyPipe { .. } => "destroy_pipe",
            Syscall::PipeStats { .. } => "pipe_stats",

            // IPC - Shared Memory
            Syscall::CreateShm { .. } => "create_shm",
            Syscall::AttachShm { .. } => "attach_shm",
            Syscall::DetachShm { .. } => "detach_shm",
            Syscall::ReadShm { .. } => "read_shm",
            Syscall::WriteShm { .. } => "write_shm",
            Syscall::DestroyShm { .. } => "destroy_shm",
            Syscall::ShmStats { .. } => "shm_stats",

            // IPC - Memory-Mapped Files
            Syscall::Mmap { .. } => "mmap",
            Syscall::MmapRead { .. } => "mmap_read",
            Syscall::MmapWrite { .. } => "mmap_write",
            Syscall::Msync { .. } => "msync",
            Syscall::Munmap { .. } => "munmap",
            Syscall::MmapStats { .. } => "mmap_stats",

            // IPC - Async Queues
            Syscall::CreateQueue { .. } => "create_queue",
            Syscall::SendQueue { .. } => "send_queue",
            Syscall::ReceiveQueue { .. } => "receive_queue",
            Syscall::SubscribeQueue { .. } => "subscribe_queue",
            Syscall::UnsubscribeQueue { .. } => "unsubscribe_queue",
            Syscall::CloseQueue { .. } => "close_queue",
            Syscall::DestroyQueue { .. } => "destroy_queue",
            Syscall::QueueStats { .. } => "queue_stats",

            // Scheduler Operations
            Syscall::ScheduleNext => "schedule_next",
            Syscall::YieldProcess => "yield_process",
            Syscall::GetProcessSchedulerStats { .. } => "get_process_scheduler_stats",
            Syscall::GetAllProcessSchedulerStats => "get_all_process_scheduler_stats",
            Syscall::BoostPriority { .. } => "boost_priority",
            Syscall::LowerPriority { .. } => "lower_priority",

            // Signal Operations
            Syscall::SendSignal { .. } => "send_signal",

            // Process Operations
            Syscall::SpawnProcess { .. } => "spawn_process",
            Syscall::GetProcessInfo { .. } => "get_process_info",

            // Memory Operations
            Syscall::GetMemoryStats => "get_memory_stats",
            Syscall::GetProcessMemoryStats { .. } => "get_process_memory_stats",
            Syscall::TriggerGC { .. } => "trigger_gc",

            // System Info Operations
            Syscall::GetSystemInfo => "get_system_info",

            // Network Operations
            Syscall::Socket { .. } => "socket",
            Syscall::Bind { .. } => "bind",
            Syscall::Connect { .. } => "connect",
            Syscall::Listen { .. } => "listen",
            Syscall::Accept { .. } => "accept",
            Syscall::Send { .. } => "send",
            Syscall::Recv { .. } => "recv",
            Syscall::SendTo { .. } => "sendto",
            Syscall::RecvFrom { .. } => "recvfrom",
            Syscall::CloseSocket { .. } => "close_socket",
            Syscall::GetSockOpt { .. } => "getsockopt",
            Syscall::SetSockOpt { .. } => "setsockopt",

            // Time Operations
            Syscall::Sleep { .. } => "sleep",
            Syscall::GetUptime => "get_uptime",

            // Additional syscalls (catch-all for extended syscalls)
            _ => "syscall",
        }
    }
}
