package filesystem

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"syscall"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/charlievieth/fastwalk"
	"github.com/gabriel-vasile/mimetype"
)

// MetadataOps handles file metadata operations
type MetadataOps struct {
	*FilesystemOps
}

// GetTools returns metadata operation tool definitions
func (m *MetadataOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.stat",
			Name:        "File Stats",
			Description: "Get detailed file metadata",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.size",
			Name:        "File Size",
			Description: "Get file size in bytes",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "filesystem.size_human",
			Name:        "Human-Readable Size",
			Description: "Get file size in human-readable format",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "string",
		},
		{
			ID:          "filesystem.total_size",
			Name:        "Directory Size",
			Description: "Calculate total size of directory",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
				{Name: "human", Type: "boolean", Description: "Return human-readable format", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.modified_time",
			Name:        "Modified Time",
			Description: "Get last modified time",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "format", Type: "string", Description: "Format (unix/iso)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.created_time",
			Name:        "Created Time",
			Description: "Get creation time (platform-specific)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "format", Type: "string", Description: "Format (unix/iso)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.accessed_time",
			Name:        "Accessed Time",
			Description: "Get last accessed time",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "format", Type: "string", Description: "Format (unix/iso)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.mime_type",
			Name:        "MIME Type",
			Description: "Detect file MIME type (fast, accurate)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "string",
		},
		{
			ID:          "filesystem.is_text",
			Name:        "Is Text File",
			Description: "Check if file is text",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.is_binary",
			Name:        "Is Binary File",
			Description: "Check if file is binary",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "boolean",
		},
	}
}

