/*!
 * Network Syscalls
 * Socket and network operations
 */

use crate::core::types::{Size, SockFd};
use serde::{Deserialize, Serialize};

/// Network operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
#[allow(dead_code)]
pub enum NetworkSyscall {
    /// Make HTTP network request
    NetworkRequest {
        /// URL to fetch
        url: String,
    },

    /// Create socket
    Socket {
        /// Address family (AF_INET, AF_INET6, etc.)
        domain: u32,
        /// Socket type (SOCK_STREAM, SOCK_DGRAM, etc.)
        socket_type: u32,
        /// Protocol (IPPROTO_TCP, IPPROTO_UDP, etc.)
        #[serde(default)]
        protocol: u32,
    },

    /// Bind socket to address
    Bind {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Address in IP:port format
        address: String,
    },

    /// Listen on socket
    Listen {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Connection backlog
        #[serde(default)]
        backlog: u32,
    },

    /// Accept connection on socket
    Accept {
        /// Socket file descriptor
        sockfd: SockFd,
    },

    /// Connect socket to address
    Connect {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Address in IP:port format
        address: String,
    },

    /// Send data on socket
    Send {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Data to send
        data: Vec<u8>,
        /// Send flags
        #[serde(default)]
        flags: u32,
    },

    /// Receive data from socket
    Recv {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Maximum bytes to receive
        size: Size,
        /// Receive flags
        #[serde(default)]
        flags: u32,
    },

    /// Send data to specific address (UDP)
    SendTo {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Data to send
        data: Vec<u8>,
        /// Destination address
        address: String,
        /// Send flags
        #[serde(default)]
        flags: u32,
    },

    /// Receive data with source address (UDP)
    RecvFrom {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Maximum bytes to receive
        size: Size,
        /// Receive flags
        #[serde(default)]
        flags: u32,
    },

    /// Close socket
    CloseSocket {
        /// Socket file descriptor
        sockfd: SockFd,
    },

    /// Set socket option
    SetSockOpt {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Protocol level
        level: u32,
        /// Option name
        optname: u32,
        /// Option value
        optval: Vec<u8>,
    },

    /// Get socket option
    GetSockOpt {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Protocol level
        level: u32,
        /// Option name
        optname: u32,
    },
}
