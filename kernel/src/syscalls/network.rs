/*!

* Network Syscalls
* Socket operations for TCP/UDP networking
*/

use crate::core::types::Pid;

use dashmap::DashMap;
use log::{info, warn};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::security::{Capability, NetworkRule};

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// Socket manager for tracking open sockets
pub struct SocketManager {
    next_fd: Arc<AtomicU32>,
    tcp_listeners: Arc<DashMap<u32, TcpListener>>,
    tcp_streams: Arc<DashMap<u32, TcpStream>>,
    udp_sockets: Arc<DashMap<u32, UdpSocket>>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            next_fd: Arc::new(AtomicU32::new(1000)), // Start socket FDs at 1000
            tcp_listeners: Arc::new(DashMap::new()),
            tcp_streams: Arc::new(DashMap::new()),
            udp_sockets: Arc::new(DashMap::new()),
        }
    }

    fn allocate_fd(&self) -> u32 {
        self.next_fd.fetch_add(1, Ordering::SeqCst)
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
    pub(super) fn socket(
        &self,
        pid: Pid,
        domain: u32,
        socket_type: u32,
        protocol: u32,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // AF_INET = 2, SOCK_STREAM = 1, SOCK_DGRAM = 2
        let sockfd = self.socket_manager.allocate_fd();

        info!(
            "PID {} created socket FD {} (domain={}, type={}, protocol={})",
            pid, sockfd, domain, socket_type, protocol
        );

        let data = serde_json::to_vec(&serde_json::json!({
            "sockfd": sockfd,
            "domain": domain,
            "type": socket_type,
            "protocol": protocol
        }))
        .unwrap();

        SyscallResult::success_with_data(data)
    }

    pub(super) fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::BindPort(None))
        {
            return SyscallResult::permission_denied("Missing BindPort capability");
        }

        // Try to bind a TCP listener
        match TcpListener::bind(address) {
            Ok(listener) => {
                self.socket_manager.tcp_listeners.insert(sockfd, listener);
                info!("PID {} bound TCP socket {} to {}", pid, sockfd, address);
                SyscallResult::success()
            }
            Err(e) => {
                // Try UDP if TCP failed
                match UdpSocket::bind(address) {
                    Ok(socket) => {
                        self.socket_manager.udp_sockets.insert(sockfd, socket);
                        info!("PID {} bound UDP socket {} to {}", pid, sockfd, address);
                        SyscallResult::success()
                    }
                    Err(udp_e) => {
                        warn!(
                            "Failed to bind socket {} to {}: TCP={}, UDP={}",
                            sockfd, address, e, udp_e
                        );
                        SyscallResult::error(format!("Bind failed: {}", e))
                    }
                }
            }
        }
    }

    pub(super) fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::BindPort(None))
        {
            return SyscallResult::permission_denied("Missing BindPort capability");
        }

        // Verify socket exists as a TCP listener
        if self.socket_manager.tcp_listeners.contains_key(&sockfd) {
            info!(
                "PID {} listening on socket {} with backlog {}",
                pid, sockfd, backlog
            );
            SyscallResult::success()
        } else {
            SyscallResult::error("Socket not bound or not a TCP socket")
        }
    }

    pub(super) fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Get the listener and try to accept (non-blocking)
        if let Some(listener) = self.socket_manager.tcp_listeners.get(&sockfd) {
            // Set non-blocking mode for this operation
            if let Ok((stream, addr)) = listener.accept() {
                drop(listener);

                // Allocate new FD for the accepted connection
                let client_fd = self.socket_manager.allocate_fd();
                self.socket_manager.tcp_streams.insert(client_fd, stream);

                info!(
                    "PID {} accepted connection on socket {}, client FD {} from {}",
                    pid, sockfd, client_fd, addr
                );

                let data = serde_json::to_vec(&serde_json::json!({
                    "client_fd": client_fd,
                    "address": addr.to_string()
                }))
                .unwrap();

                SyscallResult::success_with_data(data)
            } else {
                SyscallResult::error("No pending connections")
            }
        } else {
            SyscallResult::error("Invalid socket or not listening")
        }
    }

    pub(super) fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Try to connect to the address
        match TcpStream::connect(address) {
            Ok(stream) => {
                self.socket_manager.tcp_streams.insert(sockfd, stream);
                info!("PID {} connected socket {} to {}", pid, sockfd, address);
                SyscallResult::success()
            }
            Err(e) => {
                warn!("Failed to connect socket {} to {}: {}", sockfd, address, e);
                SyscallResult::error(format!("Connect failed: {}", e))
            }
        }
    }

    pub(super) fn send(&self, pid: Pid, sockfd: u32, data: &[u8], flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        use std::io::Write;

        // Try to send on TCP stream
        if let Some(mut stream) = self.socket_manager.tcp_streams.get_mut(&sockfd) {
            match stream.write(data) {
                Ok(bytes_sent) => {
                    info!(
                        "PID {} sent {} bytes on TCP socket {}",
                        pid, bytes_sent, sockfd
                    );
                    let result = serde_json::to_vec(&serde_json::json!({
                        "bytes_sent": bytes_sent
                    }))
                    .unwrap();
                    SyscallResult::success_with_data(result)
                }
                Err(e) => {
                    warn!("Send failed on socket {}: {}", sockfd, e);
                    SyscallResult::error(format!("Send failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid socket or not connected")
        }
    }

    pub(super) fn recv(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        use std::io::Read;

        // Try to receive from TCP stream
        if let Some(mut stream) = self.socket_manager.tcp_streams.get_mut(&sockfd) {
            let mut buffer = vec![0u8; size];
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    buffer.truncate(bytes_read);
                    info!(
                        "PID {} received {} bytes on TCP socket {}",
                        pid, bytes_read, sockfd
                    );
                    SyscallResult::success_with_data(buffer)
                }
                Err(e) => {
                    warn!("Recv failed on socket {}: {}", sockfd, e);
                    SyscallResult::error(format!("Recv failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid socket or not connected")
        }
    }

    pub(super) fn sendto(
        &self,
        pid: Pid,
        sockfd: u32,
        data: &[u8],
        address: &str,
        flags: u32,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Try to send via UDP socket
        if let Some(socket) = self.socket_manager.udp_sockets.get(&sockfd) {
            match socket.send_to(data, address) {
                Ok(bytes_sent) => {
                    info!(
                        "PID {} sent {} bytes to {} on UDP socket {}",
                        pid, bytes_sent, address, sockfd
                    );
                    let result = serde_json::to_vec(&serde_json::json!({
                        "bytes_sent": bytes_sent
                    }))
                    .unwrap();
                    SyscallResult::success_with_data(result)
                }
                Err(e) => {
                    warn!("SendTo failed on socket {}: {}", sockfd, e);
                    SyscallResult::error(format!("SendTo failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid UDP socket")
        }
    }

    pub(super) fn recvfrom(&self, pid: Pid, sockfd: u32, size: usize, flags: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Try to receive from UDP socket
        if let Some(socket) = self.socket_manager.udp_sockets.get(&sockfd) {
            let mut buffer = vec![0u8; size];
            match socket.recv_from(&mut buffer) {
                Ok((bytes_read, addr)) => {
                    buffer.truncate(bytes_read);
                    info!(
                        "PID {} received {} bytes from {} on UDP socket {}",
                        pid, bytes_read, addr, sockfd
                    );

                    let result = serde_json::to_vec(&serde_json::json!({
                        "data": buffer,
                        "address": addr.to_string()
                    }))
                    .unwrap();

                    SyscallResult::success_with_data(result)
                }
                Err(e) => {
                    warn!("RecvFrom failed on socket {}: {}", sockfd, e);
                    SyscallResult::error(format!("RecvFrom failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid UDP socket")
        }
    }

    pub(super) fn close_socket(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Try to remove from all socket collections
        let removed = self.socket_manager.tcp_listeners.remove(&sockfd).is_some()
            || self.socket_manager.tcp_streams.remove(&sockfd).is_some()
            || self.socket_manager.udp_sockets.remove(&sockfd).is_some();

        if removed {
            info!("PID {} closed socket {}", pid, sockfd);
            SyscallResult::success()
        } else {
            warn!(
                "PID {} attempted to close non-existent socket {}",
                pid, sockfd
            );
            SyscallResult::error("Invalid socket descriptor")
        }
    }

    pub(super) fn setsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
        optval: &[u8],
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!(
            "SetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}",
            sockfd, level, optname
        );
        info!("PID {} set socket option on {}", pid, sockfd);
        SyscallResult::success()
    }

    pub(super) fn getsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess(NetworkRule::AllowAll))
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        warn!(
            "GetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}",
            sockfd, level, optname
        );

        let result = serde_json::to_vec(&serde_json::json!({
            "value": 0
        }))
        .unwrap();

        info!("PID {} got socket option from {}", pid, sockfd);
        SyscallResult::success_with_data(result)
    }
}
