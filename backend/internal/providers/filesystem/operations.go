package filesystem

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// OperationsOps handles file operations (copy, move, rename, links)
type OperationsOps struct {
	*FilesystemOps
}

// GetTools returns file operation tool definitions
func (o *OperationsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.copy",
			Name:        "Copy File",
			Description: "Copy file or directory",
			Parameters: []types.Parameter{
				{Name: "source", Type: "string", Description: "Source path", Required: true},
				{Name: "destination", Type: "string", Description: "Destination path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.move",
			Name:        "Move File",
			Description: "Move or rename file/directory",
			Parameters: []types.Parameter{
				{Name: "source", Type: "string", Description: "Source path", Required: true},
				{Name: "destination", Type: "string", Description: "Destination path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.rename",
			Name:        "Rename File",
			Description: "Rename file or directory",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "new_name", Type: "string", Description: "New name", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.symlink",
			Name:        "Create Symlink",
			Description: "Create symbolic link",
			Parameters: []types.Parameter{
				{Name: "target", Type: "string", Description: "Target path", Required: true},
				{Name: "link", Type: "string", Description: "Symlink path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.readlink",
			Name:        "Read Symlink",
			Description: "Read symlink target path",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Symlink path", Required: true},
			},
			Returns: "string",
		},
		{
			ID:          "filesystem.hardlink",
			Name:        "Create Hardlink",
			Description: "Create hard link (same filesystem only)",
			Parameters: []types.Parameter{
				{Name: "target", Type: "string", Description: "Target path", Required: true},
				{Name: "link", Type: "string", Description: "Hardlink path", Required: true},
			},
			Returns: "boolean",
		},
	}
}

// Copy copies a file
func (o *OperationsOps) Copy(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return Failure("source parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return Failure("destination parameter required")
	}

	pid := o.GetPID(appCtx)

	_, err := o.Kernel.ExecuteSyscall(ctx, pid, "copy_file", map[string]interface{}{
		"source":      source,
		"destination": destination,
	})
	if err != nil {
		return Failure(fmt.Sprintf("copy failed: %v", err))
	}

	return Success(map[string]interface{}{"copied": true, "source": source, "destination": destination})
}

// Move moves a file
func (o *OperationsOps) Move(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return Failure("source parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return Failure("destination parameter required")
	}

	pid := o.GetPID(appCtx)

	_, err := o.Kernel.ExecuteSyscall(ctx, pid, "move_file", map[string]interface{}{
		"source":      source,
		"destination": destination,
	})
	if err != nil {
		return Failure(fmt.Sprintf("move failed: %v", err))
	}

	return Success(map[string]interface{}{"moved": true, "source": source, "destination": destination})
}

// Rename renames a file
func (o *OperationsOps) Rename(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	newName, ok := params["new_name"].(string)
	if !ok || newName == "" {
		return Failure("new_name parameter required")
	}

	// Construct new path in same directory
	dir := filepath.Dir(path)
	newPath := filepath.Join(dir, newName)

	pid := o.GetPID(appCtx)

	_, err := o.Kernel.ExecuteSyscall(ctx, pid, "move_file", map[string]interface{}{
		"source":      path,
		"destination": newPath,
	})
	if err != nil {
		return Failure(fmt.Sprintf("rename failed: %v", err))
	}

	return Success(map[string]interface{}{"renamed": true, "old_path": path, "new_path": newPath})
}

// Symlink creates a symbolic link
func (o *OperationsOps) Symlink(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	target, ok := params["target"].(string)
	if !ok || target == "" {
		return Failure("target parameter required")
	}

	link, ok := params["link"].(string)
	if !ok || link == "" {
		return Failure("link parameter required")
	}

	// Resolve paths
	fullTarget := o.resolvePath(ctx, target, appCtx)
	fullLink := o.resolvePath(ctx, filepath.Dir(link), appCtx)
	fullLink = filepath.Join(fullLink, filepath.Base(link))

	// Create symlink directly (may need kernel syscall in future for sandboxing)
	if err := os.Symlink(fullTarget, fullLink); err != nil {
		return Failure(fmt.Sprintf("symlink failed: %v", err))
	}

	return Success(map[string]interface{}{"created": true, "target": target, "link": link})
}

// Readlink reads a symlink target
func (o *OperationsOps) Readlink(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	fullPath := o.resolvePath(ctx, path, appCtx)

	target, err := os.Readlink(fullPath)
	if err != nil {
		return Failure(fmt.Sprintf("readlink failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "target": target})
}

// Hardlink creates a hard link
func (o *OperationsOps) Hardlink(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	target, ok := params["target"].(string)
	if !ok || target == "" {
		return Failure("target parameter required")
	}

	link, ok := params["link"].(string)
	if !ok || link == "" {
		return Failure("link parameter required")
	}

	fullTarget := o.resolvePath(ctx, target, appCtx)
	fullLink := o.resolvePath(ctx, filepath.Dir(link), appCtx)
	fullLink = filepath.Join(fullLink, filepath.Base(link))

	if err := os.Link(fullTarget, fullLink); err != nil {
		return Failure(fmt.Sprintf("hardlink failed: %v", err))
	}

	return Success(map[string]interface{}{"created": true, "target": target, "link": link})
}

// resolvePath resolves path through kernel
func (o *OperationsOps) resolvePath(ctx context.Context, path string, appCtx *types.Context) string {
	pid := o.GetPID(appCtx)
	statData, err := o.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
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
