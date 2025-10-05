// Package ai provides generated Protocol Buffer types and gRPC clients for AI service communication.
//
// Generated from: proto/ai.proto
//
// This package contains:
//   - AIServiceClient: gRPC client for AI operations
//   - UI generation request/response types
//   - Chat request/response types
//   - Token streaming types (UI and Chat)
//
// Services:
//   - GenerateUI: Non-streaming UI generation
//   - StreamUI: Streaming UI generation with thoughts
//   - StreamChat: Streaming chat with token-by-token response
//
// Token Types:
//   - THOUGHT: AI reasoning process
//   - TOKEN: Response content token
//   - PREVIEW: UI preview during generation
//   - COMPLETE: Generation finished
//   - ERROR: Generation error
//
// Usage:
//
//	This package is typically wrapped by internal/grpc/ai.go
//	for higher-level Go interfaces.
//
// Note: This is generated code. Do not edit manually.
// Regenerate with: make proto
package ai
