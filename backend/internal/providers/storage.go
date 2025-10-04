package providers

import (
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"
	"sync/atomic"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

const (
	// MaxCacheSize defines the maximum number of items in the storage cache
	MaxCacheSize = 10000
	// CacheEvictionThreshold defines when to trigger eviction (90% of max)
	CacheEvictionThreshold = 9000
)

// Storage provides persistent key-value storage per app
type Storage struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	cache       sync.Map
	cacheSize   int64 // Atomic counter for cache size
	evicting    int32 // Atomic flag to prevent concurrent evictions
}

// NewStorage creates a storage provider
func NewStorage(kernel KernelClient, storagePID uint32, storagePath string) *Storage {
	return &Storage{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}
}

// Definition returns service metadata
func (s *Storage) Definition() types.Service {
	return types.Service{
		ID:          "storage",
		Name:        "Storage Service",
		Description: "Persistent key-value storage for applications",
		Category:    types.CategoryStorage,
		Capabilities: []string{
			"read",
			"write",
			"delete",
			"list",
		},
		Tools: []types.Tool{
			{
				ID:          "storage.set",
				Name:        "Set Value",
				Description: "Store a value by key",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Storage key", Required: true},
					{Name: "value", Type: "any", Description: "Value to store", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "storage.get",
				Name:        "Get Value",
				Description: "Retrieve a value by key",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Storage key", Required: true},
				},
				Returns: "any",
			},
			{
				ID:          "storage.remove",
				Name:        "Remove Value",
				Description: "Delete a value by key",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Storage key", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "storage.list",
				Name:        "List Keys",
				Description: "List all storage keys for this app",
				Parameters:  []types.Parameter{},
				Returns:     "array",
			},
			{
				ID:          "storage.clear",
				Name:        "Clear All",
				Description: "Remove all storage for this app",
				Parameters:  []types.Parameter{},
				Returns:     "boolean",
			},
		},
	}
}

