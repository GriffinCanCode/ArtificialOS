package filesystem

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/charlievieth/fastwalk"
)

// DirectoryOps handles directory operations
type DirectoryOps struct {
	*FilesystemOps
}

// GetTools returns directory operation tool definitions
func (d *DirectoryOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.dir.list",
			Name:        "List Directory",
			Description: "List contents of a directory",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.dir.create",
			Name:        "Create Directory",
			Description: "Create a new directory (recursive)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.dir.delete",
			Name:        "Delete Directory",
			Description: "Delete directory recursively",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.dir.exists",
			Name:        "Check Directory Exists",
			Description: "Check if directory exists",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.dir.walk",
			Name:        "Walk Directory",
			Description: "Walk directory recursively (3-5x faster)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
				{Name: "max_depth", Type: "number", Description: "Max depth (0=unlimited)", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.dir.tree",
			Name:        "Directory Tree",
			Description: "Get directory tree structure",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
				{Name: "max_depth", Type: "number", Description: "Max depth (0=unlimited)", Required: false},
			},
			Returns: "string",
		},
		{
			ID:          "filesystem.dir.flatten",
			Name:        "Flatten Files",
			Description: "Get all files as flat array (fast)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Directory path", Required: true},
			},
			Returns: "array",
		},
	}
}

// List lists directory contents
func (d *DirectoryOps) List(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := d.GetPID(appCtx)
	data, err := d.Kernel.ExecuteSyscall(ctx, pid, "list_directory", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("list failed: %v", err))
	}

	// Kernel now returns entries with metadata (name, is_dir, type)
	var entries []map[string]interface{}
	if err := json.Unmarshal(data, &entries); err != nil {
		return Failure(fmt.Sprintf("parse error: %v", err))
	}

	// Build response with both entries and files array for backward compatibility
	files := make([]string, len(entries))
	for i, entry := range entries {
		if name, ok := entry["name"].(string); ok {
			files[i] = name
		}
	}

	return Success(map[string]interface{}{
		"path":    path,
		"files":   files,
		"entries": entries,
		"count":   len(entries),
	})
}

// Create creates a directory
func (d *DirectoryOps) Create(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := d.GetPID(appCtx)

	// Log the directory creation attempt
	fmt.Printf("[Filesystem] Creating directory: path=%s, pid=%d\n", path, pid)

	data, err := d.Kernel.ExecuteSyscall(ctx, pid, "create_directory", map[string]interface{}{"path": path})
	if err != nil {
		errMsg := fmt.Sprintf("create failed for path '%s': %v", path, err)
		fmt.Printf("[Filesystem] ERROR: %s\n", errMsg)
		return Failure(errMsg)
	}

	fmt.Printf("[Filesystem] Directory created successfully: path=%s, result=%s\n", path, string(data))

	return Success(map[string]interface{}{"created": true, "path": path})
}

// Delete deletes a directory recursively
func (d *DirectoryOps) Delete(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := d.GetPID(appCtx)

	// Use kernel delete_file which should handle directories
	_, err := d.Kernel.ExecuteSyscall(ctx, pid, "delete_file", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("delete failed: %v", err))
	}

	return Success(map[string]interface{}{"deleted": true, "path": path})
}

// Exists checks if directory exists
func (d *DirectoryOps) Exists(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := d.GetPID(appCtx)
	data, err := d.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Success(map[string]interface{}{"exists": false, "path": path})
	}

	var info map[string]interface{}
	if err := json.Unmarshal(data, &info); err != nil {
		return Success(map[string]interface{}{"exists": false, "path": path})
	}

	isDir, _ := info["is_dir"].(bool)
	return Success(map[string]interface{}{"exists": isDir, "path": path, "is_dir": isDir})
}

// Walk walks directory recursively using fastwalk (3-5x faster)
func (d *DirectoryOps) Walk(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	maxDepth := 0
	if depth, ok := params["max_depth"].(float64); ok {
		maxDepth = int(depth)
	}

	// Resolve path through kernel
	pid := d.GetPID(appCtx)
	statData, err := d.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("path not accessible: %v", err))
	}

	var statInfo map[string]interface{}
	if err := json.Unmarshal(statData, &statInfo); err != nil {
		return Failure("invalid path")
	}

	fullPath, _ := statInfo["path"].(string)
	if fullPath == "" {
		fullPath = path
	}

	entries := []map[string]interface{}{}
	conf := fastwalk.Config{
		Follow: false,
	}

	err = fastwalk.Walk(&conf, fullPath, func(path string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil {
			return nil // Skip errors
		}

		relPath, _ := filepath.Rel(fullPath, path)
		depth := len(strings.Split(relPath, string(os.PathSeparator))) - 1
		if maxDepth > 0 && depth > maxDepth {
			if d.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		entries = append(entries, map[string]interface{}{
			"path":     relPath,
			"is_dir":   d.IsDir(),
			"size":     info.Size(),
			"modified": info.ModTime().Unix(),
		})
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("walk failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "entries": entries, "count": len(entries)})
}

// Tree generates directory tree structure
func (d *DirectoryOps) Tree(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	maxDepth := 0
	if depth, ok := params["max_depth"].(float64); ok {
		maxDepth = int(depth)
	}

	pid := d.GetPID(appCtx)
	statData, err := d.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("path not accessible: %v", err))
	}

	var statInfo map[string]interface{}
	if err := json.Unmarshal(statData, &statInfo); err != nil {
		return Failure("invalid path")
	}

	fullPath, _ := statInfo["path"].(string)
	if fullPath == "" {
		fullPath = path
	}

	var tree strings.Builder
	tree.WriteString(filepath.Base(fullPath) + "/\n")

	conf := fastwalk.Config{Follow: false}
	err = fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || p == fullPath {
			return nil
		}

		relPath, _ := filepath.Rel(fullPath, p)
		depth := len(strings.Split(relPath, string(os.PathSeparator))) - 1
		if maxDepth > 0 && depth > maxDepth {
			if d.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		indent := strings.Repeat("  ", depth+1)
		name := filepath.Base(p)
		if d.IsDir() {
			tree.WriteString(indent + name + "/\n")
		} else {
			tree.WriteString(indent + name + "\n")
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("tree failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "tree": tree.String()})
}

// Flatten gets all files as flat array
func (d *DirectoryOps) Flatten(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := d.GetPID(appCtx)
	statData, err := d.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("path not accessible: %v", err))
	}

	var statInfo map[string]interface{}
	if err := json.Unmarshal(statData, &statInfo); err != nil {
		return Failure("invalid path")
	}

	fullPath, _ := statInfo["path"].(string)
	if fullPath == "" {
		fullPath = path
	}

	files := []string{}
	conf := fastwalk.Config{Follow: false}

	err = fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		relPath, _ := filepath.Rel(fullPath, p)
		files = append(files, relPath)
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("flatten failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "files": files, "count": len(files)})
}
