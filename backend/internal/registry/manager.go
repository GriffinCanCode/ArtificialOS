package registry

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// KernelClient interface for file operations
type KernelClient interface {
	ExecuteSyscall(pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Manager handles app registry persistence
type Manager struct {
	packages    sync.Map
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	mu          sync.RWMutex
}

// NewManager creates a new registry manager
func NewManager(kernel KernelClient, storagePID uint32, storagePath string) *Manager {
	return &Manager{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}
}

// Save saves a package to the registry
func (m *Manager) Save(pkg *types.Package) error {
	if pkg.ID == "" {
		return fmt.Errorf("package ID is required")
	}

	// Update timestamp
	pkg.UpdatedAt = time.Now()
	if pkg.CreatedAt.IsZero() {
		pkg.CreatedAt = time.Now()
	}

	// Marshal to JSON
	data, err := json.Marshal(pkg)
	if err != nil {
		return fmt.Errorf("failed to marshal package: %w", err)
	}

	// Write to filesystem via kernel
	path := m.packagePath(pkg.ID)
	params := map[string]interface{}{
		"path": path,
		"data": data,
	}

	if m.kernel != nil {
		_, err = m.kernel.ExecuteSyscall(m.storagePID, "write_file", params)
		if err != nil {
			return fmt.Errorf("failed to write package: %w", err)
		}
	}

	// Cache in memory
	m.packages.Store(pkg.ID, pkg)

	return nil
}

// Load loads a package from the registry
func (m *Manager) Load(id string) (*types.Package, error) {
	// Check cache first
	if cached, ok := m.packages.Load(id); ok {
		return cached.(*types.Package), nil
	}

	// Load from filesystem
	path := m.packagePath(id)
	params := map[string]interface{}{
		"path": path,
	}

	data, err := m.kernel.ExecuteSyscall(m.storagePID, "read_file", params)
	if err != nil {
		return nil, fmt.Errorf("failed to read package: %w", err)
	}

	// Unmarshal
	var pkg types.Package
	if err := json.Unmarshal(data, &pkg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal package: %w", err)
	}

	// Cache
	m.packages.Store(id, &pkg)

	return &pkg, nil
}

// List lists all packages, optionally filtered by category
func (m *Manager) List(category *string) ([]*types.Package, error) {
	// For now, return cached packages
	// TODO: Scan filesystem directory for all .aiapp files
	var packages []*types.Package

	m.packages.Range(func(_, value interface{}) bool {
		pkg := value.(*types.Package)
		if category == nil || pkg.Category == *category {
			packages = append(packages, pkg)
		}
		return true
	})

	return packages, nil
}

// ListMetadata lists metadata for all packages
func (m *Manager) ListMetadata(category *string) ([]types.PackageMetadata, error) {
	packages, err := m.List(category)
	if err != nil {
		return nil, err
	}

	metadata := make([]types.PackageMetadata, len(packages))
	for i, pkg := range packages {
		metadata[i] = pkg.ToMetadata()
	}

	return metadata, nil
}

// Delete removes a package from the registry
func (m *Manager) Delete(id string) error {
	// Delete from filesystem
	path := m.packagePath(id)
	params := map[string]interface{}{
		"path": path,
	}

	if m.kernel != nil {
		_, err := m.kernel.ExecuteSyscall(m.storagePID, "delete_file", params)
		if err != nil {
			return fmt.Errorf("failed to delete package: %w", err)
		}
	}

	// Remove from cache
	m.packages.Delete(id)

	return nil
}

// Exists checks if a package exists
func (m *Manager) Exists(id string) bool {
	// Check cache
	if _, ok := m.packages.Load(id); ok {
		return true
	}

	// Check filesystem
	path := m.packagePath(id)
	params := map[string]interface{}{
		"path": path,
	}

	if m.kernel != nil {
		_, err := m.kernel.ExecuteSyscall(m.storagePID, "file_exists", params)
		return err == nil
	}

	return false
}

// Stats returns registry statistics
func (m *Manager) Stats() types.RegistryStats {
	var total int
	categories := make(map[string]int)
	var lastUpdated *time.Time

	m.packages.Range(func(_, value interface{}) bool {
		pkg := value.(*types.Package)
		total++
		categories[pkg.Category]++

		if lastUpdated == nil || pkg.UpdatedAt.After(*lastUpdated) {
			lastUpdated = &pkg.UpdatedAt
		}

		return true
	})

	return types.RegistryStats{
		TotalPackages: total,
		Categories:    categories,
		LastUpdated:   lastUpdated,
	}
}

// packagePath generates the filesystem path for a package
func (m *Manager) packagePath(id string) string {
	return filepath.Join(m.storagePath, "apps", id+".aiapp")
}
