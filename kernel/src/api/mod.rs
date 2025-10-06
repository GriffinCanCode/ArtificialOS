/*!
 * API Module
 * External interfaces (gRPC, etc.)
 */

pub mod grpc_server;

// Re-export for convenience
pub use grpc_server::start_grpc_server;
