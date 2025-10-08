/*!

* Network Syscalls
* Socket operations for TCP/UDP networking
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};

use ahash::RandomState;
use dashmap::DashMap;
use log::{info, warn};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;


use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// Socket manager for tracking open sockets
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic socket FD counter
#[repr(C, align(64))]
pub struct SocketManager {
    next_fd: Arc<AtomicU32>,
    tcp_listeners: Arc<DashMap<u32, TcpListener, RandomState>>,
    tcp_streams: Arc<DashMap<u32, TcpStream, RandomState>>,
    udp_sockets: Arc<DashMap<u32, UdpSocket, RandomState>>,
    /// Track which sockets belong to which process for cleanup
    process_sockets: Arc<DashMap<Pid, Vec<u32>, RandomState>>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            next_fd: Arc::new(AtomicU32::new(1000)), // Start socket FDs at 1000
            tcp_listeners: Arc::new(DashMap::with_hasher(RandomState::new())),
            tcp_streams: Arc::new(DashMap::with_hasher(RandomState::new())),
            udp_sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    fn allocate_fd(&self) -> u32 {
        self.next_fd.fetch_add(1, Ordering::SeqCst)
    }

    /// Track that a process owns a socket
    fn track_socket(&self, pid: Pid, sockfd: u32) {
        self.process_sockets
            .entry(pid)
            .or_insert_with(Vec::new)
            .push(sockfd);
    }

    /// Untrack a socket from a process
    fn untrack_socket(&self, pid: Pid, sockfd: u32) {
        if let Some(mut sockets) = self.process_sockets.get_mut(&pid) {
            sockets.retain(|&x| x != sockfd);
        }
    }

    /// Cleanup all sockets for a terminated process
    pub fn cleanup_process_sockets(&self, pid: Pid) -> usize {
        let sockets_to_close = if let Some((_, sockets)) = self.process_sockets.remove(&pid) {
            sockets
        } else {
            return 0;
        };

        let mut closed_count = 0;
        for sockfd in sockets_to_close {
            // Try to remove from all collections
            let removed = self.tcp_listeners.remove(&sockfd).is_some()
                || self.tcp_streams.remove(&sockfd).is_some()
                || self.udp_sockets.remove(&sockfd).is_some();

            if removed {
                closed_count += 1;
            }
        }

        closed_count
    }

    /// Check if process has any open sockets
    pub fn has_process_sockets(&self, pid: Pid) -> bool {
        self.process_sockets
            .get(&pid)
            .map_or(false, |sockets| !sockets.is_empty())
    }
}

impl Clone for SocketManager {
    fn clone(&self) -> Self {
        Self {
            next_fd: Arc::clone(&self.next_fd),
            tcp_listeners: Arc::clone(&self.tcp_listeners),
            tcp_streams: Arc::clone(&self.tcp_streams),
            udp_sockets: Arc::clone(&self.udp_sockets),
            process_sockets: Arc::clone(&self.process_sockets),
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
        // Check network capability via permission manager
        use crate::permissions::{Action, Resource};
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "network".to_string(),
            },
            Action::Create,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // AF_INET = 2, SOCK_STREAM = 1, SOCK_DGRAM = 2
        let sockfd = self.socket_manager.allocate_fd();

        // Track socket for this process
        self.socket_manager.track_socket(pid, sockfd);

        info!(
            "PID {} created socket FD {} (domain={}, type={}, protocol={})",
            pid, sockfd, domain, socket_type, protocol
        );

        match json::to_vec(&serde_json::json!({
            "sockfd": sockfd,
            "domain": domain,
            "type": socket_type,
            "protocol": protocol
        })) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                warn!("Failed to serialize socket result: {}", e);
                SyscallResult::error("Internal serialization error")
            }
        }
    }

    pub(super) fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        // Parse host:port from address and check bind permission
        use crate::permissions::{Action, Resource};
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::new(pid, Resource::Network { host, port }, Action::Bind);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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
        // Listen requires network access (socket already bound)
        use crate::permissions::{Action, Resource};
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "network".to_string(),
            },
            Action::Bind,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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
        // Accept requires network access
        use crate::permissions::{Action, Resource};
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "network".to_string(),
            },
            Action::Receive,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Get the listener and try to accept (non-blocking)
        if let Some(listener) = self.socket_manager.tcp_listeners.get(&sockfd) {
            // Set non-blocking mode for this operation
            if let Ok((stream, addr)) = listener.accept() {
                drop(listener);

                // Allocate new FD for the accepted connection becuase why wouldn't we
                let client_fd = self.socket_manager.allocate_fd();
                self.socket_manager.tcp_streams.insert(client_fd, stream);

                // Track the new client socket for this process
                self.socket_manager.track_socket(pid, client_fd);

                info!(
                    "PID {} accepted connection on socket {}, client FD {} from {}",
                    pid, sockfd, client_fd, addr
                );

                match json::to_vec(&serde_json::json!({
                    "client_fd": client_fd,
                    "address": addr.to_string()
                })) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        warn!("Failed to serialize accept result: {}", e);
                        SyscallResult::error("Internal serialization error")
                    }
                }
            } else {
                SyscallResult::error("No pending connections")
            }
        } else {
            SyscallResult::error("Invalid socket or not listening")
        }
    }

    pub(super) fn connect(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        // Parse host:port from address and check connect permission
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::net_connect(pid, host, port);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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

    pub(super) fn send(&self, pid: Pid, sockfd: u32, data: &[u8], _flags: u32) -> SyscallResult {
        // Send on existing connection - permissions already checked at connect/accept time
        // Could add additional check here if needed

        use std::io::Write;

        // Try to send on TCP stream
        if let Some(mut stream) = self.socket_manager.tcp_streams.get_mut(&sockfd) {
            match stream.write(data) {
                Ok(bytes_sent) => {
                    info!(
                        "PID {} sent {} bytes on TCP socket {}",
                        pid, bytes_sent, sockfd
                    );
                    match json::to_vec(&serde_json::json!({
                        "bytes_sent": bytes_sent
                    })) {
                        Ok(result) => SyscallResult::success_with_data(result),
                        Err(e) => {
                            warn!("Failed to serialize send result: {}", e);
                            SyscallResult::error("Internal serialization error")
                        }
                    }
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

    pub(super) fn recv(&self, pid: Pid, sockfd: u32, size: usize, _flags: u32) -> SyscallResult {
        // Receive on existing connection - permissions already checked at connect/accept time
        // Could add additional check here if needed

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
        _flags: u32,
    ) -> SyscallResult {
        // Parse host:port from address and check network access
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::net_connect(pid, host, port);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Try to send via UDP socket
        if let Some(socket) = self.socket_manager.udp_sockets.get(&sockfd) {
            match socket.send_to(data, address) {
                Ok(bytes_sent) => {
                    info!(
                        "PID {} sent {} bytes to {} on UDP socket {}",
                        pid, bytes_sent, address, sockfd
                    );
                    match json::to_vec(&serde_json::json!({
                        "bytes_sent": bytes_sent
                    })) {
                        Ok(result) => SyscallResult::success_with_data(result),
                        Err(e) => {
                            warn!("Failed to serialize sendto result: {}", e);
                            SyscallResult::error("Internal serialization error")
                        }
                    }
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

    pub(super) fn recvfrom(&self, pid: Pid, sockfd: u32, size: usize, _flags: u32) -> SyscallResult {
        // Receive on UDP socket - permissions should be checked at bind time
        // Could add additional check here if needed

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

                    match json::to_vec(&serde_json::json!({
                        "data": buffer,
                        "address": addr.to_string()
                    })) {
                        Ok(result) => SyscallResult::success_with_data(result),
                        Err(e) => {
                            warn!("Failed to serialize recvfrom result: {}", e);
                            SyscallResult::error("Internal serialization error")
                        }
                    }
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
        // Close doesn't require permission check - closing is always allowed cuz I said so

        // Try to remove from all socket collections
        let removed = self.socket_manager.tcp_listeners.remove(&sockfd).is_some()
            || self.socket_manager.tcp_streams.remove(&sockfd).is_some()
            || self.socket_manager.udp_sockets.remove(&sockfd).is_some();

        if removed {
            // Untrack socket from process
            self.socket_manager.untrack_socket(pid, sockfd);
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
        _optval: &[u8],
    ) -> SyscallResult {
        // Socket options on existing socket - permissions checked at creation time

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
        // Socket options on existing socket - permissions checked at creation time

        warn!(
            "GetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}",
            sockfd, level, optname
        );

        let result = match json::to_vec(&serde_json::json!({
            "value": 0
        })) {
            Ok(data) => data,
            Err(e) => {
                warn!("Failed to serialize getsockopt result: {}", e);
                return SyscallResult::error("Internal serialization error");
            }
        };

        info!("PID {} got socket option from {}", pid, sockfd);
        SyscallResult::success_with_data(result)
    }
}
