// Package main is the entry point for the AI-OS Go backend server.
//
// This application orchestrates the AI-powered operating system backend,
// coordinating between the React frontend, Python AI service, and Rust kernel.
//
// Architecture:
//
//	Frontend (React) → Go Backend → Python AI Service (LLM)
//	                            → Rust Kernel (Syscalls)
//
// The server provides:
//   - REST API for app management
//   - WebSocket streaming for real-time AI
//   - Service provider registry
//   - Session persistence
//   - Rate limiting and security
//
// Configuration:
//   - Environment variables (12-factor)
//   - CLI flags (override env vars)
//   - Defaults for development
//
// Usage:
//
//	# Production mode
//	./server -port 8000 -kernel localhost:50051 -ai localhost:50052
//
//	# Development mode (colored logs, debug level)
//	./server -dev
//
// Signals:
//   - SIGINT, SIGTERM: Graceful shutdown
package main
