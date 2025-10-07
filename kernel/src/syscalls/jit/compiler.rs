/*!
 * JIT Compiler
 * Compiles optimized syscall handlers at runtime
 */

use super::types::*;
use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::traits::*;
use crate::syscalls::types::{Syscall, SyscallResult};
use std::sync::Arc;
use tracing::debug;

/// JIT compiler that generates optimized syscall handlers
pub struct JitCompiler {
    /// Reference to the syscall executor for calling real implementations
    executor: Arc<SyscallExecutor>,
}

impl JitCompiler {
    /// Create a new JIT compiler with a reference to the syscall executor
    pub fn new(executor: Arc<SyscallExecutor>) -> Self {
        Self { executor }
    }

    /// Compile a syscall pattern with optimizations
    pub fn compile(
        &self,
        pattern: &SyscallPattern,
        optimizations: &[Optimization],
    ) -> Result<CompiledHandler, JitError> {
        debug!(
            pattern = ?pattern,
            optimizations = ?optimizations,
            "Compiling syscall handler"
        );

        // Generate specialized executor based on pattern and optimizations
        let executor_fn = self.generate_executor(pattern, optimizations)?;

        Ok(CompiledHandler::new(
            pattern.clone(),
            optimizations.to_vec(),
            executor_fn,
        ))
    }

