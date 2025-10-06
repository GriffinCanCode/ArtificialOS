/*!
 * API Traits
 * Kernel API abstractions
 */

use super::types::*;
use crate::core::types::Pid;
use std::future::Future;
use std::pin::Pin;

/// Async API handler trait
pub trait ApiHandler: Send + Sync {
    type Request;
    type Response;

    /// Handle an API request
    fn handle(
        &self,
        request: Self::Request,
        metadata: RequestMetadata,
    ) -> Pin<Box<dyn Future<Output = ApiResult<Self::Response>> + Send + '_>>;
}

/// Server lifecycle management
pub trait ServerLifecycle: Send + Sync {
    /// Start the server
    fn start(&self) -> Pin<Box<dyn Future<Output = ApiResult<()>> + Send + '_>>;

    /// Stop the server gracefully
    fn stop(&self) -> Pin<Box<dyn Future<Output = ApiResult<()>> + Send + '_>>;

    /// Check if server is running
    fn is_running(&self) -> bool;

    /// Get server configuration
    fn config(&self) -> &ServerConfig;
}

/// Request validation
pub trait RequestValidator: Send + Sync {
    type Request;

    /// Validate an incoming request
    fn validate(&self, request: &Self::Request) -> ApiResult<()>;

    /// Sanitize request data
    fn sanitize(&self, request: Self::Request) -> ApiResult<Self::Request>;
}

/// Response builder
pub trait ResponseBuilder: Send + Sync {
    type Response;

    /// Build a success response
    fn success(data: Vec<u8>) -> Self::Response;

    /// Build an error response
    fn error(message: String) -> Self::Response;

    /// Build a response with metadata
    fn with_metadata(self, metadata: ResponseMetadata) -> Self;
}

/// API metrics collector
pub trait MetricsCollector: Send + Sync {
    /// Record a request
    fn record_request(&self, method: &str, duration_ms: u64, success: bool);

    /// Record an error
    fn record_error(&self, method: &str, error: &ApiError);

    /// Get current statistics
    fn stats(&self) -> ApiStats;

    /// Reset statistics
    fn reset(&self);
}

/// Authentication provider
pub trait AuthProvider: Send + Sync {
    /// Authenticate a request
    fn authenticate(&self, token: &str) -> ApiResult<Pid>; // Returns PID if authenticated

    /// Generate authentication token
    fn generate_token(&self, pid: Pid) -> ApiResult<String>;

    /// Revoke authentication token
    fn revoke_token(&self, token: &str) -> ApiResult<()>;
}

/// Rate limiter
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed
    fn check_rate_limit(&self, client_id: &str) -> ApiResult<()>;

    /// Record a request
    fn record_request(&self, client_id: &str);

    /// Reset rate limit for a client
    fn reset_client(&self, client_id: &str);
}
