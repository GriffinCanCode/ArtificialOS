// Package config provides 12-factor configuration management for the AI-OS backend.
//
// Configuration is loaded from environment variables with sensible defaults.
// CLI flags can override environment variables for development flexibility.
//
// Configuration Sections:
//   - Server: HTTP server settings (port, host)
//   - Kernel: Kernel gRPC connection settings
//   - AI: AI service gRPC connection settings
//   - Logging: Log level and output format
//   - RateLimit: Per-IP rate limiting configuration
//
// Example Usage:
//
//	cfg := config.LoadOrDefault()
//	fmt.Printf("Server running on %s:%s\n", cfg.Server.Host, cfg.Server.Port)
//
// Environment Variables:
//   - PORT, HOST, KERNEL_ADDR, AI_ADDR
//   - LOG_LEVEL, LOG_DEV
//   - RATE_LIMIT_RPS, RATE_LIMIT_BURST
package config
