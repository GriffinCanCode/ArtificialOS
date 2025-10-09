package settings

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Provider implements settings and configuration management
type Provider struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	cache       sync.Map // In-memory cache for settings
}

// Setting represents a configuration setting
type Setting struct {
	Key         string      `json:"key"`
	Value       interface{} `json:"value"`
	Type        string      `json:"type"` // "string", "number", "boolean", "json"
	Category    string      `json:"category"`
	Description string      `json:"description"`
	Default     interface{} `json:"default"`
}

// NewProvider creates a settings provider
func NewProvider(kernel KernelClient, storagePID uint32, storagePath string) *Provider {
	p := &Provider{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}

	// Initialize with defaults
	p.initializeDefaults()

	return p
}

// Definition returns service metadata
func (s *Provider) Definition() types.Service {
	return types.Service{
		ID:          "settings",
		Name:        "Settings Service",
		Description: "System settings and configuration management",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"get",
			"set",
			"list",
			"reset",
			"export",
			"import",
		},
		Tools: []types.Tool{
			{
				ID:          "settings.get",
				Name:        "Get Setting",
				Description: "Get a configuration setting value",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Setting key", Required: true},
				},
				Returns: "Setting",
			},
			{
				ID:          "settings.set",
				Name:        "Set Setting",
				Description: "Set a configuration setting value",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Setting key", Required: true},
					{Name: "value", Type: "any", Description: "Setting value", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "settings.list",
				Name:        "List Settings",
				Description: "List all settings optionally filtered by category",
				Parameters: []types.Parameter{
					{Name: "category", Type: "string", Description: "Category filter (optional)", Required: false},
				},
				Returns: "array",
			},
			{
				ID:          "settings.reset",
				Name:        "Reset Setting",
				Description: "Reset a setting to its default value",
				Parameters: []types.Parameter{
					{Name: "key", Type: "string", Description: "Setting key", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "settings.export",
				Name:        "Export Settings",
				Description: "Export all settings as JSON",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
			{
				ID:          "settings.import",
				Name:        "Import Settings",
				Description: "Import settings from JSON",
				Parameters: []types.Parameter{
					{Name: "settings", Type: "object", Description: "Settings to import", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "settings.categories",
				Name:        "List Categories",
				Description: "Get all setting categories",
				Parameters:  []types.Parameter{},
				Returns:     "array",
			},
		},
	}
}

// Execute runs a settings operation
func (s *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "settings.get":
		return s.get(ctx, params)
	case "settings.set":
		return s.set(ctx, params)
	case "settings.list":
		return s.list(ctx, params)
	case "settings.reset":
		return s.reset(ctx, params)
	case "settings.export":
		return s.exportSettings(ctx)
	case "settings.import":
		return s.importSettings(ctx, params)
	case "settings.categories":
		return s.categories(ctx)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

// initializeDefaults sets up default settings
func (s *Provider) initializeDefaults() {
	defaults := map[string]Setting{
		// General
		"general.theme":         {Key: "general.theme", Value: "dark", Type: "string", Category: "general", Description: "UI theme", Default: "dark"},
		"general.language":      {Key: "general.language", Value: "en", Type: "string", Category: "general", Description: "Interface language", Default: "en"},
		"general.notifications": {Key: "general.notifications", Value: true, Type: "boolean", Category: "general", Description: "Enable notifications", Default: true},

		// System
		"system.memory_limit": {Key: "system.memory_limit", Value: 1024, Type: "number", Category: "system", Description: "Memory limit (MB)", Default: 1024},
		"system.cpu_cores":    {Key: "system.cpu_cores", Value: 0, Type: "number", Category: "system", Description: "CPU cores (0 = auto)", Default: 0},
		"system.log_level":    {Key: "system.log_level", Value: "info", Type: "string", Category: "system", Description: "Logging level", Default: "info"},

		// Appearance
		"appearance.font_size":    {Key: "appearance.font_size", Value: 14, Type: "number", Category: "appearance", Description: "Font size (px)", Default: 14},
		"appearance.font_family":  {Key: "appearance.font_family", Value: "Inter", Type: "string", Category: "appearance", Description: "Font family", Default: "Inter"},
		"appearance.accent_color": {Key: "appearance.accent_color", Value: "#3b82f6", Type: "string", Category: "appearance", Description: "Accent color", Default: "#3b82f6"},

		// Network
		"network.proxy_enabled": {Key: "network.proxy_enabled", Value: false, Type: "boolean", Category: "network", Description: "Enable proxy", Default: false},
		"network.proxy_url":     {Key: "network.proxy_url", Value: "", Type: "string", Category: "network", Description: "Proxy URL", Default: ""},

		// Developer
		"developer.debug_mode": {Key: "developer.debug_mode", Value: false, Type: "boolean", Category: "developer", Description: "Enable debug mode", Default: false},
		"developer.show_fps":   {Key: "developer.show_fps", Value: false, Type: "boolean", Category: "developer", Description: "Show FPS counter", Default: false},
	}

	for k, v := range defaults {
		s.cache.Store(k, v)
	}
}

func (s *Provider) get(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	// Check cache first
	if val, ok := s.cache.Load(key); ok {
		setting := val.(Setting)
		return success(map[string]interface{}{
			"key":         setting.Key,
			"value":       setting.Value,
			"type":        setting.Type,
			"category":    setting.Category,
			"description": setting.Description,
			"default":     setting.Default,
		})
	}

	return failure(fmt.Sprintf("setting not found: %s", key))
}

func (s *Provider) set(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	value := params["value"]
	if value == nil {
		return failure("value parameter required")
	}

	// Get existing setting to preserve metadata
	var setting Setting
	if val, ok := s.cache.Load(key); ok {
		setting = val.(Setting)
		setting.Value = value
	} else {
		// Create new setting
		setting = Setting{
			Key:      key,
			Value:    value,
			Type:     inferType(value),
			Category: "custom",
		}
	}

	// Store in cache
	s.cache.Store(key, setting)

	// Persist to filesystem
	if s.kernel != nil {
		data, err := json.Marshal(setting)
		if err != nil {
			return failure(fmt.Sprintf("failed to serialize setting: %v", err))
		}

		path := fmt.Sprintf("%s/settings/%s.json", s.storagePath, key)
		_, err = s.kernel.ExecuteSyscall(ctx, s.storagePID, "write_file", map[string]interface{}{
			"path": path,
			"data": data,
		})
		if err != nil {
			return failure(fmt.Sprintf("failed to persist setting: %v", err))
		}
	}

	return success(map[string]interface{}{"stored": true, "key": key})
}

func (s *Provider) list(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	category, _ := params["category"].(string)

	var settings []Setting
	s.cache.Range(func(key, value interface{}) bool {
		setting := value.(Setting)
		if category == "" || setting.Category == category {
			settings = append(settings, setting)
		}
		return true
	})

	return success(map[string]interface{}{"settings": settings, "count": len(settings)})
}

func (s *Provider) reset(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	key, ok := params["key"].(string)
	if !ok || key == "" {
		return failure("key parameter required")
	}

	// Get setting
	val, ok := s.cache.Load(key)
	if !ok {
		return failure(fmt.Sprintf("setting not found: %s", key))
	}

	setting := val.(Setting)
	setting.Value = setting.Default
	s.cache.Store(key, setting)

	return success(map[string]interface{}{"reset": true, "key": key, "value": setting.Default})
}

func (s *Provider) exportSettings(ctx context.Context) (*types.Result, error) {
	settings := make(map[string]interface{})

	s.cache.Range(func(key, value interface{}) bool {
		setting := value.(Setting)
		settings[setting.Key] = setting.Value
		return true
	})

	return success(map[string]interface{}{"settings": settings})
}

func (s *Provider) importSettings(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	settingsData, ok := params["settings"].(map[string]interface{})
	if !ok {
		return failure("settings parameter must be an object")
	}

	count := 0
	for key, value := range settingsData {
		_, err := s.set(ctx, map[string]interface{}{"key": key, "value": value})
		if err == nil {
			count++
		}
	}

	return success(map[string]interface{}{"imported": count})
}

func (s *Provider) categories(ctx context.Context) (*types.Result, error) {
	categorySet := make(map[string]bool)

	s.cache.Range(func(key, value interface{}) bool {
		setting := value.(Setting)
		categorySet[setting.Category] = true
		return true
	})

	categories := make([]string, 0, len(categorySet))
	for cat := range categorySet {
		categories = append(categories, cat)
	}

	return success(map[string]interface{}{"categories": categories})
}

// Helper functions
func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	errMsg := message
	return &types.Result{Success: false, Error: &errMsg}, nil
}

func inferType(value interface{}) string {
	switch value.(type) {
	case bool:
		return "boolean"
	case float64, int, int64:
		return "number"
	case string:
		return "string"
	default:
		return "json"
	}
}
