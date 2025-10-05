// Package kernel provides generated Protocol Buffer types and gRPC clients for kernel communication.
//
// Generated from: proto/kernel.proto
//
// This package contains:
//   - KernelServiceClient: gRPC client for kernel operations
//   - Syscall request/response types
//   - Process management types
//   - Sandbox configuration types
//   - Event streaming types
//
// Services:
//   - ExecuteSyscall: Execute file/system operations
//   - CreateProcess: Create sandboxed processes
//   - UpdateSandbox: Modify sandbox permissions
//   - StreamEvents: Subscribe to kernel events
//
// Usage:
//
//	This package is typically wrapped by internal/grpc/kernel.go
//	for higher-level Go interfaces.
//
// Note: This is generated code. Do not edit manually.
// Regenerate with: make proto
package kernel
