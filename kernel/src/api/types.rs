/*!
 * API Types
 * Common types for kernel API layer
 */

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// API operation result
pub type ApiResult<T> = Result<T, ApiError>;

/// API errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    /// Invalid request
    InvalidRequest(String),

    /// Service unavailable
    Unavailable(String),

    /// Authentication failed
    AuthenticationFailed(String),

    /// Rate limit exceeded
    RateLimited(String),

    /// Internal server error
    InternalError(String),

    /// Request timeout
    Timeout(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            ApiError::Unavailable(msg) => write!(f, "Service unavailable: {}", msg),
            ApiError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            ApiError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ApiError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub max_connections: usize,
    pub timeout_secs: u64,
    pub keepalive_interval_secs: u64,
    pub keepalive_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:50051".parse().unwrap(),
            max_connections: 1000,
            timeout_secs: 120,
            keepalive_interval_secs: 60,
            keepalive_timeout_secs: 20,
        }
    }
}

impl ServerConfig {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            ..Default::default()
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }
}

/// Request metadata
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub client_addr: Option<SocketAddr>,
    pub request_id: String,
    pub timestamp: std::time::SystemTime,
}

impl RequestMetadata {
    pub fn new(request_id: String) -> Self {
        Self {
            client_addr: None,
            request_id,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn with_client_addr(mut self, addr: SocketAddr) -> Self {
        self.client_addr = Some(addr);
        self
    }
}

/// Response metadata
#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub processing_time_ms: u64,
    pub status: ResponseStatus,
}

/// Response status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStatus {
    Success,
    Error,
    PartialSuccess,
}

/// API statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub active_connections: usize,
}
