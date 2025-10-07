/*!
 * API Module
 * External interfaces (gRPC, etc.)
 */

pub mod async_task;
pub mod batch;
pub mod conversions;
pub mod grpc_server;
pub mod handlers;
pub mod metrics;
pub mod streaming;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use grpc_server::{start_grpc_server, GrpcServer};
pub use metrics::MetricsService;
pub use traits::*;
pub use types::*;
