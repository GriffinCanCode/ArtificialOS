/*!

* Network Syscalls
* Socket operations for TCP/UDP networking
*/

use crate::syscalls::timeout::executor::TimeoutError;

use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::permissions::{PermissionChecker, PermissionRequest};

use ahash::RandomState;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use log::{error, info, trace, warn};
use std::collections::HashSet;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::SyscallResult;

/// Unified socket abstraction - eliminates need for separate collections per type
///
/// This enum wraps all socket types into a single type, enabling:
/// - Single DashMap instead of 3 separate collections
/// - No need for separate type tagging map
/// - Single lookup during cleanup (enum variant IS the type)
#[derive(Debug)]
pub enum Socket {
    TcpListener(TcpListener),
    TcpStream(TcpStream),
    UdpSocket(UdpSocket),
}

impl Socket {
    /// Get socket type name for logging
    fn type_name(&self) -> &'static str {
        match self {
            Socket::TcpListener(_) => "TcpListener",
            Socket::TcpStream(_) => "TcpStream",
            Socket::UdpSocket(_) => "UdpSocket",
        }
    }
}

/// Socket manager for tracking open sockets
///
/// # Design Philosophy
/// - **Unified Storage**: Single DashMap for all socket types (via enum wrapper)
/// - **Cache-line aligned**: Prevents false sharing of atomic FD counter
/// - **HashSet tracking**: O(1) per-process socket tracking and removal
/// - **Lock-free FD recycling**: SegQueue prevents FD exhaustion
/// - **Simplified architecture**: 3 data structures vs. previous 6 (50% reduction)
///
/// # Performance
/// - Single lookup for cleanup (no type map needed)
/// - O(1) count checks via HashSet::len()
/// - Reduced cache pressure from fewer data structures
#[repr(C, align(64))]
pub struct SocketManager {
    next_fd: Arc<AtomicU32>,
    /// Unified socket storage - all types in one collection
    sockets: Arc<DashMap<u32, Socket, RandomState>>,
    /// Track which sockets belong to which process (HashSet for O(1) operations)
    process_sockets: Arc<DashMap<Pid, HashSet<u32>, RandomState>>,
    /// Lock-free queue for FD recycling (prevents FD exhaustion)
    free_fds: Arc<SegQueue<u32>>,
}

impl SocketManager {
    pub fn new() -> Self {
        let span = span_operation("socket_manager_init");
        let _guard = span.enter();
        info!("Socket manager initialized with unified storage and lock-free FD recycling");
        span.record_result(true);
        Self {
            next_fd: Arc::new(AtomicU32::new(1000)), // Start socket FDs at 1000
            sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_sockets: Arc::new(DashMap::with_hasher(RandomState::new())),
            free_fds: Arc::new(SegQueue::new()),
        }
    }

    /// Allocate a socket FD (recycle or create new, lock-free)
    fn allocate_fd(&self) -> u32 {
        if let Some(recycled_fd) = self.free_fds.pop() {
            trace!("Recycled FD {} for socket", recycled_fd);
            recycled_fd
        } else {
            let new_fd = self.next_fd.fetch_add(1, Ordering::SeqCst);
            trace!("Allocated new FD {} for socket", new_fd);
            new_fd
        }
    }

    /// Recycle a file descriptor for reuse (lock-free)
    fn recycle_fd(&self, fd: u32) {
        self.free_fds.push(fd);
        trace!("Recycled FD {} for reuse", fd);
    }

    /// Get current socket count for a process (O(1) via HashSet::len)
    pub fn get_socket_count(&self, pid: Pid) -> u32 {
        self.process_sockets
            .get(&pid)
            .map(|sockets| sockets.len() as u32)
            .unwrap_or(0)
    }

    /// Track that a process owns a socket
    fn track_socket(&self, pid: Pid, sockfd: u32) {
        self.process_sockets
            .entry(pid)
            .or_insert_with(HashSet::new)
            .insert(sockfd);
    }

    /// Untrack a socket from a process (O(1) removal)
    fn untrack_socket(&self, pid: Pid, sockfd: u32) {
        if let Some(mut sockets) = self.process_sockets.get_mut(&pid) {
            sockets.remove(&sockfd);
        }
    }

