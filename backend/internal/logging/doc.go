// Package logging provides structured logging using uber/zap.
//
// This package offers production-ready logging with two modes:
//   - Production: JSON output for machine parsing
//   - Development: Colored console output for human readability
//
// Log Levels:
//   - Debug: Verbose debugging information
//   - Info: General informational messages
//   - Warn: Warning messages
//   - Error: Error messages
//   - Fatal: Fatal errors (exits process)
//
// Features:
//   - Zero-allocation logging in production
//   - Structured fields for context
//   - Configurable output paths
//   - Automatic log rotation support
//
// Example Usage:
//
//	logger := logging.NewDefault()
//	logger.Info("Server starting", zap.String("port", "8000"))
//	logger.Error("Failed to connect", zap.Error(err))
package logging
