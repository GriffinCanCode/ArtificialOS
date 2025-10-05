// Package filesystem provides comprehensive file system operations for AI-OS.
//
// This package is organized into specialized modules:
//   - basic: Core file operations (read, write, create, delete)
//   - directory: Directory operations (list, create, walk, tree)
//   - operations: File manipulation (copy, move, rename, links)
//   - metadata: File metadata and statistics
//   - search: File search and filtering (glob, content, recent)
//   - formats: Structured formats (JSON, YAML, CSV, TOML)
//   - archives: Archive operations (ZIP, TAR with compression)
//
// All operations:
//   - Respect app sandbox boundaries
//   - Support both app-scoped and storage-scoped paths
//   - Provide detailed error messages
//   - Return structured JSON results
//
// Context Resolution:
//   - With AppContext: Operations scoped to app's sandbox
//   - Without AppContext: Operations use global storage PID
//
// Example Usage:
//
//	basic := &filesystem.BasicOps{FilesystemOps: ops}
//	result, err := basic.Read(ctx, params, appCtx)
package filesystem
