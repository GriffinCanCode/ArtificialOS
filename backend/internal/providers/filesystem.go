package providers

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Filesystem provides file system operations
type Filesystem struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string
}

// FileInfo represents file metadata
type FileInfo struct {
	Name      string    `json:"name"`
	Path      string    `json:"path"`
	Size      int64     `json:"size"`
	IsDir     bool      `json:"is_dir"`
	Mode      string    `json:"mode"`
	Modified  time.Time `json:"modified"`
	Extension string    `json:"extension,omitempty"`
}

// NewFilesystem creates a filesystem provider
func NewFilesystem(kernel KernelClient, storagePID uint32, storagePath string) *Filesystem {
	return &Filesystem{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}
}

// Definition returns service metadata
func (f *Filesystem) Definition() types.Service {
	return types.Service{
		ID:          "filesystem",
		Name:        "Filesystem Service",
		Description: "File and directory operations with sandboxed access",
		Category:    types.CategoryFilesystem,
		Capabilities: []string{
			"read",
			"write",
			"create",
			"delete",
			"list",
			"stat",
			"move",
			"copy",
		},
		Tools: []types.Tool{
			{
				ID:          "filesystem.list",
				Name:        "List Directory",
				Description: "List contents of a directory",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "Directory path", Required: true},
				},
				Returns: "array",
			},
			{
				ID:          "filesystem.stat",
				Name:        "File Info",
				Description: "Get file or directory metadata",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File or directory path", Required: true},
				},
				Returns: "object",
			},
			{
				ID:          "filesystem.read",
				Name:        "Read File",
				Description: "Read file contents",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File path", Required: true},
				},
				Returns: "string",
			},
			{
				ID:          "filesystem.write",
				Name:        "Write File",
				Description: "Write data to file",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File path", Required: true},
					{Name: "data", Type: "string", Description: "Data to write", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.create",
				Name:        "Create File",
				Description: "Create a new file",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File path", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.mkdir",
				Name:        "Create Directory",
				Description: "Create a new directory",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "Directory path", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.delete",
				Name:        "Delete File",
				Description: "Delete a file or empty directory",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File or directory path", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.move",
				Name:        "Move/Rename",
				Description: "Move or rename a file or directory",
				Parameters: []types.Parameter{
					{Name: "source", Type: "string", Description: "Source path", Required: true},
					{Name: "destination", Type: "string", Description: "Destination path", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.copy",
				Name:        "Copy",
				Description: "Copy a file or directory",
				Parameters: []types.Parameter{
					{Name: "source", Type: "string", Description: "Source path", Required: true},
					{Name: "destination", Type: "string", Description: "Destination path", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "filesystem.exists",
				Name:        "Check Existence",
				Description: "Check if a file or directory exists",
				Parameters: []types.Parameter{
					{Name: "path", Type: "string", Description: "File or directory path", Required: true},
				},
				Returns: "boolean",
			},
		},
	}
}

// Execute runs a filesystem operation
func (f *Filesystem) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "filesystem.list":
		return f.list(ctx, params, appCtx)
	case "filesystem.stat":
		return f.stat(ctx, params, appCtx)
	case "filesystem.read":
		return f.read(ctx, params, appCtx)
	case "filesystem.write":
		return f.write(ctx, params, appCtx)
	case "filesystem.create":
		return f.create(ctx, params, appCtx)
	case "filesystem.mkdir":
		return f.mkdir(ctx, params, appCtx)
	case "filesystem.delete":
		return f.delete(ctx, params, appCtx)
	case "filesystem.move":
		return f.move(ctx, params, appCtx)
	case "filesystem.copy":
		return f.copyFile(ctx, params, appCtx)
	case "filesystem.exists":
		return f.exists(ctx, params, appCtx)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (f *Filesystem) list(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	// Use app's sandbox PID if available, otherwise use storage PID
	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	data, err := f.kernel.ExecuteSyscall(ctx, pid, "list_directory", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("list failed: %v", err))
	}

	// Parse file list
	var files []string
	if err := json.Unmarshal(data, &files); err != nil {
		return failure(fmt.Sprintf("failed to parse result: %v", err))
	}

	return success(map[string]interface{}{
		"path":  path,
		"files": files,
		"count": len(files),
	})
}

func (f *Filesystem) stat(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	data, err := f.kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("stat failed: %v", err))
	}

	// Parse file info
	var info FileInfo
	if err := json.Unmarshal(data, &info); err != nil {
		return failure(fmt.Sprintf("failed to parse result: %v", err))
	}

	return success(map[string]interface{}{
		"name":      info.Name,
		"path":      info.Path,
		"size":      info.Size,
		"is_dir":    info.IsDir,
		"mode":      info.Mode,
		"modified":  info.Modified.Unix(),
		"extension": info.Extension,
	})
}

func (f *Filesystem) read(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	data, err := f.kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("read failed: %v", err))
	}

	return success(map[string]interface{}{
		"path":    path,
		"content": string(data),
		"size":    len(data),
	})
}

func (f *Filesystem) write(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return failure("data parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": []byte(data),
	})
	if err != nil {
		return failure(fmt.Sprintf("write failed: %v", err))
	}

	return success(map[string]interface{}{
		"written": true,
		"path":    path,
		"size":    len(data),
	})
}

func (f *Filesystem) create(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "create_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("create failed: %v", err))
	}

	return success(map[string]interface{}{"created": true, "path": path})
}

func (f *Filesystem) mkdir(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "create_directory", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("mkdir failed: %v", err))
	}

	return success(map[string]interface{}{"created": true, "path": path})
}

func (f *Filesystem) delete(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "delete_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return failure(fmt.Sprintf("delete failed: %v", err))
	}

	return success(map[string]interface{}{"deleted": true, "path": path})
}

func (f *Filesystem) move(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return failure("source parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return failure("destination parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "move_file", map[string]interface{}{
		"source":      source,
		"destination": destination,
	})
	if err != nil {
		return failure(fmt.Sprintf("move failed: %v", err))
	}

	return success(map[string]interface{}{
		"moved":       true,
		"source":      source,
		"destination": destination,
	})
}

func (f *Filesystem) copyFile(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return failure("source parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return failure("destination parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	_, err := f.kernel.ExecuteSyscall(ctx, pid, "copy_file", map[string]interface{}{
		"source":      source,
		"destination": destination,
	})
	if err != nil {
		return failure(fmt.Sprintf("copy failed: %v", err))
	}

	return success(map[string]interface{}{
		"copied":      true,
		"source":      source,
		"destination": destination,
	})
}

func (f *Filesystem) exists(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return failure("path parameter required")
	}

	if f.kernel == nil {
		return failure("kernel not available")
	}

	pid := f.storagePID
	if appCtx != nil && appCtx.SandboxPID != nil {
		pid = *appCtx.SandboxPID
	}

	data, err := f.kernel.ExecuteSyscall(ctx, pid, "file_exists", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return success(map[string]interface{}{"exists": false, "path": path})
	}

	exists := len(data) > 0 && data[0] == 1

	return success(map[string]interface{}{"exists": exists, "path": path})
}
