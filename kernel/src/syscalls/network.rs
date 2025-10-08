/*!

* Network Syscalls
* Socket operations for TCP/UDP networking
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};

use ahash::RandomState;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use log::{error, info, warn};
use std::collections::HashSet;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// Socket type for efficient single-lookup cleanup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SocketType {
    TcpListener,
    TcpStream,
    UdpSocket,
}

/// Socket manager for tracking open sockets
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic socket FD counter
/// - HashSet-based per-process tracking for O(1) untrack operations
/// - Lock-free FD recycling via SegQueue to prevent ID exhaustion
/// - Socket type tagging for single-lookup cleanup (3x faster)
/// - Atomic count tracking for O(1) limit checks
#[repr(C, align(64))]
pub struct SocketManager {
    next_fd: Arc<AtomicU32>,
    tcp_listeners: Arc<DashMap<u32, TcpListener, RandomState>>,
    tcp_streams: Arc<DashMap<u32, TcpStream, RandomState>>,
    udp_sockets: Arc<DashMap<u32, UdpSocket, RandomState>>,
    /// Socket type tags for efficient cleanup (avoids triple-lookup)
    socket_types: Arc<DashMap<u32, SocketType, RandomState>>,
    /// Track which sockets belong to which process (HashSet for O(1) untrack)
    process_sockets: Arc<DashMap<Pid, HashSet<u32>, RandomState>>,
    /// Per-process socket counts for O(1) limit checks (lock-free via alter())
    process_socket_counts: Arc<DashMap<Pid, u32, RandomState>>,
    /// Lock-free queue for FD recycling (prevents FD exhaustion)
    free_fds: Arc<SegQueue<u32>>,
}

impl SocketManager {
    pub fn new() -> Self {
        info!("Socket manager initialized with lock-free FD recycling and O(1) tracking");
        Self {
            next_fd: Arc::new(AtomicU32::new(1000)), // Start socket FDs at 1000
            tcp_listeners: Arc::new(DashMap::with_hasher(RandomState::new())),
            tcp_streams: Arc::new(DashMap::with_hasher(RandomState::new())),
            udp_sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
            socket_types: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_socket_counts: Arc::new(DashMap::with_hasher(RandomState::new())),
            free_fds: Arc::new(SegQueue::new()),
        }
    }

    /// Allocate a socket FD (recycle or create new, lock-free)
    fn allocate_fd(&self) -> u32 {
        if let Some(recycled_fd) = self.free_fds.pop() {
            recycled_fd
        } else {
            self.next_fd.fetch_add(1, Ordering::SeqCst)
        }
    }

    /// Get current socket count for a process (O(1) lookup)
    pub fn get_socket_count(&self, pid: Pid) -> u32 {
        self.process_socket_counts
            .get(&pid)
            .map(|r| *r.value())
            .unwrap_or(0)
    }

    /// Track that a process owns a socket (atomic increment)
    fn track_socket(&self, pid: Pid, sockfd: u32, socket_type: SocketType) {
        self.socket_types.insert(sockfd, socket_type);

        self.process_sockets.alter(&pid, |_, mut sockets| {
            sockets.insert(sockfd);
            sockets
        });

        // Atomic increment using alter() for lock-free counting
        self.process_socket_counts.alter(&pid, |_, count| count + 1);
    }

    /// Untrack a socket from a process (atomic decrement, O(1) removal)
    fn untrack_socket(&self, pid: Pid, sockfd: u32) {
        self.socket_types.remove(&sockfd);

        if let Some(mut sockets) = self.process_sockets.get_mut(&pid) {
            sockets.remove(&sockfd);
        }

        // Atomic decrement using alter() for lock-free counting
        self.process_socket_counts
            .alter(&pid, |_, count| count.saturating_sub(1));
    }

    /// Cleanup all sockets for a terminated process
    pub fn cleanup_process_sockets(&self, pid: Pid) -> usize {
        let sockets_to_close = if let Some((_, sockets)) = self.process_sockets.remove(&pid) {
            sockets.into_iter().collect::<Vec<_>>()
        } else {
            return 0;
        };

        let mut closed_count = 0;
        for sockfd in sockets_to_close {
            // Single lookup using type tag (3x faster than trying all collections)
            if let Some((_, socket_type)) = self.socket_types.remove(&sockfd) {
                let removed = match socket_type {
                    SocketType::TcpListener => self.tcp_listeners.remove(&sockfd).is_some(),
                    SocketType::TcpStream => self.tcp_streams.remove(&sockfd).is_some(),
                    SocketType::UdpSocket => self.udp_sockets.remove(&sockfd).is_some(),
                };

                if removed {
                    closed_count += 1;
                    // Recycle FD for reuse (lock-free)
                    self.free_fds.push(sockfd);
                }
            }
        }

        // Remove the count entry
        self.process_socket_counts.remove(&pid);

        closed_count
    }

    /// Check if process has any open sockets (O(1) check)
    pub fn has_process_sockets(&self, pid: Pid) -> bool {
        self.get_socket_count(pid) > 0
    }

    /// Get socket statistics
    pub fn stats(&self) -> SocketStats {
        SocketStats {
            total_tcp_listeners: self.tcp_listeners.len(),
            total_tcp_streams: self.tcp_streams.len(),
            total_udp_sockets: self.udp_sockets.len(),
            recycled_fds_available: self.free_fds.len(),
        }
    }
}

/// Socket statistics
#[derive(Debug, Clone)]
pub struct SocketStats {
    pub total_tcp_listeners: usize,
    pub total_tcp_streams: usize,
    pub total_udp_sockets: usize,
    pub recycled_fds_available: usize,
}

impl SocketStats {
    /// Total sockets across all types
    pub fn total_sockets(&self) -> usize {
        self.total_tcp_listeners + self.total_tcp_streams + self.total_udp_sockets
    }
}

impl Clone for SocketManager {
    fn clone(&self) -> Self {
        Self {
            next_fd: Arc::clone(&self.next_fd),
            tcp_listeners: Arc::clone(&self.tcp_listeners),
            tcp_streams: Arc::clone(&self.tcp_streams),
            udp_sockets: Arc::clone(&self.udp_sockets),
            socket_types: Arc::clone(&self.socket_types),
            process_sockets: Arc::clone(&self.process_sockets),
            process_socket_counts: Arc::clone(&self.process_socket_counts),
            free_fds: Arc::clone(&self.free_fds),
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
        // Check per-process socket limit BEFORE doing expensive operations
        use crate::security::ResourceLimitProvider;
        if let Some(limits) = self.sandbox_manager.get_limits(pid) {
            let current_socket_count = self.socket_manager.get_socket_count(pid);
            if current_socket_count >= limits.max_file_descriptors {
                error!(
                    "PID {} exceeded socket limit: {}/{} sockets",
                    pid, current_socket_count, limits.max_file_descriptors
                );
                return SyscallResult::permission_denied(format!(
                    "Socket limit exceeded: {}/{} sockets open",
                    current_socket_count, limits.max_file_descriptors
                ));
            }
        }

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

        // Determine socket type based on type parameter
        let sock_type = match socket_type {
            1 => SocketType::TcpStream, // SOCK_STREAM
            2 => SocketType::UdpSocket, // SOCK_DGRAM
            _ => SocketType::TcpStream, // Default to TCP
        };

        // Track socket for this process with type tag
        self.socket_manager.track_socket(pid, sockfd, sock_type);

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
                // Update socket type to listener
                self.socket_manager.socket_types.insert(sockfd, SocketType::TcpListener);
                info!("PID {} bound TCP socket {} to {}", pid, sockfd, address);
                SyscallResult::success()
            }
            Err(e) => {
                // Try UDP if TCP failed
                match UdpSocket::bind(address) {
                    Ok(socket) => {
                        self.socket_manager.udp_sockets.insert(sockfd, socket);
                        // Update socket type to UDP
                        self.socket_manager.socket_types.insert(sockfd, SocketType::UdpSocket);
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

                // Track the new client socket for this process with type tag
                self.socket_manager.track_socket(pid, client_fd, SocketType::TcpStream);

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
                // Update socket type to stream (was allocated generically)
                self.socket_manager.socket_types.insert(sockfd, SocketType::TcpStream);
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

        // Single lookup using type tag (3x faster)
        if let Some((_, socket_type)) = self.socket_manager.socket_types.remove(&sockfd) {
            let removed = match socket_type {
                SocketType::TcpListener => self.socket_manager.tcp_listeners.remove(&sockfd).is_some(),
                SocketType::TcpStream => self.socket_manager.tcp_streams.remove(&sockfd).is_some(),
                SocketType::UdpSocket => self.socket_manager.udp_sockets.remove(&sockfd).is_some(),
            };

            if removed {
                // Untrack socket from process (O(1) with HashSet)
                self.socket_manager.untrack_socket(pid, sockfd);
                // Recycle FD for reuse (lock-free)
                self.socket_manager.free_fds.push(sockfd);
                info!("PID {} closed socket {} (recycled FD)", pid, sockfd);
                SyscallResult::success()
            } else {
                warn!("Socket {} type found but socket not in collection", sockfd);
                SyscallResult::error("Socket inconsistency")
            }
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