    /// Cleanup all sockets for a terminated process
    ///
    /// # Design
    /// Single-pass cleanup with unified storage:
    /// 1. Remove process tracking (one atomic operation)
    /// 2. Close each socket (single lookup per socket)
    /// 3. Recycle FDs for reuse
    pub fn cleanup_process_sockets(&self, pid: Pid) -> usize {
        let span = span_operation("socket_cleanup_process");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        // Remove all socket FDs owned by this process (atomic operation)
        let sockets_to_close = if let Some((_, sockets)) = self.process_sockets.remove(&pid) {
            sockets
        } else {
            trace!("No sockets to cleanup for PID {}", pid);
            span.record("closed_count", "0");
            span.record_result(true);
            return 0;
        };

        let socket_count = sockets_to_close.len();
        let mut closed_count = 0;
        for sockfd in sockets_to_close {
            // Single lookup in unified collection (no type map needed)
            if let Some((_, socket)) = self.sockets.remove(&sockfd) {
                closed_count += 1;
                // Socket is dropped here, closing OS resource automatically
                let type_name = socket.type_name();
                trace!("Closed {} socket FD {} for PID {}", type_name, sockfd, pid);

                // Recycle FD for reuse (lock-free)
                self.free_fds.push(sockfd);
            }
        }

        info!(
            "Cleaned up {}/{} sockets for PID {}",
            closed_count, socket_count, pid
        );
        span.record("closed_count", &format!("{}", closed_count));
        span.record_result(true);
        closed_count
    }

    /// Check if process has any open sockets (O(1) check)
    pub fn has_process_sockets(&self, pid: Pid) -> bool {
        self.process_sockets
            .get(&pid)
            .map(|sockets| !sockets.is_empty())
            .unwrap_or(false)
    }

    /// Get socket statistics
    pub fn stats(&self) -> SocketStats {
        // Count by iterating and matching variants (happens rarely, during stats collection)
        let mut tcp_listeners = 0;
        let mut tcp_streams = 0;
        let mut udp_sockets = 0;

        for entry in self.sockets.iter() {
            match entry.value() {
                Socket::TcpListener(_) => tcp_listeners += 1,
                Socket::TcpStream(_) => tcp_streams += 1,
                Socket::UdpSocket(_) => udp_sockets += 1,
            }
        }

        SocketStats {
            total_tcp_listeners: tcp_listeners,
            total_tcp_streams: tcp_streams,
            total_udp_sockets: udp_sockets,
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
            sockets: Arc::clone(&self.sockets),
            process_sockets: Arc::clone(&self.process_sockets),
            free_fds: Arc::clone(&self.free_fds),
        }
    }
}

impl SyscallExecutorWithIpc {
    pub(in crate::syscalls) fn socket(
        &self,
        pid: Pid,
        domain: u32,
        socket_type: u32,
        protocol: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_create");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("domain", &format!("{}", domain));
        span.record("socket_type", &format!("{}", socket_type));
        span.record("protocol", &format!("{}", protocol));

        // Check per-process socket limit BEFORE doing expensive operations
        use crate::security::ResourceLimitProvider;
        if let Some(limits) = self.sandbox_manager().get_limits(pid) {
            let current_socket_count = self.socket_manager().get_socket_count(pid);
            trace!(
                "PID {} has {}/{} sockets open",
                pid,
                current_socket_count,
                limits.max_file_descriptors
            );
            if current_socket_count >= limits.max_file_descriptors {
                error!(
                    "PID {} exceeded socket limit: {}/{} sockets",
                    pid, current_socket_count, limits.max_file_descriptors
                );
                span.record_error(&format!(
                    "Socket limit exceeded: {}/{}",
                    current_socket_count, limits.max_file_descriptors
                ));
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
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // AF_INET = 2, SOCK_STREAM = 1, SOCK_DGRAM = 2
        let sockfd = self.socket_manager().allocate_fd();

        // Track socket for this process (socket will be created on bind/connect)
        self.socket_manager().track_socket(pid, sockfd);

        info!(
            "PID {} allocated socket FD {} (domain={}, type={}, protocol={})",
            pid, sockfd, domain, socket_type, protocol
        );
        span.record("sockfd", &format!("{}", sockfd));
        span.record_result(true);

        match json::to_vec(&serde_json::json!({
            "sockfd": sockfd,
            "domain": domain,
            "type": socket_type,
            "protocol": protocol
        })) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                warn!("Failed to serialize socket result: {}", e);
                span.record_error("Serialization failed");
                SyscallResult::error("Internal serialization error")
            }
        }
    }

