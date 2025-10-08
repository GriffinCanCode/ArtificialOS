package search

import (
	"context"
	"testing"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Mock kernel client for testing
type mockKernelClient struct {
	executeSyscallFunc func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

func (m *mockKernelClient) ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	if m.executeSyscallFunc != nil {
		return m.executeSyscallFunc(ctx, pid, syscallType, params)
	}
	return []byte("[]"), nil
}

func TestProviderDefinition(t *testing.T) {
	mockClient := &mockKernelClient{}
	provider := NewProvider(mockClient, 1)

	def := provider.Definition()

	if def.ID != "search" {
		t.Errorf("Expected ID 'search', got '%s'", def.ID)
	}

	if def.Name != "Spotlight Search" {
		t.Errorf("Expected name 'Spotlight Search', got '%s'", def.Name)
	}

	if def.Category != types.CategorySystem {
		t.Errorf("Expected category System, got %v", def.Category)
	}

	if len(def.Tools) != 3 {
		t.Errorf("Expected 3 tools, got %d", len(def.Tools))
	}
}

func TestSearchFilesBasic(t *testing.T) {
	callCount := 0
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			callCount++
			if syscallType != "search_files" {
				t.Errorf("Expected syscall type 'search_files', got '%s'", syscallType)
			}

			query, ok := params["query"].(string)
			if !ok || query != "test" {
				t.Errorf("Expected query 'test', got '%v'", params["query"])
			}

			return []byte(`[{"path":"/test.txt","score":0.1}]`), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query": "test",
	}

	result, err := provider.searchFiles(context.Background(), params)

	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if !result.Success {
		t.Error("Expected successful result")
	}

	if callCount != 1 {
		t.Errorf("Expected 1 kernel call, got %d", callCount)
	}
}

func TestSearchFilesWithParameters(t *testing.T) {
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			// Verify all parameters are passed correctly
			if params["path"] != "/custom" {
				t.Errorf("Expected path '/custom', got '%v'", params["path"])
			}
			if params["limit"] != 100 {
				t.Errorf("Expected limit 100, got '%v'", params["limit"])
			}
			if params["recursive"] != true {
				t.Errorf("Expected recursive true, got '%v'", params["recursive"])
			}
			if params["threshold"] != 0.5 {
				t.Errorf("Expected threshold 0.5, got '%v'", params["threshold"])
			}

			return []byte("[]"), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query":     "test",
		"path":      "/custom",
		"limit":     float64(100),
		"recursive": true,
		"threshold": 0.5,
	}

	_, err := provider.searchFiles(context.Background(), params)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}
}

func TestSearchFilesMissingQuery(t *testing.T) {
	mockClient := &mockKernelClient{}
	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{}

	result, err := provider.searchFiles(context.Background(), params)

	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if result.Success {
		t.Error("Expected failure for missing query")
	}

	if result.Error == nil || *result.Error != "query parameter required" {
		t.Errorf("Expected error message 'query parameter required', got '%v'", result.Error)
	}
}

func TestSearchContent(t *testing.T) {
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			if syscallType != "search_content" {
				t.Errorf("Expected syscall type 'search_content', got '%s'", syscallType)
			}

			return []byte(`[{"path":"/test.txt","content":"hello world","line_number":1}]`), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query": "hello",
	}

	result, err := provider.searchContent(context.Background(), params)

	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if !result.Success {
		t.Error("Expected successful result")
	}
}

func TestSearchAllParallel(t *testing.T) {
	callCount := 0
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			callCount++
			// Simulate some latency
			time.Sleep(10 * time.Millisecond)
			return []byte("[]"), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query": "test",
	}

	start := time.Now()
	result, err := provider.searchAll(context.Background(), params)
	duration := time.Since(start)

	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if !result.Success {
		t.Error("Expected successful result")
	}

	// Should complete in ~10ms due to parallel execution, not 20ms
	if duration > 100*time.Millisecond {
		t.Errorf("Search took too long: %v (expected parallel execution)", duration)
	}
}

func TestExecuteUnknownTool(t *testing.T) {
	mockClient := &mockKernelClient{}
	provider := NewProvider(mockClient, 1)

	result, err := provider.Execute(context.Background(), "search.unknown", nil, nil)

	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if result.Success {
		t.Error("Expected failure for unknown tool")
	}
}

func TestSearchFilesDefaultParameters(t *testing.T) {
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			// Verify defaults are applied
			if params["path"] != "/" {
				t.Errorf("Expected default path '/', got '%v'", params["path"])
			}
			if params["limit"] != 50 {
				t.Errorf("Expected default limit 50, got '%v'", params["limit"])
			}
			if params["threshold"] != 0.3 {
				t.Errorf("Expected default threshold 0.3, got '%v'", params["threshold"])
			}

			return []byte("[]"), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query": "test",
	}

	_, err := provider.searchFiles(context.Background(), params)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}
}

func TestSearchContentDefaultParameters(t *testing.T) {
	mockClient := &mockKernelClient{
		executeSyscallFunc: func(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
			// Verify defaults are applied
			if params["path"] != "/" {
				t.Errorf("Expected default path '/', got '%v'", params["path"])
			}
			if params["limit"] != 50 {
				t.Errorf("Expected default limit 50, got '%v'", params["limit"])
			}

			return []byte("[]"), nil
		},
	}

	provider := NewProvider(mockClient, 1)

	params := map[string]interface{}{
		"query": "test",
	}

	_, err := provider.searchContent(context.Background(), params)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}
}
