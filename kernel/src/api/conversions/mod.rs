/*!
 * Conversion utilities for gRPC protocol buffers
 */

pub mod proto;
pub mod response;

pub use proto::{proto_to_syscall_full, proto_to_syscall_simple};
pub use response::{proto_to_sandbox_capability, syscall_result_to_proto};
