// Package grpc provides gRPC client implementations for kernel and AI services.
//
// This package wraps the generated protobuf clients with connection management,
// error handling, and Go-friendly interfaces.
//
// Clients:
//   - KernelClient: System call execution and process management
//   - AIClient: UI generation and chat streaming
//
// Features:
//   - Automatic connection pooling and reuse
//   - Context-based timeouts and cancellation
//   - Streaming support for real-time operations
//   - Type-safe parameter handling
//
// Example Usage:
//
//	kernel, err := grpc.NewKernelClient("localhost:50051")
//	pid, osPid, err := kernel.CreateProcess(ctx, "my-app", 5, "medium", opts)
//
//	ai, err := grpc.NewAIClient("localhost:50052")
//	stream, err := ai.StreamUI(ctx, "create calculator", context, nil)
package grpc
