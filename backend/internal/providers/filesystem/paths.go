package filesystem

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/paths"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// resolvePath resolves a path to an absolute VFS path
// Handles both app-scoped paths (when appCtx exists) and storage-scoped paths
func resolvePath(path string, appCtx *types.Context) string {
	if appCtx != nil && appCtx.AppID != nil && *appCtx.AppID != "" {
		// App-scoped: /storage/apps/{app_id}/{path}
		if filepath.IsAbs(path) {
			// Already absolute, use as-is (allows cross-app access if permitted)
			return filepath.Clean(path)
		}
		// Relative path: scope to app's directory using standard paths
		return paths.ResolveAppPath(*appCtx.AppID, path)
	}

	// Storage-scoped: /storage/{path}
	if filepath.IsAbs(path) {
		return filepath.Clean(path)
	}
	return filepath.Join(paths.Storage, path)
}

// validateUserspacePath ensures path is within allowed userspace
func validateUserspacePath(path string) error {
	// Normalize path
	cleanPath := filepath.Clean(path)

	// Check if within userspace
	if !paths.IsUserspacePath(cleanPath) {
		return fmt.Errorf("path %s is outside allowed userspace", path)
	}

	// Prevent directory traversal attacks
	if strings.Contains(cleanPath, "..") {
		return fmt.Errorf("path cannot contain .. components")
	}

	return nil
}

// ensureAppDirectory creates an app's directory structure if it doesn't exist
func (ops *FilesystemOps) ensureAppDirectory(appID string) error {
	// This would call the kernel to create directories
	// For now, just validate the app ID
	if err := paths.ValidateAppID(appID); err != nil {
		return fmt.Errorf("invalid app ID: %w", err)
	}
	return nil
}
