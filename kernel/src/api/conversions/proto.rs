/*!
 * Protocol Buffer Conversion Utilities
 * Converts between protobuf messages and internal syscall types
 */

use std::path::PathBuf;
use crate::syscalls::Syscall;
use crate::api::server::grpc_server::kernel_proto::*;

/// Convert protobuf SyscallRequest to internal Syscall enum
/// Returns Err if syscall type is unsupported or missing
pub fn proto_to_syscall_full(req: &SyscallRequest) -> Result<Syscall, String> {
    match &req.syscall {
        Some(syscall_request::Syscall::ReadFile(call)) => Ok(Syscall::ReadFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::WriteFile(call)) => Ok(Syscall::WriteFile {
            path: PathBuf::from(call.path.clone()),
            data: call.data.clone(),
        }),
        Some(syscall_request::Syscall::CreateFile(call)) => Ok(Syscall::CreateFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::DeleteFile(call)) => Ok(Syscall::DeleteFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::ListDirectory(call)) => Ok(Syscall::ListDirectory {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::FileExists(call)) => Ok(Syscall::FileExists {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::FileStat(call)) => Ok(Syscall::FileStat {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::MoveFile(call)) => Ok(Syscall::MoveFile {
            source: PathBuf::from(call.source.clone()),
            destination: PathBuf::from(call.destination.clone()),
        }),
        Some(syscall_request::Syscall::CopyFile(call)) => Ok(Syscall::CopyFile {
            source: PathBuf::from(call.source.clone()),
            destination: PathBuf::from(call.destination.clone()),
        }),
        Some(syscall_request::Syscall::CreateDirectory(call)) => Ok(Syscall::CreateDirectory {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::RemoveDirectory(call)) => Ok(Syscall::RemoveDirectory {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::GetWorkingDirectory(_)) => Ok(Syscall::GetWorkingDirectory),
        Some(syscall_request::Syscall::SetWorkingDirectory(call)) => {
            Ok(Syscall::SetWorkingDirectory {
                path: PathBuf::from(call.path.clone()),
            })
        }
        Some(syscall_request::Syscall::TruncateFile(call)) => Ok(Syscall::TruncateFile {
            path: PathBuf::from(call.path.clone()),
            size: call.size,
        }),
        Some(syscall_request::Syscall::SpawnProcess(call)) => Ok(Syscall::SpawnProcess {
            command: call.command.clone(),
            args: call.args.clone(),
        }),
        Some(syscall_request::Syscall::KillProcess(call)) => Ok(Syscall::KillProcess {
            target_pid: call.target_pid,
        }),
        Some(syscall_request::Syscall::GetProcessInfo(call)) => Ok(Syscall::GetProcessInfo {
            target_pid: call.target_pid,
        }),
        Some(syscall_request::Syscall::GetProcessList(_)) => Ok(Syscall::GetProcessList),
        Some(syscall_request::Syscall::SetProcessPriority(call)) => {
            Ok(Syscall::SetProcessPriority {
                target_pid: call.target_pid,
                priority: call.priority as u8,
            })
        }
        Some(syscall_request::Syscall::GetProcessState(call)) => Ok(Syscall::GetProcessState {
            target_pid: call.target_pid,
        }),
        Some(syscall_request::Syscall::GetProcessStats(call)) => Ok(Syscall::GetProcessStats {
            target_pid: call.target_pid,
        }),
        Some(syscall_request::Syscall::WaitProcess(call)) => Ok(Syscall::WaitProcess {
            target_pid: call.target_pid,
            timeout_ms: call.timeout_ms,
        }),
        Some(syscall_request::Syscall::GetSystemInfo(_)) => Ok(Syscall::GetSystemInfo),
        Some(syscall_request::Syscall::GetCurrentTime(_)) => Ok(Syscall::GetCurrentTime),
        Some(syscall_request::Syscall::GetEnvVar(call)) => {
            Ok(Syscall::GetEnvironmentVar { key: call.key.clone() })
        }
        Some(syscall_request::Syscall::SetEnvVar(call)) => Ok(Syscall::SetEnvironmentVar {
            key: call.key.clone(),
            value: call.value.clone(),
        }),
        Some(syscall_request::Syscall::Sleep(call)) => Ok(Syscall::Sleep {
            duration_ms: call.duration_ms,
        }),
        Some(syscall_request::Syscall::GetUptime(_)) => Ok(Syscall::GetUptime),
        Some(syscall_request::Syscall::GetMemoryStats(_)) => Ok(Syscall::GetMemoryStats),
        Some(syscall_request::Syscall::GetProcessMemoryStats(call)) => {
            Ok(Syscall::GetProcessMemoryStats {
                target_pid: call.target_pid,
            })
        }
        Some(syscall_request::Syscall::TriggerGc(call)) => Ok(Syscall::TriggerGC {
            target_pid: call.target_pid,
        }),
        Some(syscall_request::Syscall::SendSignal(call)) => Ok(Syscall::SendSignal {
            target_pid: call.target_pid,
            signal: call.signal,
        }),
        Some(syscall_request::Syscall::NetworkRequest(call)) => {
            Ok(Syscall::NetworkRequest { url: call.url.clone() })
        }
        Some(syscall_request::Syscall::Socket(call)) => Ok(Syscall::Socket {
            domain: call.domain,
            socket_type: call.socket_type,
            protocol: call.protocol,
        }),
        Some(syscall_request::Syscall::Bind(call)) => Ok(Syscall::Bind {
            sockfd: call.sockfd,
            address: call.address.clone(),
        }),
        Some(syscall_request::Syscall::Listen(call)) => Ok(Syscall::Listen {
            sockfd: call.sockfd,
            backlog: call.backlog,
        }),
        Some(syscall_request::Syscall::Accept(call)) => Ok(Syscall::Accept {
            sockfd: call.sockfd,
        }),
        Some(syscall_request::Syscall::Connect(call)) => Ok(Syscall::Connect {
            sockfd: call.sockfd,
            address: call.address.clone(),
        }),
        Some(syscall_request::Syscall::Send(call)) => Ok(Syscall::Send {
            sockfd: call.sockfd,
            data: call.data.clone(),
            flags: call.flags,
        }),
        Some(syscall_request::Syscall::Recv(call)) => Ok(Syscall::Recv {
            sockfd: call.sockfd,
            size: call.size as usize,
            flags: call.flags,
        }),
        Some(syscall_request::Syscall::SendTo(call)) => Ok(Syscall::SendTo {
            sockfd: call.sockfd,
            data: call.data.clone(),
            address: call.address.clone(),
            flags: call.flags,
        }),
        Some(syscall_request::Syscall::RecvFrom(call)) => Ok(Syscall::RecvFrom {
            sockfd: call.sockfd,
            size: call.size as usize,
            flags: call.flags,
        }),
        Some(syscall_request::Syscall::CloseSocket(call)) => Ok(Syscall::CloseSocket {
            sockfd: call.sockfd,
        }),
        Some(syscall_request::Syscall::SetSockOpt(call)) => Ok(Syscall::SetSockOpt {
            sockfd: call.sockfd,
            level: call.level,
            optname: call.optname,
            optval: call.optval.clone(),
        }),
        Some(syscall_request::Syscall::GetSockOpt(call)) => Ok(Syscall::GetSockOpt {
            sockfd: call.sockfd,
            level: call.level,
            optname: call.optname,
        }),
        Some(syscall_request::Syscall::Open(call)) => Ok(Syscall::Open {
            path: PathBuf::from(call.path.clone()),
            flags: call.flags,
            mode: call.mode,
        }),
        Some(syscall_request::Syscall::Close(call)) => Ok(Syscall::Close { fd: call.fd }),
        Some(syscall_request::Syscall::Dup(call)) => Ok(Syscall::Dup { fd: call.fd }),
        Some(syscall_request::Syscall::Dup2(call)) => Ok(Syscall::Dup2 {
            oldfd: call.oldfd,
            newfd: call.newfd,
        }),
        Some(syscall_request::Syscall::Lseek(call)) => Ok(Syscall::Lseek {
            fd: call.fd,
            offset: call.offset,
            whence: call.whence,
        }),
        Some(syscall_request::Syscall::Fcntl(call)) => Ok(Syscall::Fcntl {
            fd: call.fd,
            cmd: call.cmd,
            arg: call.arg,
        }),
        Some(syscall_request::Syscall::CreatePipe(call)) => Ok(Syscall::CreatePipe {
            reader_pid: call.reader_pid,
            writer_pid: call.writer_pid,
            capacity: call.capacity.map(|c| c as usize),
        }),
        Some(syscall_request::Syscall::WritePipe(call)) => Ok(Syscall::WritePipe {
            pipe_id: call.pipe_id,
            data: call.data.clone(),
        }),
        Some(syscall_request::Syscall::ReadPipe(call)) => Ok(Syscall::ReadPipe {
            pipe_id: call.pipe_id,
            size: call.size as usize,
        }),
        Some(syscall_request::Syscall::ClosePipe(call)) => Ok(Syscall::ClosePipe {
            pipe_id: call.pipe_id,
        }),
        Some(syscall_request::Syscall::DestroyPipe(call)) => Ok(Syscall::DestroyPipe {
            pipe_id: call.pipe_id,
        }),
        Some(syscall_request::Syscall::PipeStats(call)) => Ok(Syscall::PipeStats {
            pipe_id: call.pipe_id,
        }),
        Some(syscall_request::Syscall::CreateShm(call)) => Ok(Syscall::CreateShm {
            size: call.size as usize,
        }),
        Some(syscall_request::Syscall::AttachShm(call)) => Ok(Syscall::AttachShm {
            segment_id: call.segment_id,
            read_only: call.read_only,
        }),
        Some(syscall_request::Syscall::DetachShm(call)) => Ok(Syscall::DetachShm {
            segment_id: call.segment_id,
        }),
        Some(syscall_request::Syscall::WriteShm(call)) => Ok(Syscall::WriteShm {
            segment_id: call.segment_id,
            offset: call.offset as usize,
            data: call.data.clone(),
        }),
        Some(syscall_request::Syscall::ReadShm(call)) => Ok(Syscall::ReadShm {
            segment_id: call.segment_id,
            offset: call.offset as usize,
            size: call.size as usize,
        }),
        Some(syscall_request::Syscall::DestroyShm(call)) => Ok(Syscall::DestroyShm {
            segment_id: call.segment_id,
        }),
        Some(syscall_request::Syscall::ShmStats(call)) => Ok(Syscall::ShmStats {
            segment_id: call.segment_id,
        }),
        Some(syscall_request::Syscall::Mmap(call)) => Ok(Syscall::Mmap {
            path: call.path.clone(),
            offset: call.offset as usize,
            length: call.length as usize,
            prot: call.prot as u8,
            shared: call.shared,
        }),
        Some(syscall_request::Syscall::MmapRead(call)) => Ok(Syscall::MmapRead {
            mmap_id: call.mmap_id,
            offset: call.offset as usize,
            length: call.length as usize,
        }),
        Some(syscall_request::Syscall::MmapWrite(call)) => Ok(Syscall::MmapWrite {
            mmap_id: call.mmap_id,
            offset: call.offset as usize,
            data: call.data.clone(),
        }),
        Some(syscall_request::Syscall::Msync(call)) => Ok(Syscall::Msync {
            mmap_id: call.mmap_id,
        }),
        Some(syscall_request::Syscall::Munmap(call)) => Ok(Syscall::Munmap {
            mmap_id: call.mmap_id,
        }),
        Some(syscall_request::Syscall::MmapStats(call)) => Ok(Syscall::MmapStats {
            mmap_id: call.mmap_id,
        }),
        Some(syscall_request::Syscall::CreateQueue(call)) => Ok(Syscall::CreateQueue {
            queue_type: call.queue_type.clone(),
            capacity: call.capacity.map(|c| c as usize),
        }),
        Some(syscall_request::Syscall::SendQueue(call)) => Ok(Syscall::SendQueue {
            queue_id: call.queue_id,
            data: call.data.clone(),
            priority: call.priority.map(|p| p as u8),
        }),
        Some(syscall_request::Syscall::ReceiveQueue(call)) => Ok(Syscall::ReceiveQueue {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::SubscribeQueue(call)) => Ok(Syscall::SubscribeQueue {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::UnsubscribeQueue(call)) => Ok(Syscall::UnsubscribeQueue {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::CloseQueue(call)) => Ok(Syscall::CloseQueue {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::DestroyQueue(call)) => Ok(Syscall::DestroyQueue {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::QueueStats(call)) => Ok(Syscall::QueueStats {
            queue_id: call.queue_id,
        }),
        Some(syscall_request::Syscall::ScheduleNext(_)) => Ok(Syscall::ScheduleNext),
        Some(syscall_request::Syscall::YieldProcess(_)) => Ok(Syscall::YieldProcess),
        Some(syscall_request::Syscall::GetCurrentScheduled(_)) => Ok(Syscall::GetCurrentScheduled),
        Some(syscall_request::Syscall::GetSchedulerStats(_)) => Ok(Syscall::GetSchedulerStats),
        Some(syscall_request::Syscall::SetSchedulingPolicy(call)) => {
            Ok(Syscall::SetSchedulingPolicy {
                policy: call.policy.clone(),
            })
        }
        Some(syscall_request::Syscall::GetSchedulingPolicy(_)) => Ok(Syscall::GetSchedulingPolicy),
        Some(syscall_request::Syscall::SetTimeQuantum(call)) => Ok(Syscall::SetTimeQuantum {
            quantum_micros: call.quantum_micros,
        }),
        Some(syscall_request::Syscall::GetTimeQuantum(_)) => Ok(Syscall::GetTimeQuantum),
        Some(syscall_request::Syscall::GetProcessSchedulerStats(_)) => {
            Err("GetProcessSchedulerStats not yet implemented".to_string())
        }
        Some(syscall_request::Syscall::GetAllProcessSchedulerStats(_)) => {
            Err("GetAllProcessSchedulerStats not yet implemented".to_string())
        }
        Some(syscall_request::Syscall::BoostPriority(_)) => {
            Err("BoostPriority not yet implemented".to_string())
        }
        Some(syscall_request::Syscall::LowerPriority(_)) => {
            Err("LowerPriority not yet implemented".to_string())
        }
        None => Err("No syscall provided".to_string()),
    }
}

/// Simplified conversion for async/batch operations (supports subset of syscalls)
pub fn proto_to_syscall_simple(req: &SyscallRequest) -> Result<Syscall, String> {
    match &req.syscall {
        Some(syscall_request::Syscall::ReadFile(call)) => Ok(Syscall::ReadFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::WriteFile(call)) => Ok(Syscall::WriteFile {
            path: PathBuf::from(call.path.clone()),
            data: call.data.clone(),
        }),
        Some(syscall_request::Syscall::CreateFile(call)) => Ok(Syscall::CreateFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::DeleteFile(call)) => Ok(Syscall::DeleteFile {
            path: PathBuf::from(call.path.clone()),
        }),
        Some(syscall_request::Syscall::SpawnProcess(call)) => Ok(Syscall::SpawnProcess {
            command: call.command.clone(),
            args: call.args.clone(),
        }),
        Some(syscall_request::Syscall::Sleep(call)) => Ok(Syscall::Sleep {
            duration_ms: call.duration_ms,
        }),
        _ => Err("Unsupported syscall for async/batch".to_string()),
    }
}
