package clipboard

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Provider implements clipboard operations with kernel integration
type Provider struct {
	kernel KernelClient
	pid    uint32
}

// NewProvider creates a clipboard provider
func NewProvider(kernel KernelClient, pid uint32) *Provider {
	return &Provider{
		kernel: kernel,
		pid:    pid,
	}
}

// Definition returns service metadata
func (c *Provider) Definition() types.Service {
	return types.Service{
		ID:          "clipboard",
		Name:        "Clipboard Service",
		Description: "Multi-format clipboard with history and subscription support",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"copy",
			"paste",
			"history",
			"multi_format",
			"global_clipboard",
			"subscriptions",
			"statistics",
		},
		Tools: c.getTools(),
	}
}

func (c *Provider) getTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "clipboard.copy",
			Name:        "Copy to Clipboard",
			Description: "Copy data to clipboard with format specification",
			Parameters: []types.Parameter{
				{Name: "data", Type: "string", Description: "Data to copy", Required: true},
				{Name: "format", Type: "string", Description: "Data format (text, html, bytes, image/*)", Required: false},
				{Name: "global", Type: "boolean", Description: "Copy to global clipboard", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "clipboard.paste",
			Name:        "Paste from Clipboard",
			Description: "Retrieve data from clipboard",
			Parameters: []types.Parameter{
				{Name: "global", Type: "boolean", Description: "Paste from global clipboard", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "clipboard.history",
			Name:        "Get Clipboard History",
			Description: "Retrieve clipboard history entries",
			Parameters: []types.Parameter{
				{Name: "limit", Type: "number", Description: "Maximum number of entries", Required: false},
				{Name: "global", Type: "boolean", Description: "Get global clipboard history", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "clipboard.get_entry",
			Name:        "Get Clipboard Entry",
			Description: "Retrieve specific clipboard entry by ID",
			Parameters: []types.Parameter{
				{Name: "entry_id", Type: "number", Description: "Entry ID to retrieve", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "clipboard.clear",
			Name:        "Clear Clipboard",
			Description: "Clear clipboard content",
			Parameters: []types.Parameter{
				{Name: "global", Type: "boolean", Description: "Clear global clipboard", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "clipboard.subscribe",
			Name:        "Subscribe to Changes",
			Description: "Subscribe to clipboard change notifications",
			Parameters: []types.Parameter{
				{Name: "formats", Type: "array", Description: "Format filters (empty = all formats)", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "clipboard.unsubscribe",
			Name:        "Unsubscribe from Changes",
			Description: "Unsubscribe from clipboard change notifications",
			Parameters:  []types.Parameter{},
			Returns:     "boolean",
		},
		{
			ID:          "clipboard.stats",
			Name:        "Get Clipboard Statistics",
			Description: "Retrieve clipboard usage statistics",
			Parameters:  []types.Parameter{},
			Returns:     "object",
		},
	}
}

// Execute runs a clipboard operation
func (c *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "clipboard.copy":
		return c.copy(ctx, params)
	case "clipboard.paste":
		return c.paste(ctx, params)
	case "clipboard.history":
		return c.history(ctx, params)
	case "clipboard.get_entry":
		return c.getEntry(ctx, params)
	case "clipboard.clear":
		return c.clear(ctx, params)
	case "clipboard.subscribe":
		return c.subscribe(ctx, params)
	case "clipboard.unsubscribe":
		return c.unsubscribe(ctx)
	case "clipboard.stats":
		return c.stats(ctx)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (c *Provider) copy(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	data, ok := params["data"].(string)
	if !ok || data == "" {
		return failure("data parameter required")
	}

	format := "text"
	if f, ok := params["format"].(string); ok && f != "" {
		format = f
	}

	global := false
	if g, ok := params["global"].(bool); ok {
		global = g
	}

	// Execute clipboard_copy syscall
	result, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_copy", map[string]interface{}{
		"data":   []byte(data),
		"format": format,
		"global": global,
	})

	if err != nil {
		return failure(fmt.Sprintf("copy failed: %v", err))
	}

	// Parse entry ID from result
	var entryID uint64
	if err := json.Unmarshal(result, &entryID); err != nil {
		return failure(fmt.Sprintf("failed to parse entry ID: %v", err))
	}

	return success(map[string]interface{}{
		"copied":   true,
		"entry_id": entryID,
		"format":   format,
		"global":   global,
	})
}

func (c *Provider) paste(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	global := false
	if g, ok := params["global"].(bool); ok {
		global = g
	}

	// Execute clipboard_paste syscall
	result, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_paste", map[string]interface{}{
		"global": global,
	})

	if err != nil {
		return failure(fmt.Sprintf("paste failed: %v", err))
	}

	// Parse clipboard entry
	var entry map[string]interface{}
	if err := json.Unmarshal(result, &entry); err != nil {
		return failure(fmt.Sprintf("failed to parse clipboard entry: %v", err))
	}

	return success(entry)
}

func (c *Provider) history(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	var limit *int
	if l, ok := params["limit"].(float64); ok {
		limitInt := int(l)
		limit = &limitInt
	}

	global := false
	if g, ok := params["global"].(bool); ok {
		global = g
	}

	// Execute clipboard_history syscall
	syscallParams := map[string]interface{}{
		"global": global,
	}
	if limit != nil {
		syscallParams["limit"] = *limit
	}

	result, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_history", syscallParams)
	if err != nil {
		return failure(fmt.Sprintf("history failed: %v", err))
	}

	// Parse history
	var history []map[string]interface{}
	if err := json.Unmarshal(result, &history); err != nil {
		return failure(fmt.Sprintf("failed to parse history: %v", err))
	}

	return success(map[string]interface{}{
		"entries": history,
		"count":   len(history),
	})
}

func (c *Provider) getEntry(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	entryID, ok := params["entry_id"].(float64)
	if !ok {
		return failure("entry_id parameter required")
	}

	// Execute clipboard_get_entry syscall
	result, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_get_entry", map[string]interface{}{
		"entry_id": uint64(entryID),
	})

	if err != nil {
		return failure(fmt.Sprintf("get entry failed: %v", err))
	}

	// Parse entry
	var entry map[string]interface{}
	if err := json.Unmarshal(result, &entry); err != nil {
		return failure(fmt.Sprintf("failed to parse entry: %v", err))
	}

	return success(entry)
}

func (c *Provider) clear(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	global := false
	if g, ok := params["global"].(bool); ok {
		global = g
	}

	// Execute clipboard_clear syscall
	_, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_clear", map[string]interface{}{
		"global": global,
	})

	if err != nil {
		return failure(fmt.Sprintf("clear failed: %v", err))
	}

	return success(map[string]interface{}{"cleared": true})
}

func (c *Provider) subscribe(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	formats := []string{}
	if f, ok := params["formats"].([]interface{}); ok {
		for _, format := range f {
			if str, ok := format.(string); ok {
				formats = append(formats, str)
			}
		}
	}

	// Execute clipboard_subscribe syscall
	_, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_subscribe", map[string]interface{}{
		"formats": formats,
	})

	if err != nil {
		return failure(fmt.Sprintf("subscribe failed: %v", err))
	}

	return success(map[string]interface{}{"subscribed": true})
}

func (c *Provider) unsubscribe(ctx context.Context) (*types.Result, error) {
	// Execute clipboard_unsubscribe syscall
	_, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_unsubscribe", map[string]interface{}{})
	if err != nil {
		return failure(fmt.Sprintf("unsubscribe failed: %v", err))
	}

	return success(map[string]interface{}{"unsubscribed": true})
}

func (c *Provider) stats(ctx context.Context) (*types.Result, error) {
	// Execute clipboard_stats syscall
	result, err := c.kernel.ExecuteSyscall(ctx, c.pid, "clipboard_stats", map[string]interface{}{})
	if err != nil {
		return failure(fmt.Sprintf("stats failed: %v", err))
	}

	// Parse stats
	var stats map[string]interface{}
	if err := json.Unmarshal(result, &stats); err != nil {
		return failure(fmt.Sprintf("failed to parse stats: %v", err))
	}

	return success(stats)
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