// Execute runs a storage operation
func (s *Storage) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	if appCtx == nil || appCtx.AppID == nil {
		return failure("app context required for storage operations")
	}

	switch toolID {
	case "storage.set":
		return s.set(ctx, *appCtx.AppID, params)
	case "storage.get":
		return s.get(ctx, *appCtx.AppID, params)
	case "storage.remove":
		return s.remove(ctx, *appCtx.AppID, params)
	case "storage.list":
		return s.list(ctx, *appCtx.AppID)
	case "storage.clear":
		return s.clear(ctx, *appCtx.AppID)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (s *Storage) set(ctx context.Context, appID string, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	value := params["value"]
	if value == nil {
		return failure("value parameter required")
	}

	// Serialize value
	data, err := json.Marshal(value)
	if err != nil {
		return failure(fmt.Sprintf("failed to serialize value: %v", err))
	}

	// Write to filesystem via kernel
	path := s.keyPath(appID, key)
	if s.kernel != nil {
		_, err = s.kernel.ExecuteSyscall(ctx, s.storagePID, "write_file", map[string]interface{}{
			"path": path,
			"data": data,
		})
		if err != nil {
			return failure(fmt.Sprintf("write failed: %v", err))
		}
	}

	// Update cache with size limit enforcement
	cacheKey := s.cacheKey(appID, key)

	// Check if this is a new entry
	_, existed := s.cache.Load(cacheKey)

	s.cache.Store(cacheKey, value)

	// Increment cache size if new entry
	if !existed {
		newSize := atomic.AddInt64(&s.cacheSize, 1)
		// Evict oldest entries if cache is too large
		if newSize > CacheEvictionThreshold {
			s.evictCacheEntries()
		}
	}

	return success(map[string]interface{}{"stored": true, "key": key})
}

func (s *Storage) get(ctx context.Context, appID string, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	// Check cache first
	cacheKey := s.cacheKey(appID, key)
	if cached, ok := s.cache.Load(cacheKey); ok {
		return success(map[string]interface{}{"value": cached})
	}

	// Read from filesystem via kernel
	path := s.keyPath(appID, key)
	if s.kernel == nil {
		return failure("storage not available")
	}

	data, err := s.kernel.ExecuteSyscall(ctx, s.storagePID, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return success(map[string]interface{}{"value": nil}) // Key not found
	}

	// Deserialize with length check to prevent DoS
	if len(data) > 10*1024*1024 { // 10MB limit
		return failure("stored value exceeds size limit")
	}

	var value interface{}
	if err := json.Unmarshal(data, &value); err != nil {
		return failure(fmt.Sprintf("failed to deserialize key %s: %v", key, err))
	}

	// Update cache with size limit enforcement
	_, existed := s.cache.Load(cacheKey)
	s.cache.Store(cacheKey, value)

	// Increment cache size if new entry
	if !existed {
		newSize := atomic.AddInt64(&s.cacheSize, 1)
		// Evict oldest entries if cache is too large
		if newSize > CacheEvictionThreshold {
			s.evictCacheEntries()
		}
	}

	return success(map[string]interface{}{"value": value})
}

func (s *Storage) remove(ctx context.Context, appID string, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	// Delete from filesystem via kernel
	path := s.keyPath(appID, key)
	if s.kernel != nil {
		_, err := s.kernel.ExecuteSyscall(ctx, s.storagePID, "delete_file", map[string]interface{}{
			"path": path,
		})
		if err != nil {
			return failure(fmt.Sprintf("delete failed: %v", err))
		}
	}

	// Remove from cache
	cacheKey := s.cacheKey(appID, key)
	if _, existed := s.cache.Load(cacheKey); existed {
		s.cache.Delete(cacheKey)
		atomic.AddInt64(&s.cacheSize, -1)
	}

	return success(map[string]interface{}{"deleted": true})
}

func (s *Storage) list(ctx context.Context, appID string) (*types.Result, error) {
	// TODO: Implement directory listing via kernel
	// For now, return cached keys
	var keys []string
	prefix := s.cacheKey(appID, "")

	s.cache.Range(func(key, _ interface{}) bool {
		keyStr := key.(string)
		if len(keyStr) > len(prefix) && keyStr[:len(prefix)] == prefix {
			keys = append(keys, keyStr[len(prefix):])
		}
		return true
	})

	return success(map[string]interface{}{"keys": keys})
}

func (s *Storage) clear(ctx context.Context, appID string) (*types.Result, error) {
	// Delete all keys for this app
	prefix := s.cacheKey(appID, "")
	var toDelete []interface{}

	s.cache.Range(func(key, _ interface{}) bool {
		keyStr := key.(string)
		if len(keyStr) > len(prefix) && keyStr[:len(prefix)] == prefix {
			toDelete = append(toDelete, key)
		}
		return true
	})

	for _, key := range toDelete {
		s.cache.Delete(key)
	}

	// Update cache size counter
	atomic.AddInt64(&s.cacheSize, -int64(len(toDelete)))

	return success(map[string]interface{}{"cleared": true, "count": len(toDelete)})
}

func (s *Storage) keyPath(appID, key string) string {
	return filepath.Join(s.storagePath, "storage", appID, key+".json")
}

func (s *Storage) cacheKey(appID, key string) string {
	return fmt.Sprintf("%s:%s", appID, key)
}

// evictCacheEntries removes entries when cache grows too large
// Uses a single-threaded eviction approach to prevent race conditions
func (s *Storage) evictCacheEntries() {
	// Use atomic CAS to ensure only one goroutine performs eviction
	if !atomic.CompareAndSwapInt32(&s.evicting, 0, 1) {
		return // Another goroutine is already evicting
	}
	defer atomic.StoreInt32(&s.evicting, 0)

	// Double-check size after acquiring eviction lock
	currentSize := atomic.LoadInt64(&s.cacheSize)
	if currentSize <= CacheEvictionThreshold {
		return
	}

	targetEvictions := currentSize - CacheEvictionThreshold + 1000 // Remove extra for headroom
	evicted := int64(0)

	// Simple eviction: remove entries until we hit target
	// Note: sync.Map doesn't maintain insertion order, so this is pseudo-random
	s.cache.Range(func(key, _ interface{}) bool {
		if evicted >= targetEvictions {
			return false // Stop iteration
		}
		s.cache.Delete(key)
		evicted++
		return true
	})

	// Update counter
	atomic.AddInt64(&s.cacheSize, -evicted)
}

func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{
		Success: true,
		Data:    data,
	}, nil
}

func failure(message string) (*types.Result, error) {
	errMsg := message
	return &types.Result{
		Success: false,
		Error:   &errMsg,
	}, fmt.Errorf("%s", message)
}
