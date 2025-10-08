/*!
 * Network Syscall Handler
 * Handles network and socket syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for network syscalls
pub struct NetworkHandler {
    executor: SyscallExecutorWithIpc,
}

impl NetworkHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for NetworkHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::NetworkRequest { ref url } => Some(self.executor.network_request(pid, url).into()),
            Syscall::Socket {
                domain,
                socket_type,
                protocol,
            } => Some(self.executor.socket(pid, *domain, *socket_type, *protocol).into()),
            Syscall::Bind {
                sockfd,
                ref address,
            } => Some(self.executor.bind(pid, *sockfd, address).into()),
            Syscall::Listen { sockfd, backlog } => {
                Some(self.executor.listen(pid, *sockfd, *backlog))
            }
            Syscall::Accept { sockfd } => Some(self.executor.accept(pid, *sockfd).into()),
            Syscall::Connect {
                sockfd,
                ref address,
            } => Some(self.executor.connect(pid, *sockfd, address).into()),
            Syscall::Send {
                sockfd,
                ref data,
                flags,
            } => Some(self.executor.send(pid, *sockfd, data, *flags).into()),
            Syscall::Recv {
                sockfd,
                size,
                flags,
            } => Some(self.executor.recv(pid, *sockfd, *size, *flags).into()),
            Syscall::SendTo {
                sockfd,
                ref data,
                ref address,
                flags,
            } => Some(self.executor.sendto(pid, *sockfd, data, address, *flags).into()),
            Syscall::RecvFrom {
                sockfd,
                size,
                flags,
            } => Some(self.executor.recvfrom(pid, *sockfd, *size, *flags).into()),
            Syscall::CloseSocket { sockfd } => Some(self.executor.close_socket(pid, *sockfd).into()),
            Syscall::SetSockOpt {
                sockfd,
                level,
                optname,
                ref optval,
            } => Some(
                self.executor
                    .setsockopt(pid, *sockfd, *level, *optname, optval),
            ),
            Syscall::GetSockOpt {
                sockfd,
                level,
                optname,
            } => Some(self.executor.getsockopt(pid, *sockfd, *level, *optname).into()),
            _ => None, // Not a network syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "network_handler"
    }
}
