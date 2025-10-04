package providers

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Storage provides persistent key-value storage per app
type Storage struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	cache       sync.Map
	mu          sync.RWMutex
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
func (s *Storage) Execute(toolID string, params map[string]interface{}, ctx *types.Context) (*types.Result, error) {
	if ctx == nil || ctx.AppID == nil {
		return failure("app context required for storage operations")
	}

	switch toolID {
	case "storage.set":
		return s.set(*ctx.AppID, params)
	case "storage.get":
		return s.get(*ctx.AppID, params)
	case "storage.remove":
		return s.remove(*ctx.AppID, params)
	case "storage.list":
		return s.list(*ctx.AppID)
	case "storage.clear":
		return s.clear(*ctx.AppID)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (s *Storage) set(appID string, params map[string]interface{}) (*types.Result, error) {
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
		_, err = s.kernel.ExecuteSyscall(s.storagePID, "write_file", map[string]interface{}{
			"path": path,
			"data": data,
		})
		if err != nil {
			return failure(fmt.Sprintf("write failed: %v", err))
		}
	}

	// Update cache
	cacheKey := s.cacheKey(appID, key)
	s.cache.Store(cacheKey, value)

	return success(map[string]interface{}{"stored": true, "key": key})
}

func (s *Storage) get(appID string, params map[string]interface{}) (*types.Result, error) {
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

	data, err := s.kernel.ExecuteSyscall(s.storagePID, "read_file", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return success(map[string]interface{}{"value": nil}) // Key not found
	}

	// Deserialize
	var value interface{}
	if err := json.Unmarshal(data, &value); err != nil {
		return failure(fmt.Sprintf("failed to deserialize: %v", err))
	}

	// Update cache
	s.cache.Store(cacheKey, value)

	return success(map[string]interface{}{"value": value})
}

func (s *Storage) remove(appID string, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	// Delete from filesystem via kernel
	path := s.keyPath(appID, key)
	if s.kernel != nil {
		_, err := s.kernel.ExecuteSyscall(s.storagePID, "delete_file", map[string]interface{}{
			"path": path,
		})
		if err != nil {
			return failure(fmt.Sprintf("delete failed: %v", err))
		}
	}

	// Remove from cache
	cacheKey := s.cacheKey(appID, key)
	s.cache.Delete(cacheKey)

	return success(map[string]interface{}{"deleted": true})
}

func (s *Storage) list(appID string) (*types.Result, error) {
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

func (s *Storage) clear(appID string) (*types.Result, error) {
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

	return success(map[string]interface{}{"cleared": true, "count": len(toDelete)})
}

func (s *Storage) keyPath(appID, key string) string {
	return filepath.Join(s.storagePath, "storage", appID, key+".json")
}

func (s *Storage) cacheKey(appID, key string) string {
	return fmt.Sprintf("%s:%s", appID, key)
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
