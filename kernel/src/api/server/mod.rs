/*!
 * Server Module
 * gRPC server and metrics components
 */

pub mod grpc_server;
pub mod metrics;

pub use grpc_server::{kernel_proto, start_grpc_server, GrpcServer, KernelServiceImpl};
pub use metrics::MetricsService;
