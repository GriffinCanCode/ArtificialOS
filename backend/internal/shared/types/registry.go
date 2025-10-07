package types

import "time"

// AppType represents the type of application
type AppType string

const (
	AppTypeBlueprint  AppType = "blueprint"   // Existing JSON/BP apps (prebuilt components)
	AppTypeNativeWeb  AppType = "native_web"  // NEW: TypeScript/React apps (custom components, NO prebuilts)
	AppTypeNativeProc AppType = "native_proc" // NEW: Native OS processes (Python, CLI, etc.)
)

// Package represents an installable/savable app package
type Package struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Icon        string    `json:"icon"`
	Category    string    `json:"category"`
	Version     string    `json:"version"`
	Author      string    `json:"author"`
	Type        AppType   `json:"type"` // NEW: App type discriminator
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
	Services    []string  `json:"services"`
	Permissions []string  `json:"permissions"`
	Tags        []string  `json:"tags"`

	// For blueprint apps only
	Blueprint map[string]interface{} `json:"blueprint,omitempty"`

	// For native web apps (TS/React)
	BundlePath  *string            `json:"bundle_path,omitempty"`  // NEW: JS bundle path
	WebManifest *NativeWebManifest `json:"web_manifest,omitempty"` // NEW: Web app metadata

	// For native process apps (executables)
	ProcManifest *NativeProcManifest `json:"proc_manifest,omitempty"` // NEW: Process app metadata
}

// NativeWebManifest for TypeScript/React apps
type NativeWebManifest struct {
	EntryPoint string        `json:"entry_point"` // e.g., "index.js"
	Exports    NativeExports `json:"exports"`
	DevServer  *string       `json:"dev_server,omitempty"` // For development
}

// NativeExports defines what the native app exports
type NativeExports struct {
	Component string `json:"component"` // Default export name (React component)
}

// NativeProcManifest for native OS process apps
type NativeProcManifest struct {
	Executable string            `json:"executable"`  // Path to executable or command
	Args       []string          `json:"args"`        // Default arguments
	WorkingDir string            `json:"working_dir"` // Working directory
	UIType     string            `json:"ui_type"`     // "terminal", "headless", "custom"
	Env        map[string]string `json:"env"`         // Environment variables
}

// PackageMetadata contains summary information about a package
type PackageMetadata struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Icon        string    `json:"icon"`
	Category    string    `json:"category"`
	Version     string    `json:"version"`
	Author      string    `json:"author"`
	CreatedAt   time.Time `json:"created_at"`
	Tags        []string  `json:"tags"`
}

// ToMetadata extracts metadata from a package
func (p *Package) ToMetadata() PackageMetadata {
	return PackageMetadata{
		ID:          p.ID,
		Name:        p.Name,
		Description: p.Description,
		Icon:        p.Icon,
		Category:    p.Category,
		Version:     p.Version,
		Author:      p.Author,
		CreatedAt:   p.CreatedAt,
		Tags:        p.Tags,
	}
}

// RegistryStats contains registry statistics
type RegistryStats struct {
	TotalPackages int            `json:"total_packages"`
	Categories    map[string]int `json:"categories"`
	LastUpdated   *time.Time     `json:"last_updated,omitempty"`
}
