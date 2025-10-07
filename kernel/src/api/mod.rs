/*!
 * API Module
 * External interfaces (gRPC, etc.)
 */

pub mod conversions;
pub mod execution;
pub mod handlers;
pub mod server;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use execution::{AsyncTaskManager, BatchExecutor, StreamingManager, TaskStatus};
pub use server::{start_grpc_server, GrpcServer, MetricsService};
pub use traits::*;
pub use types::*;