// Stat gets file stats
func (m *MetadataOps) Stat(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := m.GetPID(appCtx)
	data, err := m.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	var info map[string]interface{}
	if err := json.Unmarshal(data, &info); err != nil {
		return Failure(fmt.Sprintf("parse error: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "info": info})
}

// Size gets file size
func (m *MetadataOps) Size(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := m.GetPID(appCtx)
	data, err := m.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	var info map[string]interface{}
	if err := json.Unmarshal(data, &info); err != nil {
		return Failure(fmt.Sprintf("parse error: %v", err))
	}

	size, _ := info["size"].(float64)
	return Success(map[string]interface{}{"path": path, "size": int64(size)})
}

// SizeHuman gets human-readable file size
func (m *MetadataOps) SizeHuman(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := m.GetPID(appCtx)
	data, err := m.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	var info map[string]interface{}
	if err := json.Unmarshal(data, &info); err != nil {
		return Failure(fmt.Sprintf("parse error: %v", err))
	}

	size, _ := info["size"].(float64)
	humanSize := formatBytes(int64(size))

	return Success(map[string]interface{}{"path": path, "size": humanSize, "bytes": int64(size)})
}

// TotalSize calculates directory size
func (m *MetadataOps) TotalSize(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	human := false
	if h, ok := params["human"].(bool); ok {
		human = h
	}

	fullPath := m.resolvePath(ctx, path, appCtx)

	var totalSize int64
	fileCount := 0
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		totalSize += info.Size()
		fileCount++
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("size calculation failed: %v", err))
	}

	result := map[string]interface{}{
		"path":  path,
		"bytes": totalSize,
		"files": fileCount,
	}

	if human {
		result["size"] = formatBytes(totalSize)
	}

	return Success(result)
}

// ModifiedTime gets last modified time
func (m *MetadataOps) ModifiedTime(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	format := "unix"
	if f, ok := params["format"].(string); ok {
		format = f
	}

	fullPath := m.resolvePath(ctx, path, appCtx)
	info, err := os.Stat(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	modTime := info.ModTime()
	result := map[string]interface{}{
		"path": path,
		"unix": modTime.Unix(),
	}

	if format == "iso" {
		result["iso"] = modTime.Format(time.RFC3339)
	}

	return Success(result)
}

// CreatedTime gets creation time
func (m *MetadataOps) CreatedTime(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	format := "unix"
	if f, ok := params["format"].(string); ok {
		format = f
	}

	fullPath := m.resolvePath(ctx, path, appCtx)
	info, err := os.Stat(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	// Platform-specific creation time
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !ok {
		return Failure("platform not supported")
	}

	// Darwin/macOS uses Birthtimespec
	var birthTime time.Time
	if stat.Birthtimespec.Sec != 0 {
		birthTime = time.Unix(stat.Birthtimespec.Sec, stat.Birthtimespec.Nsec)
	} else {
		// Fallback to modification time if birth time unavailable
		birthTime = info.ModTime()
	}

	result := map[string]interface{}{
		"path": path,
		"unix": birthTime.Unix(),
	}

	if format == "iso" {
		result["iso"] = birthTime.Format(time.RFC3339)
	}

	return Success(result)
}

// AccessedTime gets last accessed time
func (m *MetadataOps) AccessedTime(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	format := "unix"
	if f, ok := params["format"].(string); ok {
		format = f
	}

	fullPath := m.resolvePath(ctx, path, appCtx)
	info, err := os.Stat(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("stat failed: %v", err))
	}

	stat, ok := info.Sys().(*syscall.Stat_t)
	if !ok {
		return Failure("platform not supported")
	}

	// Darwin/macOS uses Atimespec
	var accessTime time.Time
	if stat.Atimespec.Sec != 0 {
		accessTime = time.Unix(stat.Atimespec.Sec, stat.Atimespec.Nsec)
	} else {
		// Fallback to modification time
		accessTime = info.ModTime()
	}

	result := map[string]interface{}{
		"path": path,
		"unix": accessTime.Unix(),
	}

	if format == "iso" {
		result["iso"] = accessTime.Format(time.RFC3339)
	}

	return Success(result)
}

// MIMEType detects file MIME type
func (m *MetadataOps) MIMEType(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	fullPath := m.resolvePath(ctx, path, appCtx)

	mtype, err := mimetype.DetectFile(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("mime detection failed: %v", err))
	}

	return Success(map[string]interface{}{
		"path":      path,
		"mime_type": mtype.String(),
		"extension": mtype.Extension(),
	})
}

// IsText checks if file is text
func (m *MetadataOps) IsText(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	fullPath := m.resolvePath(ctx, path, appCtx)

	mtype, err := mimetype.DetectFile(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("detection failed: %v", err))
	}

	isText := strings.HasPrefix(mtype.String(), "text/") ||
		mtype.String() == "application/json" ||
		mtype.String() == "application/xml" ||
		mtype.String() == "application/javascript"

	return Success(map[string]interface{}{"path": path, "is_text": isText, "mime_type": mtype.String()})
}

// IsBinary checks if file is binary
func (m *MetadataOps) IsBinary(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	fullPath := m.resolvePath(ctx, path, appCtx)

	mtype, err := mimetype.DetectFile(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("detection failed: %v", err))
	}

	isText := strings.HasPrefix(mtype.String(), "text/") ||
		mtype.String() == "application/json" ||
		mtype.String() == "application/xml" ||
		mtype.String() == "application/javascript"

	return Success(map[string]interface{}{"path": path, "is_binary": !isText, "mime_type": mtype.String()})
}

// resolvePath resolves path through kernel
func (m *MetadataOps) resolvePath(ctx context.Context, path string, appCtx *types.Context) string {
	pid := m.GetPID(appCtx)
	statData, err := m.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return path
	}

	var statInfo map[string]interface{}
	if err := json.Unmarshal(statData, &statInfo); err != nil {
		return path
	}

	if fullPath, ok := statInfo["path"].(string); ok && fullPath != "" {
		return fullPath
	}
	return path
}

// formatBytes formats bytes to human-readable size
func formatBytes(bytes int64) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}

	div, exp := int64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}

	units := []string{"KB", "MB", "GB", "TB", "PB"}
	return fmt.Sprintf("%.2f %s", float64(bytes)/float64(div), units[exp])
}