    pub(in crate::syscalls) fn bind(&self, pid: Pid, sockfd: u32, address: &str) -> SyscallResult {
        let span = span_operation("socket_bind");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("address", address);

        // Parse host:port from address and check bind permission
        use crate::permissions::{Action, Resource};
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::new(pid, Resource::Network { host, port }, Action::Bind);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try to bind a TCP listener
        match TcpListener::bind(address) {
            Ok(listener) => {
                self.socket_manager()
                    .sockets
                    .insert(sockfd, Socket::TcpListener(listener));
                info!("PID {} bound TCP socket {} to {}", pid, sockfd, address);
                span.record("socket_type", "TCP");
                span.record_result(true);
                SyscallResult::success()
            }
            Err(e) => {
                trace!("TCP bind failed for socket {}, trying UDP: {}", sockfd, e);
                // Try UDP if TCP failed
                match UdpSocket::bind(address) {
                    Ok(socket) => {
                        self.socket_manager()
                            .sockets
                            .insert(sockfd, Socket::UdpSocket(socket));
                        info!("PID {} bound UDP socket {} to {}", pid, sockfd, address);
                        span.record("socket_type", "UDP");
                        span.record_result(true);
                        SyscallResult::success()
                    }
                    Err(udp_e) => {
                        warn!(
                            "Failed to bind socket {} to {}: TCP={}, UDP={}",
                            sockfd, address, e, udp_e
                        );
                        span.record_error(&format!("Bind failed: {}", e));
                        SyscallResult::error(format!("Bind failed: {}", e))
                    }
                }
            }
        }
    }

