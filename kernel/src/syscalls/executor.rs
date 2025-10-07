/*!
 * Syscall Executor
 * Central executor for all syscalls with sandboxing
 */

use crate::core::types::Pid;
use crate::monitoring::{span_syscall, MetricsCollector};
use crate::security::SandboxManager;
use log::info;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use super::types::{Syscall, SyscallResult};

/// Global system start time for uptime tracking
pub static SYSTEM_START: OnceLock<Instant> = OnceLock::new();

/// System call executor
#[derive(Clone)]
pub struct SyscallExecutor {
    pub(super) sandbox_manager: SandboxManager,
    pub(super) pipe_manager: Option<crate::ipc::PipeManager>,
    pub(super) shm_manager: Option<crate::ipc::ShmManager>,
    pub(super) queue_manager: Option<crate::ipc::QueueManager>,
    pub(super) process_manager: Option<crate::process::ProcessManagerImpl>,
    pub(super) memory_manager: Option<crate::memory::MemoryManager>,
    pub(super) signal_manager: Option<crate::signals::SignalManagerImpl>,
    pub(super) vfs: Option<crate::vfs::MountManager>,
    pub(super) metrics: Option<Arc<MetricsCollector>>,
    pub(super) fd_manager: super::fd::FdManager,
    pub(super) socket_manager: super::network::SocketManager,
}

impl SyscallExecutor {
    pub fn new(sandbox_manager: SandboxManager) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        info!("Syscall executor initialized");
        Self {
            sandbox_manager,
            pipe_manager: None,
            shm_manager: None,
            queue_manager: None,
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_ipc(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
    ) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        info!("Syscall executor initialized with IPC support");
        Self {
            sandbox_manager,
            pipe_manager: Some(pipe_manager),
            shm_manager: Some(shm_manager),
            queue_manager: None,
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
        }
    }

    pub fn with_queues(mut self, queue_manager: crate::ipc::QueueManager) -> Self {
        self.queue_manager = Some(queue_manager);
        info!("Queue support enabled for syscall executor");
        self
    }

    /// Add signal manager support
    pub fn with_signals(mut self, signal_manager: crate::signals::SignalManagerImpl) -> Self {
        self.signal_manager = Some(signal_manager);
        info!("Signal support enabled for syscall executor");
        self
    }

