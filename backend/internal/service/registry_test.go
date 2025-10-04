package service

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

type mockProvider struct {
	id string
}

func (m *mockProvider) Definition() types.Service {
	return types.Service{
		ID:           m.id,
		Name:         "Mock Service",
		Description:  "A mock service for testing",
		Category:     types.CategoryStorage,
		Capabilities: []string{"read", "write"},
		Tools: []types.Tool{
			{
				ID:          m.id + ".test",
				Name:        "Test Tool",
				Description: "A test tool",
				Returns:     "string",
			},
		},
	}
}

func (m *mockProvider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"result": "success"},
	}, nil
}

func TestRegister(t *testing.T) {
	r := NewRegistry()
	p := &mockProvider{id: "test"}

	if err := r.Register(p); err != nil {
		t.Fatalf("Register failed: %v", err)
	}

	if _, ok := r.Get("test"); !ok {
		t.Error("Service should be registered")
	}
}

func TestList(t *testing.T) {
	r := NewRegistry()
	r.Register(&mockProvider{id: "test1"})
	r.Register(&mockProvider{id: "test2"})

	services := r.List(nil)
	if len(services) != 2 {
		t.Errorf("Expected 2 services, got %d", len(services))
	}

	cat := types.CategoryStorage
	filtered := r.List(&cat)
	if len(filtered) != 2 {
		t.Errorf("Expected 2 storage services, got %d", len(filtered))
	}
}

func TestDiscover(t *testing.T) {
	r := NewRegistry()
	r.Register(&mockProvider{id: "storage"})

	results := r.Discover("storage read write", 5)
	if len(results) == 0 {
		t.Error("Should discover storage service")
	}

	if results[0].ID != "storage" {
		t.Errorf("Expected storage service, got %s", results[0].ID)
	}
}

func TestExecute(t *testing.T) {
	r := NewRegistry()
	r.Register(&mockProvider{id: "test"})

	ctx := context.Background()
	result, err := r.Execute(ctx, "test.test", map[string]interface{}{}, nil)
	if err != nil {
		t.Fatalf("Execute failed: %v", err)
	}

	if !result.Success {
		t.Error("Expected successful execution")
	}
}

func TestStats(t *testing.T) {
	r := NewRegistry()
	r.Register(&mockProvider{id: "test1"})
	r.Register(&mockProvider{id: "test2"})

	stats := r.Stats()
	totalServices := stats["total_services"].(int)
	if totalServices != 2 {
		t.Errorf("Expected 2 total services, got %d", totalServices)
	}

	totalTools := stats["total_tools"].(int)
	if totalTools != 2 {
		t.Errorf("Expected 2 total tools, got %d", totalTools)
	}
}
