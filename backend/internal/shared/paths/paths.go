// Package paths provides standardized filesystem paths for consistent access across the backend.
//
// This package mirrors the kernel's path structure to ensure all components use the same
// directory layout. Any changes here should be synchronized with kernel/src/vfs/paths.rs.
package paths

import (
	"fmt"
	"path/filepath"
)

// Mount points
const (
	Storage = "/storage"
	Tmp     = "/tmp"
	Cache   = "/cache"
)

// Storage subdirectories
const (
	// NativeApps contains prebuilt applications bundled with the OS
	NativeApps = "/storage/native-apps"

	// Apps contains user-generated or AI-generated applications
	Apps = "/storage/apps"

	// User contains user files and data
	User = "/storage/user"

	// System contains system configuration and data
	System = "/storage/system"

	// Lib contains shared libraries and resources
	Lib = "/storage/lib"
)

// User subdirectories
const (
	Documents = "/storage/user/documents"
	Downloads = "/storage/user/downloads"
	Projects  = "/storage/user/projects"
)

// App returns application-specific paths
type App struct {
	ID string
}

// DataDir returns the app's data directory
func (a App) DataDir() string {
	return filepath.Join(Apps, a.ID, "data")
}

// ConfigDir returns the app's config directory
func (a App) ConfigDir() string {
	return filepath.Join(Apps, a.ID, "config")
}

// CacheDir returns the app's cache directory
func (a App) CacheDir() string {
	return filepath.Join(Cache, a.ID)
}

// TempDir returns the app's temp directory
func (a App) TempDir() string {
	return filepath.Join(Tmp, a.ID)
}

// AppPath returns paths for a specific application
func AppPath(appID string) App {
	return App{ID: appID}
}

// IsUserspacePath checks if path is within allowed userspace
func IsUserspacePath(path string) bool {
	return filepath.HasPrefix(path, Apps) ||
		filepath.HasPrefix(path, User) ||
		filepath.HasPrefix(path, Tmp) ||
		filepath.HasPrefix(path, Cache)
}

// IsNativeAppPath checks if path is a native app
func IsNativeAppPath(path string) bool {
	return filepath.HasPrefix(path, NativeApps)
}

// IsSystemPath checks if path is system-protected
func IsSystemPath(path string) bool {
	return filepath.HasPrefix(path, System)
}

// ResolveAppPath resolves a path relative to an app's root
func ResolveAppPath(appID, relativePath string) string {
	appRoot := filepath.Join(Apps, appID)
	return filepath.Join(appRoot, relativePath)
}

// StandardDirectories returns all standard directories that should exist
func StandardDirectories() []string {
	return []string{
		NativeApps,
		Apps,
		User,
		System,
		Lib,
		Documents,
		Downloads,
		Projects,
	}
}

// ValidateAppID checks if an app ID is valid for path construction
func ValidateAppID(appID string) error {
	if appID == "" {
		return fmt.Errorf("app ID cannot be empty")
	}
	if filepath.IsAbs(appID) {
		return fmt.Errorf("app ID cannot be an absolute path")
	}
	if filepath.Clean(appID) != appID {
		return fmt.Errorf("app ID contains invalid path components")
	}
	return nil
}