    pub fn with_full_features(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
        process_manager: crate::process::ProcessManagerImpl,
        memory_manager: crate::memory::MemoryManager,
    ) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        info!("Syscall executor initialized with full features");
        Self {
            sandbox_manager,
            pipe_manager: Some(pipe_manager),
            shm_manager: Some(shm_manager),
            queue_manager: None,
            process_manager: Some(process_manager),
            memory_manager: Some(memory_manager),
            signal_manager: None,
            vfs: None,
            metrics: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
        }
    }

    /// Set VFS mount manager
    pub fn with_vfs(mut self, vfs: crate::vfs::MountManager) -> Self {
        self.vfs = Some(vfs);
        info!("VFS enabled for syscall executor");
        self
    }

    /// Execute a system call with sandboxing
    pub fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        info!("Executing syscall for PID {}: {:?}", pid, syscall);

        match syscall {
            // File operations
            Syscall::ReadFile { ref path } => self.read_file(pid, path),
            Syscall::WriteFile { ref path, ref data } => self.write_file(pid, path, data),
            Syscall::CreateFile { ref path } => self.create_file(pid, path),
            Syscall::DeleteFile { ref path } => self.delete_file(pid, path),
            Syscall::ListDirectory { ref path } => self.list_directory(pid, path),
            Syscall::FileExists { ref path } => self.file_exists(pid, path),
            Syscall::FileStat { ref path } => self.file_stat(pid, path),
            Syscall::MoveFile {
                ref source,
                ref destination,
            } => self.move_file(pid, source, destination),
            Syscall::CopyFile {
                ref source,
                ref destination,
            } => self.copy_file(pid, source, destination),
            Syscall::CreateDirectory { ref path } => self.create_directory(pid, path),
            Syscall::RemoveDirectory { ref path } => self.remove_directory(pid, path),
            Syscall::GetWorkingDirectory => self.get_working_directory(pid),
            Syscall::SetWorkingDirectory { ref path } => self.set_working_directory(pid, path),
            Syscall::TruncateFile { ref path, size } => self.truncate_file(pid, path, size),

            // Process operations
            Syscall::SpawnProcess {
                ref command,
                ref args,
            } => self.spawn_process(pid, command, args),
            Syscall::KillProcess { target_pid } => self.kill_process(pid, target_pid),
            Syscall::GetProcessInfo { target_pid } => self.get_process_info(pid, target_pid),
            Syscall::GetProcessList => self.get_process_list(pid),
            Syscall::SetProcessPriority {
                target_pid,
                priority,
            } => self.set_process_priority(pid, target_pid, priority),
            Syscall::GetProcessState { target_pid } => self.get_process_state(pid, target_pid),
            Syscall::GetProcessStats { target_pid } => self.get_process_stats_call(pid, target_pid),
            Syscall::WaitProcess {
                target_pid,
                timeout_ms,
            } => self.wait_process(pid, target_pid, timeout_ms),

            // System info
            Syscall::GetSystemInfo => self.get_system_info(pid),
            Syscall::GetCurrentTime => self.get_current_time(pid),
            Syscall::GetEnvironmentVar { ref key } => self.get_env_var(pid, key),
            Syscall::SetEnvironmentVar { ref key, ref value } => self.set_env_var(pid, key, value),

            // Network
            Syscall::NetworkRequest { ref url } => self.network_request(pid, url),

            // IPC - Pipes
            Syscall::CreatePipe {
                reader_pid,
                writer_pid,
                capacity,
            } => self.create_pipe(pid, reader_pid, writer_pid, capacity),
            Syscall::WritePipe { pipe_id, ref data } => self.write_pipe(pid, pipe_id, data),
            Syscall::ReadPipe { pipe_id, size } => self.read_pipe(pid, pipe_id, size),
            Syscall::ClosePipe { pipe_id } => self.close_pipe(pid, pipe_id),
            Syscall::DestroyPipe { pipe_id } => self.destroy_pipe(pid, pipe_id),
            Syscall::PipeStats { pipe_id } => self.pipe_stats(pid, pipe_id),

            // IPC - Shared Memory
            Syscall::CreateShm { size } => self.create_shm(pid, size),
            Syscall::AttachShm {
                segment_id,
                read_only,
            } => self.attach_shm(pid, segment_id, read_only),
            Syscall::DetachShm { segment_id } => self.detach_shm(pid, segment_id),
            Syscall::WriteShm {
                segment_id,
                offset,
                ref data,
            } => self.write_shm(pid, segment_id, offset, data),
            Syscall::ReadShm {
                segment_id,
                offset,
                size,
            } => self.read_shm(pid, segment_id, offset, size),
            Syscall::DestroyShm { segment_id } => self.destroy_shm(pid, segment_id),
            Syscall::ShmStats { segment_id } => self.shm_stats(pid, segment_id),

            // IPC - Async Queues
            Syscall::CreateQueue {
                ref queue_type,
                capacity,
            } => self.create_queue(pid, queue_type, capacity),
            Syscall::SendQueue {
                queue_id,
                ref data,
                priority,
            } => self.send_queue(pid, queue_id, data, priority),
            Syscall::ReceiveQueue { queue_id } => self.receive_queue(pid, queue_id),
            Syscall::SubscribeQueue { queue_id } => self.subscribe_queue(pid, queue_id),
            Syscall::UnsubscribeQueue { queue_id } => self.unsubscribe_queue(pid, queue_id),
            Syscall::CloseQueue { queue_id } => self.close_queue(pid, queue_id),
            Syscall::DestroyQueue { queue_id } => self.destroy_queue(pid, queue_id),
            Syscall::QueueStats { queue_id } => self.queue_stats(pid, queue_id),

            // Scheduler operations
            Syscall::ScheduleNext => self.schedule_next(pid),
            Syscall::YieldProcess => self.yield_process(pid),
            Syscall::GetCurrentScheduled => self.get_current_scheduled(pid),
            Syscall::GetSchedulerStats => self.get_scheduler_stats(pid),
            Syscall::SetSchedulingPolicy { ref policy } => self.set_scheduling_policy(pid, policy),
            Syscall::GetSchedulingPolicy => self.get_scheduling_policy(pid),
            Syscall::SetTimeQuantum { quantum_micros } => self.set_time_quantum(pid, quantum_micros),
            Syscall::GetTimeQuantum => self.get_time_quantum(pid),
            Syscall::GetProcessSchedulerStats { target_pid } => {
                self.get_process_scheduler_stats(pid, target_pid)
            }
            Syscall::GetAllProcessSchedulerStats => self.get_all_process_scheduler_stats(pid),
            Syscall::BoostPriority { target_pid } => self.boost_priority(pid, target_pid),
            Syscall::LowerPriority { target_pid } => self.lower_priority(pid, target_pid),

            // Time operations
            Syscall::Sleep { duration_ms } => self.sleep(pid, duration_ms),
            Syscall::GetUptime => self.get_uptime(pid),

            // Memory operations
            Syscall::GetMemoryStats => self.get_memory_stats(pid),
            Syscall::GetProcessMemoryStats { target_pid } => {
                self.get_process_memory_stats(pid, target_pid)
            }
            Syscall::TriggerGC { target_pid } => self.trigger_gc(pid, target_pid),

            // Signal operations
            Syscall::SendSignal { target_pid, signal } => self.send_signal(pid, target_pid, signal),

            // Network operations
            Syscall::Socket {
                domain,
                socket_type,
                protocol,
            } => self.socket(pid, domain, socket_type, protocol),
            Syscall::Bind {
                sockfd,
                ref address,
            } => self.bind(pid, sockfd, address),
            Syscall::Listen { sockfd, backlog } => self.listen(pid, sockfd, backlog),
            Syscall::Accept { sockfd } => self.accept(pid, sockfd),
            Syscall::Connect {
                sockfd,
                ref address,
            } => self.connect(pid, sockfd, address),
            Syscall::Send {
                sockfd,
                ref data,
                flags,
            } => self.send(pid, sockfd, data, flags),
            Syscall::Recv {
                sockfd,
                size,
                flags,
            } => self.recv(pid, sockfd, size, flags),
            Syscall::SendTo {
                sockfd,
                ref data,
                ref address,
                flags,
            } => self.sendto(pid, sockfd, data, address, flags),
            Syscall::RecvFrom {
                sockfd,
                size,
                flags,
            } => self.recvfrom(pid, sockfd, size, flags),
            Syscall::CloseSocket { sockfd } => self.close_socket(pid, sockfd),
            Syscall::SetSockOpt {
                sockfd,
                level,
                optname,
                ref optval,
            } => self.setsockopt(pid, sockfd, level, optname, optval),
            Syscall::GetSockOpt {
                sockfd,
                level,
                optname,
            } => self.getsockopt(pid, sockfd, level, optname),

            // File Descriptor operations
            Syscall::Open {
                ref path,
                flags,
                mode,
            } => self.open(pid, path, flags, mode),
            Syscall::Close { fd } => self.close_fd(pid, fd),
            Syscall::Dup { fd } => self.dup(pid, fd),
            Syscall::Dup2 { oldfd, newfd } => self.dup2(pid, oldfd, newfd),
            Syscall::Lseek { fd, offset, whence } => self.lseek(pid, fd, offset, whence),
            Syscall::Fcntl { fd, cmd, arg } => self.fcntl(pid, fd, cmd, arg),
        }
    }
}
