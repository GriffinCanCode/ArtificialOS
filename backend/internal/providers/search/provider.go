package search

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelInterface defines the required kernel operations
type KernelInterface interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Provider implements search functionality across multiple sources
type Provider struct {
	kernel KernelInterface
	pid    uint32
}

// NewProvider creates a new search provider
func NewProvider(kernelClient KernelInterface, pid uint32) *Provider {
	return &Provider{
		kernel: kernelClient,
		pid:    pid,
	}
}

// Definition returns the search service definition
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:           "search",
		Name:         "Spotlight Search",
		Description:  "Global search across files, apps, and system resources",
		Category:     types.CategorySystem,
		Capabilities: []string{"search_files", "search_content", "search_apps", "search_all"},
		Tools:        p.getTools(),
	}
}

// Execute runs a search tool
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "search.files":
		return p.searchFiles(ctx, params)
	case "search.content":
		return p.searchContent(ctx, params)
	case "search.all":
		return p.searchAll(ctx, params)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

// searchFiles searches for files by name
func (p *Provider) searchFiles(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	query, ok := params["query"].(string)
	if !ok || query == "" {
		return failure("query parameter required")
	}

	path, _ := params["path"].(string)
	if path == "" {
		path = "/"
	}

	limit, _ := params["limit"].(float64)
	if limit == 0 {
		limit = 50
	}

	recursive, _ := params["recursive"].(bool)
	threshold, _ := params["threshold"].(float64)
	if threshold == 0 {
		threshold = 0.3
	}

	// Execute kernel syscall
	syscallParams := map[string]interface{}{
		"path":      path,
		"query":     query,
		"limit":     int(limit),
		"recursive": recursive,
		"threshold": threshold,
	}

	result, err := p.kernel.ExecuteSyscall(ctx, p.pid, "search_files", syscallParams)
	if err != nil {
		return failure(err.Error())
	}

	return success(map[string]interface{}{"results": result})
}

// searchContent searches file contents
func (p *Provider) searchContent(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	query, ok := params["query"].(string)
	if !ok || query == "" {
		return failure("query parameter required")
	}

	path, _ := params["path"].(string)
	if path == "" {
		path = "/"
	}

	limit, _ := params["limit"].(float64)
	if limit == 0 {
		limit = 50
	}

	recursive, _ := params["recursive"].(bool)

	syscallParams := map[string]interface{}{
		"path":      path,
		"query":     query,
		"limit":     int(limit),
		"recursive": recursive,
	}

	result, err := p.kernel.ExecuteSyscall(ctx, p.pid, "search_content", syscallParams)
	if err != nil {
		return failure(err.Error())
	}

	return success(map[string]interface{}{"results": result})
}

// searchAll searches across multiple sources (files, apps, etc.)
func (p *Provider) searchAll(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	query, ok := params["query"].(string)
	if !ok || query == "" {
		return failure("query parameter required")
	}

	limit, _ := params["limit"].(float64)
	if limit == 0 {
		limit = 20
	}

	// Search multiple sources in parallel
	var wg sync.WaitGroup
	results := make(map[string]interface{})
	mu := sync.Mutex{}

	// Search files
	wg.Add(1)
	go func() {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
		defer cancel()

		fileParams := map[string]interface{}{
			"query":     query,
			"path":      "/",
			"limit":     limit,
			"recursive": true,
			"threshold": 0.4,
		}

		result, err := p.searchFiles(ctx, fileParams)
		if err == nil && result.Success {
			mu.Lock()
			results["files"] = result.Data
			mu.Unlock()
		}
	}()

	// Wait for all searches to complete
	wg.Wait()

	return success(results)
}

// Helper functions
func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}

// getTools returns the list of available search tools
func (p *Provider) getTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "search.files",
			Name:        "Search Files",
			Description: "Search for files by name using fuzzy matching",
			Parameters: []types.Parameter{
				{Name: "query", Type: "string", Required: true, Description: "Search query"},
				{Name: "path", Type: "string", Required: false, Description: "Starting directory (default: /)"},
				{Name: "limit", Type: "number", Required: false, Description: "Maximum results (default: 50)"},
				{Name: "recursive", Type: "boolean", Required: false, Description: "Recursive search (default: true)"},
				{Name: "threshold", Type: "number", Required: false, Description: "Match threshold 0-1 (default: 0.3)"},
			},
			Returns: "array",
		},
		{
			ID:          "search.content",
			Name:        "Search Content",
			Description: "Search file contents (grep-like)",
			Parameters: []types.Parameter{
				{Name: "query", Type: "string", Required: true, Description: "Search query"},
				{Name: "path", Type: "string", Required: false, Description: "Starting directory (default: /)"},
				{Name: "limit", Type: "number", Required: false, Description: "Maximum results (default: 50)"},
				{Name: "recursive", Type: "boolean", Required: false, Description: "Recursive search (default: true)"},
			},
			Returns: "array",
		},
		{
			ID:          "search.all",
			Name:        "Search All",
			Description: "Search across all sources (files, apps, etc.)",
			Parameters: []types.Parameter{
				{Name: "query", Type: "string", Required: true, Description: "Search query"},
				{Name: "limit", Type: "number", Required: false, Description: "Results per category (default: 20)"},
			},
			Returns: "object",
		},
	}
}
