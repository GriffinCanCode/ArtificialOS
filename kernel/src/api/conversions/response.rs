/*!
 * Response Conversion Utilities
 * Converts internal syscall results to protobuf responses
 */

use crate::syscalls::SyscallResult;
use crate::security::{Capability as SandboxCapability, NetworkRule};
use crate::api::server::grpc_server::kernel_proto::*;

/// Convert internal SyscallResult to protobuf SyscallResponse
pub fn syscall_result_to_proto(result: SyscallResult) -> SyscallResponse {
    match result {
        SyscallResult::Success { data } => SyscallResponse {
            result: Some(syscall_response::Result::Success(SuccessResult {
                data: data.unwrap_or_default(),
            })),
        },
        SyscallResult::Error { message } => SyscallResponse {
            result: Some(syscall_response::Result::Error(ErrorResult { message })),
        },
        SyscallResult::PermissionDenied { reason } => SyscallResponse {
            result: Some(syscall_response::Result::PermissionDenied(
                PermissionDeniedResult { reason },
            )),
        },
    }
}

/// Convert proto capability to internal sandbox capability
pub fn proto_to_sandbox_capability(cap: Capability) -> SandboxCapability {
    match cap {
        Capability::ReadFile => SandboxCapability::ReadFile(None),
        Capability::WriteFile => SandboxCapability::WriteFile(None),
        Capability::CreateFile => SandboxCapability::CreateFile(None),
        Capability::DeleteFile => SandboxCapability::DeleteFile(None),
        Capability::ListDirectory => SandboxCapability::ListDirectory(None),
        Capability::SpawnProcess => SandboxCapability::SpawnProcess,
        Capability::KillProcess => SandboxCapability::KillProcess,
        Capability::NetworkAccess => SandboxCapability::NetworkAccess(NetworkRule::AllowAll),
        Capability::BindPort => SandboxCapability::BindPort(None),
        Capability::SystemInfo => SandboxCapability::SystemInfo,
        Capability::TimeAccess => SandboxCapability::TimeAccess,
        Capability::SendMessage => SandboxCapability::SendMessage,
        Capability::ReceiveMessage => SandboxCapability::ReceiveMessage,
    }
}
