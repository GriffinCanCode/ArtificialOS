/*!

 * Network Syscalls
 * Socket operations for TCP/UDP networking
 */

use crate::core::types::Pid;

use log::{info, warn};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// Socket manager for tracking open sockets
pub struct SocketManager {
    next_fd: Arc<RwLock<u32>>,
    tcp_listeners: Arc<RwLock<HashMap<u32, TcpListener>>>,
    tcp_streams: Arc<RwLock<HashMap<u32, TcpStream>>>,
    udp_sockets: Arc<RwLock<HashMap<u32, UdpSocket>>>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            next_fd: Arc::new(RwLock::new(1000)), // Start socket FDs at 1000
            tcp_listeners: Arc::new(RwLock::new(HashMap::new())),
            tcp_streams: Arc::new(RwLock::new(HashMap::new())),
            udp_sockets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn allocate_fd(&self) -> u32 {
        let mut next = self.next_fd.write().unwrap();
        let fd = *next;
        *next += 1;
        fd
    }
}

impl Clone for SocketManager {
    fn clone(&self) -> Self {
        Self {
            next_fd: Arc::clone(&self.next_fd),
            tcp_listeners: Arc::clone(&self.tcp_listeners),
            tcp_streams: Arc::clone(&self.tcp_streams),
            udp_sockets: Arc::clone(&self.udp_sockets),
        }
    }
}

impl SyscallExecutor {
    pub(super) fn socket(&self, pid: Pid, domain: u32, socket_type: u32, protocol: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Placeholder implementation - would create actual socket
        warn!("Socket syscall not fully implemented: domain={}, type={}, protocol={}", domain, socket_type, protocol);

        // Return mock socket FD
        let sockfd = 1000 + pid;
        info!("PID {} created socket FD {}", pid, sockfd);

        let data = serde_json::to_vec(&serde_json::json!({
            "sockfd": sockfd,
            "domain": domain,
            "type": socket_type,
            "protocol": protocol
        })).unwrap();

        SyscallResult::success_with_data(data)
    }

    pub(super) fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::BindPort)
        {
            return SyscallResult::permission_denied("Missing BindPort capability");
        }

        warn!("Bind syscall not fully implemented: sockfd={}, address={}", sockfd, address);
        info!("PID {} bound socket {} to {}", pid, sockfd, address);
        SyscallResult::success()
    }

    pub(super) fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::BindPort)
        {
            return SyscallResult::permission_denied("Missing BindPort capability");
        }

        warn!("Listen syscall not fully implemented: sockfd={}, backlog={}", sockfd, backlog);
        info!("PID {} listening on socket {} with backlog {}", pid, sockfd, backlog);
        SyscallResult::success()
    }

    pub(super) fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("Accept syscall not fully implemented: sockfd={}", sockfd);

        // Return mock client FD
        let client_fd = sockfd + 1;
        info!("PID {} accepted connection on socket {}, client FD {}", pid, sockfd, client_fd);

        let data = serde_json::to_vec(&serde_json::json!({
            "client_fd": client_fd,
            "address": "127.0.0.1:0"
        })).unwrap();

        SyscallResult::success_with_data(data)
    }

    pub(super) fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("Connect syscall not fully implemented: sockfd={}, address={}", sockfd, address);
        info!("PID {} connected socket {} to {}", pid, sockfd, address);
        SyscallResult::success()
    }

    pub(super) fn send(&self, pid: Pid, sockfd: u32, data: &[u8], flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("Send syscall not fully implemented: sockfd={}, size={}, flags={}", sockfd, data.len(), flags);
        info!("PID {} sent {} bytes on socket {}", pid, data.len(), sockfd);

        let result = serde_json::to_vec(&serde_json::json!({
            "bytes_sent": data.len()
        })).unwrap();

        SyscallResult::success_with_data(result)
    }

    pub(super) fn recv(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("Recv syscall not fully implemented: sockfd={}, size={}, flags={}", sockfd, size, flags);

        // Return empty data for now
        info!("PID {} received 0 bytes on socket {}", pid, sockfd);
        SyscallResult::success_with_data(vec![])
    }

    pub(super) fn sendto(&self, pid: Pid, sockfd: u32, data: &[u8], address: &str, flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("SendTo syscall not fully implemented: sockfd={}, address={}, size={}, flags={}", sockfd, address, data.len(), flags);
        info!("PID {} sent {} bytes to {} on socket {}", pid, data.len(), address, sockfd);

        let result = serde_json::to_vec(&serde_json::json!({
            "bytes_sent": data.len()
        })).unwrap();

        SyscallResult::success_with_data(result)
    }

    pub(super) fn recvfrom(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("RecvFrom syscall not fully implemented: sockfd={}, size={}, flags={}", sockfd, size, flags);

        let result = serde_json::to_vec(&serde_json::json!({
            "data": "",
            "address": "0.0.0.0:0"
        })).unwrap();

        info!("PID {} received 0 bytes on socket {}", pid, sockfd);
        SyscallResult::success_with_data(result)
    }

    pub(super) fn close_socket(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("CloseSocket syscall not fully implemented: sockfd={}", sockfd);
        info!("PID {} closed socket {}", pid, sockfd);
        SyscallResult::success()
    }

    pub(super) fn setsockopt(&self, pid: Pid, sockfd: u32, level: u32, optname: u32, optval: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("SetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}", sockfd, level, optname);
        info!("PID {} set socket option on {}", pid, sockfd);
        SyscallResult::success()
    }

    pub(super) fn getsockopt(&self, pid: Pid, sockfd: u32, level: u32, optname: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!("GetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}", sockfd, level, optname);

        let result = serde_json::to_vec(&serde_json::json!({
            "value": 0
        })).unwrap();

        info!("PID {} got socket option from {}", pid, sockfd);
        SyscallResult::success_with_data(result)
    }
}
