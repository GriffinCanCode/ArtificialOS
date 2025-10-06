package files

import (
	"context"
	"fmt"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"os"
	"path/filepath"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// DownloadsOps handles file downloads
type DownloadsOps struct {
	*client.HTTPOps
}

// GetTools returns download tool definitions
func (d *DownloadsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.download",
			Name:        "Download File",
			Description: "Download file from URL to local path",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "File URL", Required: true},
				{Name: "path", Type: "string", Description: "Save location", Required: true},
				{Name: "filename", Type: "string", Description: "Optional filename override", Required: false},
				{Name: "create_dirs", Type: "boolean", Description: "Create parent directories (default: true)", Required: false},
			},
			Returns: "object",
		},
	}
}

// Download downloads file from URL with automatic retry and error handling
func (d *DownloadsOps) Download(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	path, err := client.GetString(params, "path", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Check if filename override provided
	filename, _ := client.GetString(params, "filename", false)
	if filename != "" {
		// If path is a directory, append filename
		if stat, err := os.Stat(path); err == nil && stat.IsDir() {
			path = filepath.Join(path, filename)
		} else {
			// Use the directory part and new filename
			path = filepath.Join(filepath.Dir(path), filename)
		}
	}

	// Check for context cancellation before starting
	select {
	case <-ctx.Done():
		return client.Failure(fmt.Sprintf("download cancelled: %v", ctx.Err()))
	default:
	}

	// Create parent directories if needed
	createDirs := client.GetBool(params, "create_dirs", true)
	if createDirs {
		dir := filepath.Dir(path)
		if err := os.MkdirAll(dir, 0755); err != nil {
			return client.Failure(fmt.Sprintf("failed to create directory: %v", err))
		}
	}

	// Create request with rate limiting
	req, err := d.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Set output file
	req.SetOutput(path)

	// Execute download
	resp, err := req.Get(urlStr)
	if err != nil {
		// Clean up partial download on error
		os.Remove(path)
		return client.Failure(fmt.Sprintf("download failed: %v", err))
	}

	// Check HTTP status
	if resp.StatusCode() < 200 || resp.StatusCode() >= 300 {
		os.Remove(path)
		return client.Failure(fmt.Sprintf("download failed: HTTP %d", resp.StatusCode()))
	}

	// Get file info
	stat, err := os.Stat(path)
	if err != nil {
		return client.Failure(fmt.Sprintf("failed to stat downloaded file: %v", err))
	}

	return client.Success(map[string]interface{}{
		"downloaded":   true,
		"path":         path,
		"size":         stat.Size(),
		"status":       resp.StatusCode(),
		"time_ms":      resp.Time().Milliseconds(),
		"content_type": resp.Header().Get("Content-Type"),
	})
}
