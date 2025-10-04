package filesystem

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// BasicOps handles basic file operations
type BasicOps struct {
	*FilesystemOps
}

// GetTools returns basic file operation tool definitions
func (b *BasicOps) GetTools() []types.Tool {
	return []types.Tool{
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
			Description: "Write data to file (overwrites existing)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "string", Description: "Data to write", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.append",
			Name:        "Append to File",
			Description: "Append data to end of file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "string", Description: "Data to append", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.create",
			Name:        "Create File",
			Description: "Create a new empty file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
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
			ID:          "filesystem.exists",
			Name:        "Check Existence",
			Description: "Check if a file or directory exists",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File or directory path", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.read_lines",
			Name:        "Read Lines",
			Description: "Read file as array of lines",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.read_json",
			Name:        "Read JSON",
			Description: "Read and parse JSON file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.write_json",
			Name:        "Write JSON",
			Description: "Write data as JSON file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "object", Description: "Data to write", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.read_binary",
			Name:        "Read Binary File",
			Description: "Read file as binary data",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "bytes",
		},
		{
			ID:          "filesystem.write_binary",
			Name:        "Write Binary File",
			Description: "Write binary data to file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "bytes", Description: "Binary data", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.write_lines",
			Name:        "Write Lines",
			Description: "Write array of lines to file",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "lines", Type: "array", Description: "Lines to write", Required: true},
			},
			Returns: "boolean",
		},
	}
}

// Read reads file contents
func (b *BasicOps) Read(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	data, err := b.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	return Success(map[string]interface{}{
		"path":    path,
		"content": string(data),
		"size":    len(data),
	})
}

// Write writes data to file (overwrites)
func (b *BasicOps) Write(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return Failure("data parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	_, err := b.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": []byte(data),
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{
		"written": true,
		"path":    path,
		"size":    len(data),
	})
}

// Append appends data to file
func (b *BasicOps) Append(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return Failure("data parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	// Read existing content
	existing, err := b.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		// File might not exist, treat as empty
		existing = []byte{}
	}

	// Append and write
	combined := append(existing, []byte(data)...)
	_, err = b.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": combined,
	})
	if err != nil {
		return Failure(fmt.Sprintf("append failed: %v", err))
	}

	return Success(map[string]interface{}{
		"appended": true,
		"path":     path,
		"size":     len(combined),
	})
}

// Create creates a new empty file
func (b *BasicOps) Create(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	_, err := b.Kernel.ExecuteSyscall(ctx, pid, "create_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("create failed: %v", err))
	}

	return Success(map[string]interface{}{"created": true, "path": path})
}

// Delete deletes a file
func (b *BasicOps) Delete(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	_, err := b.Kernel.ExecuteSyscall(ctx, pid, "delete_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("delete failed: %v", err))
	}

	return Success(map[string]interface{}{"deleted": true, "path": path})
}

// Exists checks if file/directory exists
func (b *BasicOps) Exists(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	data, err := b.Kernel.ExecuteSyscall(ctx, pid, "file_exists", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Success(map[string]interface{}{"exists": false, "path": path})
	}

	exists := len(data) > 0 && data[0] == 1

	return Success(map[string]interface{}{"exists": exists, "path": path})
}

// ReadLines reads file as array of lines
func (b *BasicOps) ReadLines(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	data, err := b.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	// Split into lines
	lines := []string{}
	var line []byte
	for _, b := range data {
		if b == '\n' {
			lines = append(lines, string(line))
			line = []byte{}
		} else if b != '\r' { // Skip carriage returns
			line = append(line, b)
		}
	}
	if len(line) > 0 {
		lines = append(lines, string(line))
	}

	return Success(map[string]interface{}{
		"path":  path,
		"lines": lines,
		"count": len(lines),
	})
}

// ReadJSON reads and parses JSON file
func (b *BasicOps) ReadJSON(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	data, err := b.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	// Parse JSON
	var parsed interface{}
	if err := json.Unmarshal(data, &parsed); err != nil {
		return Failure(fmt.Sprintf("invalid JSON: %v", err))
	}

	return Success(map[string]interface{}{
		"path": path,
		"data": parsed,
	})
}

// WriteJSON writes data as JSON file
func (b *BasicOps) WriteJSON(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	data, ok := params["data"]
	if !ok {
		return Failure("data parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	// Serialize to JSON
	jsonData, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return Failure(fmt.Sprintf("JSON serialization failed: %v", err))
	}

	_, err = b.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": jsonData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{
		"written": true,
		"path":    path,
		"size":    len(jsonData),
	})
}

// ReadBinary reads file as binary data (same as Read, returns bytes)
func (b *BasicOps) ReadBinary(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	data, err := b.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	return Success(map[string]interface{}{
		"path": path,
		"data": data,
		"size": len(data),
	})
}

// WriteBinary writes binary data to file
func (b *BasicOps) WriteBinary(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	// Accept data as []byte or base64 string
	var data []byte
	switch v := params["data"].(type) {
	case []byte:
		data = v
	case string:
		// Assume base64 encoded
		decoded, err := json.Marshal(v) // Use JSON to handle any string type
		if err != nil {
			return Failure("invalid data parameter")
		}
		data = decoded
	default:
		return Failure("data parameter must be bytes or string")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	_, err := b.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": data,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{
		"written": true,
		"path":    path,
		"size":    len(data),
	})
}

// WriteLines writes array of lines to file
func (b *BasicOps) WriteLines(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	linesParam, ok := params["lines"]
	if !ok {
		return Failure("lines parameter required")
	}

	// Convert to string array
	var lines []string
	switch v := linesParam.(type) {
	case []string:
		lines = v
	case []interface{}:
		for _, item := range v {
			if str, ok := item.(string); ok {
				lines = append(lines, str)
			} else {
				lines = append(lines, fmt.Sprintf("%v", item))
			}
		}
	default:
		return Failure("lines parameter must be array")
	}

	if b.Kernel == nil {
		return Failure("kernel not available")
	}

	pid := b.GetPID(appCtx)

	// Join lines with newline
	content := ""
	for i, line := range lines {
		if i > 0 {
			content += "\n"
		}
		content += line
	}

	_, err := b.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": []byte(content),
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{
		"written": true,
		"path":    path,
		"lines":   len(lines),
	})
}