    pub(in crate::syscalls) fn listen(&self, pid: Pid, sockfd: u32, backlog: u32) -> SyscallResult {
        let span = span_operation("socket_listen");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("backlog", &format!("{}", backlog));

        // Listen requires network access (socket already bound)
        use crate::permissions::{Action, Resource};
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "network".to_string(),
            },
            Action::Bind,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Verify socket exists and is a TCP listener
        if let Some(socket) = self.socket_manager().sockets.get(&sockfd) {
            match socket.value() {
                Socket::TcpListener(_) => {
                    info!(
                        "PID {} listening on socket {} with backlog {}",
                        pid, sockfd, backlog
                    );
                    span.record_result(true);
                    SyscallResult::success()
                }
                _ => {
                    span.record_error("Socket not a TCP listener");
                    SyscallResult::error("Socket not a TCP listener")
                }
            }
        } else {
            span.record_error("Socket not found or not bound");
            SyscallResult::error("Socket not found or not bound")
        }
    }

    pub(in crate::syscalls) fn accept(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        let span = span_operation("socket_accept");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));

        // Accept requires network access
        use crate::permissions::{Action, Resource};
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "network".to_string(),
            },
            Action::Receive,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Use timeout executor for blocking accept
        #[derive(Debug)]
        enum AcceptError {
            NoPendingConnections,
            NotListener,
            InvalidSocket,
            Other(String),
        }

        let result = self.timeout_executor().execute_with_retry(
            || {
                if let Some(socket) = self.socket_manager().sockets.get(&sockfd) {
                    match socket.value() {
                        Socket::TcpListener(listener) => {
                            match listener.accept() {
                                Ok((stream, addr)) => {
                                    drop(socket); // Release lock before returning
                                    Ok((stream, addr))
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    Err(AcceptError::NoPendingConnections)
                                }
                                Err(e) => Err(AcceptError::Other(e.to_string())),
                            }
                        }
                        _ => Err(AcceptError::NotListener),
                    }
                } else {
                    Err(AcceptError::InvalidSocket)
                }
            },
            |e| matches!(e, AcceptError::NoPendingConnections),
            self.timeout_config().network,
            "socket_accept",
        );

        match result {
            Ok((stream, addr)) => {
                // Allocate new FD for the accepted connection
                let client_fd = self.socket_manager().allocate_fd();
                self.socket_manager()
                    .sockets
                    .insert(client_fd, Socket::TcpStream(stream));

                // Track the new client socket for this process
                self.socket_manager().track_socket(pid, client_fd);

                info!(
                    "PID {} accepted connection on socket {}, client FD {} from {}",
                    pid, sockfd, client_fd, addr
                );
                span.record("client_fd", &format!("{}", client_fd));
                span.record("client_address", &addr.to_string());
                span.record_result(true);

                match json::to_vec(&serde_json::json!({
                    "client_fd": client_fd,
                    "address": addr.to_string()
                })) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        warn!("Failed to serialize accept result: {}", e);
                        span.record_error("Serialization failed");
                        SyscallResult::error("Internal serialization error")
                    }
                }
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Accept timed out for PID {}, socket {} after {}ms",
                    pid, sockfd, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error("Accept timed out")
            }
            Err(TimeoutError::Operation(AcceptError::NotListener)) => {
                span.record_error("Socket is not a TCP listener");
                SyscallResult::error("Socket is not a TCP listener")
            }
            Err(TimeoutError::Operation(AcceptError::InvalidSocket)) => {
                span.record_error("Invalid socket or not listening");
                SyscallResult::error("Invalid socket or not listening")
            }
            Err(TimeoutError::Operation(AcceptError::NoPendingConnections)) => {
                span.record_error("No pending connections");
                SyscallResult::error("No pending connections")
            }
            Err(TimeoutError::Operation(AcceptError::Other(e))) => {
                span.record_error(&format!("Accept failed: {}", e));
                SyscallResult::error(format!("Accept failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn connect(
        &self,
        pid: Pid,
        sockfd: u32,
        address: &str,
    ) -> SyscallResult {
        let span = span_operation("socket_connect");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("address", address);

        // Parse host:port from address and check connect permission
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::net_connect(pid, host, port);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Use timeout executor for blocking connect
        let address_owned = address.to_string();
        let result = self.timeout_executor().execute_with_deadline(
            || TcpStream::connect(&address_owned),
            self.timeout_config().network,
            "socket_connect",
        );

        match result {
            Ok(stream) => {
                self.socket_manager()
                    .sockets
                    .insert(sockfd, Socket::TcpStream(stream));
                info!("PID {} connected socket {} to {}", pid, sockfd, address);
                span.record_result(true);
                SyscallResult::success()
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Connect timed out for PID {}, socket {} to {} after {}ms",
                    pid, sockfd, address, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error("Connect timed out")
            }
            Err(TimeoutError::Operation(e)) => {
                warn!("Failed to connect socket {} to {}: {}", sockfd, address, e);
                span.record_error(&format!("Connect failed: {}", e));
                SyscallResult::error(format!("Connect failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn send(
        &self,
        pid: Pid,
        sockfd: u32,
        data: &[u8],
        _flags: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_send");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("data_size", &format!("{}", data.len()));

        // Send on existing connection - permissions already checked at connect/accept time

        use std::io::Write;

        #[derive(Debug)]
        enum SendError {
            WouldBlock,
            NotStream,
            InvalidSocket,
            Other(String),
        }

        let data_to_send = data.to_vec();
        let result = self.timeout_executor().execute_with_retry(
            || {
                if let Some(mut socket) = self.socket_manager().sockets.get_mut(&sockfd) {
                    match socket.value_mut() {
                        Socket::TcpStream(stream) => match stream.write(&data_to_send) {
                            Ok(bytes_sent) => Ok(bytes_sent),
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                Err(SendError::WouldBlock)
                            }
                            Err(e) => Err(SendError::Other(e.to_string())),
                        },
                        _ => Err(SendError::NotStream),
                    }
                } else {
                    Err(SendError::InvalidSocket)
                }
            },
            |e| matches!(e, SendError::WouldBlock),
            self.timeout_config().network,
            "socket_send",
        );

        match result {
            Ok(bytes_sent) => {
                info!(
                    "PID {} sent {} bytes on TCP socket {}",
                    pid, bytes_sent, sockfd
                );
                span.record("bytes_sent", &format!("{}", bytes_sent));
                span.record_result(true);
                match json::to_vec(&serde_json::json!({ "bytes_sent": bytes_sent })) {
                    Ok(result) => SyscallResult::success_with_data(result),
                    Err(e) => {
                        warn!("Failed to serialize send result: {}", e);
                        span.record_error("Serialization failed");
                        SyscallResult::error("Internal serialization error")
                    }
                }
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Send timed out for PID {}, socket {} after {}ms",
                    pid, sockfd, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error("Send timed out")
            }
            Err(TimeoutError::Operation(SendError::NotStream)) => {
                span.record_error("Socket is not a TCP stream");
                SyscallResult::error("Socket is not a TCP stream")
            }
            Err(TimeoutError::Operation(SendError::InvalidSocket)) => {
                span.record_error("Invalid socket or not connected");
                SyscallResult::error("Invalid socket or not connected")
            }
            Err(TimeoutError::Operation(SendError::WouldBlock)) => {
                span.record_error("Send would block");
                SyscallResult::error("Send would block")
            }
            Err(TimeoutError::Operation(SendError::Other(e))) => {
                warn!("Send failed on socket {}: {}", sockfd, e);
                span.record_error(&format!("Send failed: {}", e));
                SyscallResult::error(format!("Send failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn recv(
        &self,
        pid: Pid,
        sockfd: u32,
        size: usize,
        _flags: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_recv");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("requested_size", &format!("{}", size));

        // Receive on existing connection - permissions already checked at connect/accept time

        use std::io::Read;

        #[derive(Debug)]
        enum RecvError {
            WouldBlock,
            NotStream,
            InvalidSocket,
            Other(String),
        }

        let result = self.timeout_executor().execute_with_retry(
            || {
                if let Some(mut socket) = self.socket_manager().sockets.get_mut(&sockfd) {
                    match socket.value_mut() {
                        Socket::TcpStream(stream) => {
                            let mut buffer = vec![0u8; size];
                            match stream.read(&mut buffer) {
                                Ok(bytes_read) => {
                                    buffer.truncate(bytes_read);
                                    Ok(buffer)
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    Err(RecvError::WouldBlock)
                                }
                                Err(e) => Err(RecvError::Other(e.to_string())),
                            }
                        }
                        _ => Err(RecvError::NotStream),
                    }
                } else {
                    Err(RecvError::InvalidSocket)
                }
            },
            |e| matches!(e, RecvError::WouldBlock),
            self.timeout_config().network,
            "socket_recv",
        );

        match result {
            Ok(buffer) => {
                info!(
                    "PID {} received {} bytes on TCP socket {}",
                    pid,
                    buffer.len(),
                    sockfd
                );
                span.record("bytes_received", &format!("{}", buffer.len()));
                span.record_result(true);
                SyscallResult::success_with_data(buffer)
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Recv timed out for PID {}, socket {} after {}ms",
                    pid, sockfd, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error("Recv timed out")
            }
            Err(TimeoutError::Operation(RecvError::NotStream)) => {
                span.record_error("Socket is not a TCP stream");
                SyscallResult::error("Socket is not a TCP stream")
            }
            Err(TimeoutError::Operation(RecvError::InvalidSocket)) => {
                span.record_error("Invalid socket or not connected");
                SyscallResult::error("Invalid socket or not connected")
            }
            Err(TimeoutError::Operation(RecvError::WouldBlock)) => {
                span.record_error("Recv would block");
                SyscallResult::error("Recv would block")
            }
            Err(TimeoutError::Operation(RecvError::Other(e))) => {
                warn!("Recv failed on socket {}: {}", sockfd, e);
                span.record_error(&format!("Recv failed: {}", e));
                SyscallResult::error(format!("Recv failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn sendto(
        &self,
        pid: Pid,
        sockfd: u32,
        data: &[u8],
        address: &str,
        _flags: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_sendto");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("address", address);
        span.record("data_size", &format!("{}", data.len()));

        // Parse host:port from address and check network access
        let parts: Vec<&str> = address.split(':').collect();
        let host = parts.get(0).unwrap_or(&"").to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok());

        let request = PermissionRequest::net_connect(pid, host, port);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try to send via UDP socket
        if let Some(socket) = self.socket_manager().sockets.get(&sockfd) {
            match socket.value() {
                Socket::UdpSocket(udp) => match udp.send_to(data, address) {
                    Ok(bytes_sent) => {
                        info!(
                            "PID {} sent {} bytes to {} on UDP socket {}",
                            pid, bytes_sent, address, sockfd
                        );
                        span.record("bytes_sent", &format!("{}", bytes_sent));
                        span.record_result(true);
                        match json::to_vec(&serde_json::json!({
                            "bytes_sent": bytes_sent
                        })) {
                            Ok(result) => SyscallResult::success_with_data(result),
                            Err(e) => {
                                warn!("Failed to serialize sendto result: {}", e);
                                span.record_error("Serialization failed");
                                SyscallResult::error("Internal serialization error")
                            }
                        }
                    }
                    Err(e) => {
                        warn!("SendTo failed on socket {}: {}", sockfd, e);
                        span.record_error(&format!("SendTo failed: {}", e));
                        SyscallResult::error(format!("SendTo failed: {}", e))
                    }
                },
                _ => {
                    span.record_error("Socket is not a UDP socket");
                    SyscallResult::error("Socket is not a UDP socket")
                }
            }
        } else {
            span.record_error("Invalid UDP socket");
            SyscallResult::error("Invalid UDP socket")
        }
    }

    pub(in crate::syscalls) fn recvfrom(
        &self,
        pid: Pid,
        sockfd: u32,
        size: usize,
        _flags: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_recvfrom");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("requested_size", &format!("{}", size));

        // Receive on UDP socket - permissions should be checked at bind time
        // Could add additional check here if needed

        // Try to receive from UDP socket
        if let Some(socket) = self.socket_manager().sockets.get(&sockfd) {
            match socket.value() {
                Socket::UdpSocket(udp) => {
                    let mut buffer = vec![0u8; size];
                    match udp.recv_from(&mut buffer) {
                        Ok((bytes_read, addr)) => {
                            buffer.truncate(bytes_read);
                            info!(
                                "PID {} received {} bytes from {} on UDP socket {}",
                                pid, bytes_read, addr, sockfd
                            );
                            span.record("bytes_received", &format!("{}", bytes_read));
                            span.record("source_address", &addr.to_string());
                            span.record_result(true);

                            match json::to_vec(&serde_json::json!({
                                "data": buffer,
                                "address": addr.to_string()
                            })) {
                                Ok(result) => SyscallResult::success_with_data(result),
                                Err(e) => {
                                    warn!("Failed to serialize recvfrom result: {}", e);
                                    span.record_error("Serialization failed");
                                    SyscallResult::error("Internal serialization error")
                                }
                            }
                        }
                        Err(e) => {
                            warn!("RecvFrom failed on socket {}: {}", sockfd, e);
                            span.record_error(&format!("RecvFrom failed: {}", e));
                            SyscallResult::error(format!("RecvFrom failed: {}", e))
                        }
                    }
                }
                _ => {
                    span.record_error("Socket is not a UDP socket");
                    SyscallResult::error("Socket is not a UDP socket")
                }
            }
        } else {
            span.record_error("Invalid UDP socket");
            SyscallResult::error("Invalid UDP socket")
        }
    }

    pub(in crate::syscalls) fn close_socket(&self, pid: Pid, sockfd: u32) -> SyscallResult {
        let span = span_operation("socket_close");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));

        // Close doesn't require permission check - closing is always allowed

        // Single lookup in unified collection
        if let Some((_, socket)) = self.socket_manager().sockets.remove(&sockfd) {
            // Untrack socket from process (O(1) with HashSet)
            self.socket_manager().untrack_socket(pid, sockfd);
            // Recycle FD for reuse (lock-free)
            self.socket_manager().recycle_fd(sockfd);

            let type_name = socket.type_name();
            info!(
                "PID {} closed {} socket {} (recycled FD)",
                pid, type_name, sockfd
            );
            span.record("socket_type", type_name);
            span.record_result(true);
            SyscallResult::success()
        } else {
            warn!(
                "PID {} attempted to close non-existent socket {}",
                pid, sockfd
            );
            span.record_error("Invalid socket descriptor");
            SyscallResult::error("Invalid socket descriptor")
        }
    }

    pub(in crate::syscalls) fn setsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
        _optval: &[u8],
    ) -> SyscallResult {
        let span = span_operation("socket_setsockopt");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("level", &format!("{}", level));
        span.record("optname", &format!("{}", optname));

        // Socket options on existing socket - permissions checked at creation time

        warn!(
            "SetSockOpt syscall not fully implemented: sockfd={}, level={}, optname={}",
            sockfd, level, optname
        );
        info!("PID {} set socket option on {}", pid, sockfd);
        span.record_result(true);
        SyscallResult::success()
    }

    pub(in crate::syscalls) fn getsockopt(
        &self,
        pid: Pid,
        sockfd: u32,
        level: u32,
        optname: u32,
    ) -> SyscallResult {
        let span = span_operation("socket_getsockopt");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("sockfd", &format!("{}", sockfd));
        span.record("level", &format!("{}", level));
        span.record("optname", &format!("{}", optname));

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
                span.record_error("Serialization failed");
                return SyscallResult::error("Internal serialization error");
            }
        };

        info!("PID {} got socket option from {}", pid, sockfd);
        span.record_result(true);
        SyscallResult::success_with_data(result)
    }
}
