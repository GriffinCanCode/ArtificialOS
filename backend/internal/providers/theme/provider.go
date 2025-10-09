package theme

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

// Provider implements theme management
type Provider struct {
	kernel       KernelClient
	storagePID   uint32
	storagePath  string
	themes       sync.Map
	currentTheme string
}

// Theme represents a UI theme
type Theme struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Description string                 `json:"description,omitempty"`
	Type        string                 `json:"type"` // "dark", "light", "custom"
	Colors      map[string]string      `json:"colors"`
	Fonts       map[string]string      `json:"fonts,omitempty"`
	Custom      map[string]interface{} `json:"custom,omitempty"`
}

// NewProvider creates a theme provider
func NewProvider(kernel KernelClient, storagePID uint32, storagePath string) *Provider {
	p := &Provider{
		kernel:       kernel,
		storagePID:   storagePID,
		storagePath:  storagePath,
		currentTheme: "dark",
	}

	// Initialize with default themes
	p.initializeDefaults()

	return p
}

// Definition returns service metadata
func (t *Provider) Definition() types.Service {
	return types.Service{
		ID:          "theme",
		Name:        "Theme Manager",
		Description: "Manage UI themes and appearance",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"list",
			"get",
			"set",
			"create",
			"delete",
		},
		Tools: []types.Tool{
			{
				ID:          "theme.list",
				Name:        "List Themes",
				Description: "List all available themes",
				Parameters:  []types.Parameter{},
				Returns:     "array",
			},
			{
				ID:          "theme.get",
				Name:        "Get Theme",
				Description: "Get a theme by ID",
				Parameters: []types.Parameter{
					{Name: "id", Type: "string", Description: "Theme ID", Required: true},
				},
				Returns: "Theme",
			},
			{
				ID:          "theme.current",
				Name:        "Get Current Theme",
				Description: "Get the currently active theme",
				Parameters:  []types.Parameter{},
				Returns:     "Theme",
			},
			{
				ID:          "theme.set",
				Name:        "Set Theme",
				Description: "Set the active theme",
				Parameters: []types.Parameter{
					{Name: "id", Type: "string", Description: "Theme ID", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "theme.create",
				Name:        "Create Theme",
				Description: "Create a custom theme",
				Parameters: []types.Parameter{
					{Name: "theme", Type: "object", Description: "Theme definition", Required: true},
				},
				Returns: "Theme",
			},
			{
				ID:          "theme.delete",
				Name:        "Delete Theme",
				Description: "Delete a custom theme",
				Parameters: []types.Parameter{
					{Name: "id", Type: "string", Description: "Theme ID", Required: true},
				},
				Returns: "boolean",
			},
		},
	}
}

// Execute runs a theme operation
func (t *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "theme.list":
		return t.list(ctx)
	case "theme.get":
		return t.get(ctx, params)
	case "theme.current":
		return t.current(ctx)
	case "theme.set":
		return t.set(ctx, params)
	case "theme.create":
		return t.create(ctx, params)
	case "theme.delete":
		return t.delete(ctx, params)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (t *Provider) initializeDefaults() {
	// Dark theme
	dark := Theme{
		ID:          "dark",
		Name:        "Dark",
		Description: "Default dark theme",
		Type:        "dark",
		Colors: map[string]string{
			"background": "#1a1a1a",
			"surface":    "#252525",
			"primary":    "#3b82f6",
			"secondary":  "#8b5cf6",
			"accent":     "#10b981",
			"text":       "#ffffff",
			"textMuted":  "#a0a0a0",
			"border":     "#404040",
		},
		Fonts: map[string]string{
			"sans": "Inter, system-ui, sans-serif",
			"mono": "JetBrains Mono, monospace",
		},
	}

	// Light theme
	light := Theme{
		ID:          "light",
		Name:        "Light",
		Description: "Default light theme",
		Type:        "light",
		Colors: map[string]string{
			"background": "#ffffff",
			"surface":    "#f5f5f5",
			"primary":    "#3b82f6",
			"secondary":  "#8b5cf6",
			"accent":     "#10b981",
			"text":       "#1a1a1a",
			"textMuted":  "#666666",
			"border":     "#e0e0e0",
		},
		Fonts: map[string]string{
			"sans": "Inter, system-ui, sans-serif",
			"mono": "JetBrains Mono, monospace",
		},
	}

	// High contrast theme
	highContrast := Theme{
		ID:          "high-contrast",
		Name:        "High Contrast",
		Description: "High contrast theme for accessibility",
		Type:        "dark",
		Colors: map[string]string{
			"background": "#000000",
			"surface":    "#1a1a1a",
			"primary":    "#00ffff",
			"secondary":  "#ff00ff",
			"accent":     "#00ff00",
			"text":       "#ffffff",
			"textMuted":  "#cccccc",
			"border":     "#ffffff",
		},
		Fonts: map[string]string{
			"sans": "Inter, system-ui, sans-serif",
			"mono": "JetBrains Mono, monospace",
		},
	}

	t.themes.Store("dark", dark)
	t.themes.Store("light", light)
	t.themes.Store("high-contrast", highContrast)
}

func (t *Provider) list(ctx context.Context) (*types.Result, error) {
	var themes []Theme

	t.themes.Range(func(key, value interface{}) bool {
		theme := value.(Theme)
		themes = append(themes, theme)
		return true
	})

	return success(map[string]interface{}{"themes": themes, "count": len(themes)})
}

func (t *Provider) get(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	id, ok := params["id"].(string)
	if !ok || id == "" {
		return failure("id parameter required")
	}

	val, ok := t.themes.Load(id)
	if !ok {
		return failure(fmt.Sprintf("theme not found: %s", id))
	}

	theme := val.(Theme)
	return success(map[string]interface{}{
		"id":          theme.ID,
		"name":        theme.Name,
		"description": theme.Description,
		"type":        theme.Type,
		"colors":      theme.Colors,
		"fonts":       theme.Fonts,
		"custom":      theme.Custom,
	})
}

func (t *Provider) current(ctx context.Context) (*types.Result, error) {
	return t.get(ctx, map[string]interface{}{"id": t.currentTheme})
}

func (t *Provider) set(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	id, ok := params["id"].(string)
	if !ok || id == "" {
		return failure("id parameter required")
	}

	// Verify theme exists
	if _, ok := t.themes.Load(id); !ok {
		return failure(fmt.Sprintf("theme not found: %s", id))
	}

	t.currentTheme = id

	// Persist to storage
	if t.kernel != nil {
		data, err := json.Marshal(map[string]string{"current_theme": id})
		if err == nil {
			path := fmt.Sprintf("%s/theme/current.json", t.storagePath)
			t.kernel.ExecuteSyscall(ctx, t.storagePID, "write_file", map[string]interface{}{
				"path": path,
				"data": data,
			})
		}
	}

	return success(map[string]interface{}{"set": true, "theme": id})
}

func (t *Provider) create(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	themeData, ok := params["theme"].(map[string]interface{})
	if !ok {
		return failure("theme parameter must be an object")
	}

	// Parse theme
	themeJSON, err := json.Marshal(themeData)
	if err != nil {
		return failure(fmt.Sprintf("failed to serialize theme: %v", err))
	}

	var theme Theme
	if err := json.Unmarshal(themeJSON, &theme); err != nil {
		return failure(fmt.Sprintf("failed to parse theme: %v", err))
	}

	if theme.ID == "" {
		return failure("theme must have an id")
	}

	if theme.Type == "" {
		theme.Type = "custom"
	}

	// Store theme
	t.themes.Store(theme.ID, theme)

	// Persist to filesystem
	if t.kernel != nil {
		data, err := json.Marshal(theme)
		if err == nil {
			path := fmt.Sprintf("%s/theme/%s.json", t.storagePath, theme.ID)
			t.kernel.ExecuteSyscall(ctx, t.storagePID, "write_file", map[string]interface{}{
				"path": path,
				"data": data,
			})
		}
	}

	return success(map[string]interface{}{
		"created": true,
		"theme":   theme,
	})
}

func (t *Provider) delete(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	id, ok := params["id"].(string)
	if !ok || id == "" {
		return failure("id parameter required")
	}

	// Don't allow deleting built-in themes
	if id == "dark" || id == "light" || id == "high-contrast" {
		return failure("cannot delete built-in theme")
	}

	// Delete from cache
	t.themes.Delete(id)

	// Delete from filesystem
	if t.kernel != nil {
		path := fmt.Sprintf("%s/theme/%s.json", t.storagePath, id)
		t.kernel.ExecuteSyscall(ctx, t.storagePID, "delete_file", map[string]interface{}{
			"path": path,
		})
	}

	return success(map[string]interface{}{"deleted": true, "id": id})
}

// Helper functions
func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	errMsg := message
	return &types.Result{Success: false, Error: &errMsg}, nil
}
