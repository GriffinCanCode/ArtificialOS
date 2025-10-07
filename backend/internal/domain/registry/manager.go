package registry

import (
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"
	"sync/atomic"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelClient interface for file operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

const (
	// MaxRegistryCacheSize defines the maximum number of packages in the registry cache
	MaxRegistryCacheSize = 1000
	// RegistryCacheEvictionThreshold defines when to trigger eviction (90% of max)
	RegistryCacheEvictionThreshold = 900
)

// Manager handles app registry persistence
type Manager struct {
	packages    sync.Map
	cacheSize   int64 // Atomic counter for cache size
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	evictionMu  sync.Mutex // Dedicated lock for eviction to prevent race conditions
	evicting    int32      // Atomic flag to prevent concurrent evictions
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
func (m *Manager) Save(ctx context.Context, pkg *types.Package) error {
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
		_, err = m.kernel.ExecuteSyscall(ctx, m.storagePID, "write_file", params)
		if err != nil {
			return fmt.Errorf("failed to write package: %w", err)
		}
	}

	// Cache in memory with size limit enforcement
	_, existed := m.packages.Load(pkg.ID)
	m.packages.Store(pkg.ID, pkg)

	// Increment cache size if new entry
	if !existed {
		newSize := atomic.AddInt64(&m.cacheSize, 1)
		// Evict oldest entries if cache is too large
		if newSize > RegistryCacheEvictionThreshold {
			m.evictCacheEntries()
		}
	}

	return nil
}

// Load loads a package from the registry
func (m *Manager) Load(ctx context.Context, id string) (*types.Package, error) {
	// Check cache first
	if cached, ok := m.packages.Load(id); ok {
		return cached.(*types.Package), nil
	}

	// Load from filesystem
	path := m.packagePath(id)
	params := map[string]interface{}{
		"path": path,
	}

	data, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "read_file", params)
	if err != nil {
		return nil, fmt.Errorf("failed to read package: %w", err)
	}

	// Unmarshal and validate
	var pkg types.Package
	if err := json.Unmarshal(data, &pkg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal package %s: %w", id, err)
	}

	// Basic validation
	if pkg.ID == "" {
		return nil, fmt.Errorf("package %s has empty ID field", id)
	}

	// Cache with size limit enforcement
	_, existed := m.packages.Load(id)
	m.packages.Store(id, &pkg)

	// Increment cache size if new entry
	if !existed {
		newSize := atomic.AddInt64(&m.cacheSize, 1)
		// Evict oldest entries if cache is too large
		if newSize > RegistryCacheEvictionThreshold {
			m.evictCacheEntries()
		}
	}

	return &pkg, nil
}

// List lists all packages, optionally filtered by category
func (m *Manager) List(category *string) ([]*types.Package, error) {
	// Scan filesystem directory for all .aiapp files
	appsDir := filepath.Join(m.storagePath, "apps")

	if m.kernel != nil {
		// Use kernel to list directory
		ctx := context.Background()
		data, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "list_directory", map[string]interface{}{
			"path": appsDir,
		})

		if err == nil {
			// Parse directory listing
			var entries []string
			if err := json.Unmarshal(data, &entries); err == nil {
				// Load all .aiapp files found
				for _, entry := range entries {
					if filepath.Ext(entry) == ".aiapp" {
						pkgID := entry[:len(entry)-7] // Remove ".aiapp" extension
						// Load if not in cache
						if _, ok := m.packages.Load(pkgID); !ok {
							_, _ = m.Load(ctx, pkgID) // Ignore errors for individual files
						}
					}
				}
			}
		}
	}

	// Return cached packages (now includes filesystem scan results)
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
func (m *Manager) Delete(ctx context.Context, id string) error {
	// Delete from filesystem
	path := m.packagePath(id)
	params := map[string]interface{}{
		"path": path,
	}

	if m.kernel != nil {
		_, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "delete_file", params)
		if err != nil {
			return fmt.Errorf("failed to delete package: %w", err)
		}
	}

	// Remove from cache
	if _, existed := m.packages.Load(id); existed {
		m.packages.Delete(id)
		atomic.AddInt64(&m.cacheSize, -1)
	}

	return nil
}

// Exists checks if a package exists
func (m *Manager) Exists(ctx context.Context, id string) bool {
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
		_, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "file_exists", params)
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

// evictCacheEntries removes entries when cache grows too large
// Uses a single-threaded eviction approach to prevent race conditions
func (m *Manager) evictCacheEntries() {
	// Use atomic CAS to ensure only one goroutine performs eviction
	if !atomic.CompareAndSwapInt32(&m.evicting, 0, 1) {
		return // Another goroutine is already evicting
	}
	defer atomic.StoreInt32(&m.evicting, 0)

	// Double-check size after acquiring eviction lock
	currentSize := atomic.LoadInt64(&m.cacheSize)
	if currentSize <= RegistryCacheEvictionThreshold {
		return
	}

	targetEvictions := currentSize - RegistryCacheEvictionThreshold + 100 // Remove extra for headroom
	evicted := int64(0)

	// Simple eviction: remove entries until we hit target
	// Note: sync.Map doesn't maintain insertion order, so this is pseudo-random
	m.packages.Range(func(key, _ interface{}) bool {
		if evicted >= targetEvictions {
			return false // Stop iteration
		}
		m.packages.Delete(key)
		evicted++
		return true
	})

	// Update counter
	atomic.AddInt64(&m.cacheSize, -evicted)
}

// packagePath generates the filesystem path for a package
func (m *Manager) packagePath(id string) string {
	return filepath.Join(m.storagePath, "apps", id+".aiapp")
}