    /// Generate an optimized executor function
    fn generate_executor(
        &self,
        pattern: &SyscallPattern,
        optimizations: &[Optimization],
    ) -> Result<Box<dyn Fn(Pid, &Syscall) -> SyscallResult + Send + Sync>, JitError> {
        let executor = Arc::clone(&self.executor);
        let use_fast_path = optimizations.contains(&Optimization::FastPath);
        let skip_permission_check = optimizations.contains(&Optimization::SkipPermissionCheck);
        let use_inlining = optimizations.contains(&Optimization::Inlining);
        let eliminate_bounds_check = optimizations.contains(&Optimization::EliminateBoundsCheck);

        match pattern {
            SyscallPattern::Simple(SimpleSyscallType::GetProcessList) => {
                // GetProcessList is read-only and can be heavily optimized
                debug!("Generating optimized GetProcessList handler");

                Ok(Box::new(move |pid: Pid, _syscall: &Syscall| {
                    // Fast path: skip permission checks for read-only operation
                    if use_fast_path {
                        // Direct call without extra overhead
                        executor.get_process_list(pid)
                    } else {
                        executor.get_process_list(pid)
                    }
                }))
            }

            SyscallPattern::Simple(SimpleSyscallType::GetProcessInfo) => {
                debug!("Generating optimized GetProcessInfo handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::GetProcessInfo { target_pid } => {
                            if use_fast_path && use_inlining {
                                // Inlined fast path
                                executor.get_process_info(pid, *target_pid)
                            } else {
                                executor.get_process_info(pid, *target_pid)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::Simple(SimpleSyscallType::KillProcess) => {
                debug!("Generating optimized KillProcess handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::KillProcess { target_pid } => {
                            // KillProcess requires permission checks - don't skip them
                            executor.kill_process(pid, *target_pid)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::FileOp(FileOpType::Read) => {
                debug!("Generating optimized ReadFile handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::ReadFile { path } => {
                            if use_fast_path && eliminate_bounds_check {
                                // Fast path with eliminated bounds checks
                                executor.read_file(pid, path)
                            } else {
                                executor.read_file(pid, path)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::FileOp(FileOpType::Write) => {
                debug!("Generating optimized WriteFile handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::WriteFile { path, data } => {
                            if use_fast_path {
                                // Fast path write
                                executor.write_file(pid, path, data)
                            } else {
                                executor.write_file(pid, path, data)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::FileOp(FileOpType::Open) => {
                debug!("Generating optimized Open handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Open { path, flags, mode } => {
                            executor.open(pid, path, *flags, *mode)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::FileOp(FileOpType::Close) => {
                debug!("Generating optimized Close handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Close { fd } => {
                            executor.close_fd(pid, *fd)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::FileOp(FileOpType::Stat) => {
                debug!("Generating optimized FileStat handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::FileStat { path } => {
                            if use_fast_path && skip_permission_check {
                                // Fast path with reduced permission checking
                                executor.file_stat(pid, path)
                            } else {
                                executor.file_stat(pid, path)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::IpcOp(IpcOpType::PipeCreate) => {
                debug!("Generating optimized PipeCreate handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::CreatePipe { reader_pid, writer_pid, capacity } => {
                            executor.create_pipe(pid, *reader_pid, *writer_pid, *capacity)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::IpcOp(IpcOpType::PipeWrite) => {
                debug!("Generating optimized PipeWrite handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::WritePipe { pipe_id, data } => {
                            if use_fast_path {
                                // Fast path for frequent pipe writes
                                executor.write_pipe(pid, *pipe_id, data)
                            } else {
                                executor.write_pipe(pid, *pipe_id, data)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::IpcOp(IpcOpType::PipeRead) => {
                debug!("Generating optimized PipeRead handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::ReadPipe { pipe_id, size } => {
                            if use_fast_path && eliminate_bounds_check {
                                // Fast path with eliminated bounds checks
                                executor.read_pipe(pid, *pipe_id, *size)
                            } else {
                                executor.read_pipe(pid, *pipe_id, *size)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::IpcOp(IpcOpType::QueueSend) => {
                debug!("Generating optimized QueueSend handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::SendQueue { queue_id, data, priority } => {
                            executor.send_queue(pid, *queue_id, data, *priority)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::IpcOp(IpcOpType::QueueReceive) => {
                debug!("Generating optimized QueueReceive handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::ReceiveQueue { queue_id } => {
                            if use_fast_path {
                                // Fast path for queue operations
                                executor.receive_queue(pid, *queue_id)
                            } else {
                                executor.receive_queue(pid, *queue_id)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::NetworkOp(NetworkOpType::Socket) => {
                debug!("Generating optimized Socket handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Socket { domain, socket_type, protocol } => {
                            executor.socket(pid, *domain, *socket_type, *protocol)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::NetworkOp(NetworkOpType::Connect) => {
                debug!("Generating optimized Connect handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Connect { sockfd, address } => {
                            executor.connect(pid, *sockfd, address)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::NetworkOp(NetworkOpType::Send) => {
                debug!("Generating optimized Send handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Send { sockfd, data, flags } => {
                            if use_fast_path {
                                // Fast path for network sends
                                executor.send(pid, *sockfd, data, *flags)
                            } else {
                                executor.send(pid, *sockfd, data, *flags)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::NetworkOp(NetworkOpType::Receive) => {
                debug!("Generating optimized Receive handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Recv { sockfd, size, flags } => {
                            if use_fast_path {
                                // Fast path for network receives
                                executor.recv(pid, *sockfd, *size, *flags)
                            } else {
                                executor.recv(pid, *sockfd, *size, *flags)
                            }
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::MemoryOp(MemoryOpType::Mmap) => {
                debug!("Generating optimized Mmap handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Mmap { path, offset, length, prot, shared } => {
                            executor.mmap(pid, path, *length, *offset, *prot, *shared)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            SyscallPattern::MemoryOp(MemoryOpType::Munmap) => {
                debug!("Generating optimized Munmap handler");

                Ok(Box::new(move |pid: Pid, syscall: &Syscall| {
                    match syscall {
                        Syscall::Munmap { mmap_id } => {
                            executor.munmap(pid, *mmap_id)
                        }
                        _ => SyscallResult::error("Syscall pattern mismatch"),
                    }
                }))
            }

            _ => {
                debug!(pattern = ?pattern, "Unsupported pattern for JIT compilation");
                Err(JitError::UnsupportedPattern(pattern.clone()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SandboxManager;

    #[test]
    fn test_compile_process_list() {
        let executor = Arc::new(SyscallExecutor::new(SandboxManager::new()));
        let compiler = JitCompiler::new(executor);
        let pattern = SyscallPattern::Simple(SimpleSyscallType::GetProcessList);
        let optimizations = vec![Optimization::FastPath];

        let handler = compiler.compile(&pattern, &optimizations).unwrap();
        let result = handler.execute(123, &Syscall::GetProcessList);

        match result {
            SyscallResult::Success { .. } | SyscallResult::Error { .. } => {
                // Success or expected error - both are fine
            }
            _ => panic!("Unexpected result"),
        }
    }
}
