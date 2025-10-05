// Package providers implements the service provider system for AI-OS.
//
// Service providers expose capabilities to AI-powered applications through
// a standardized tool-based interface. Each provider implements filesystem,
// network, computation, or storage operations.
//
// Available Providers:
//   - Filesystem: File operations, directories, archives, formats
//   - HTTP: Web requests, downloads, uploads, parsing
//   - Math: Arithmetic, trigonometry, statistics, precision
//   - Scraper: HTML parsing, content extraction, XPath queries
//   - Storage: Key-value persistence per application
//   - Auth: User authentication and session management
//   - System: System information and logging
//
// Provider Interface:
//   - Definition(): Returns service metadata and tool definitions
//   - Execute(): Executes a tool with parameters and context
//
// Example Usage:
//
//	fs := providers.NewFilesystem(kernel, storagePID, storagePath)
//	result, err := fs.Execute(ctx, "fs.read", params, appCtx)
package providers
