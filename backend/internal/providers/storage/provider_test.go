package storage

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

type mockKernel struct {
	storage map[string][]byte
}

func newMockKernel() *mockKernel {
	return &mockKernel{
		storage: make(map[string][]byte),
	}
}

func (m *mockKernel) ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	switch syscallType {
	case "write_file":
		path := params["path"].(string)
		data := params["data"].([]byte)
		m.storage[path] = data
		return []byte{}, nil
	case "read_file":
		path := params["path"].(string)
		data, exists := m.storage[path]
		if !exists {
			return nil, &MockError{"file not found"}
		}
		return data, nil
	case "delete_file":
		path := params["path"].(string)
		delete(m.storage, path)
		return []byte{}, nil
	}
	return []byte{}, nil
}

type MockError struct {
	msg string
}

func (e *MockError) Error() string {
	return e.msg
}

func TestStorageSetGet(t *testing.T) {
	kernel := newMockKernel()
	storage := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()
	appCtx := &types.Context{AppID: strPtr("app1")}

	// Set a value
	result, err := storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "test_key",
		"value": "test_value",
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Set failed: %v", err)
	}

	// Get the value
	result, err = storage.Execute(ctx, "storage.get", map[string]interface{}{
		"key": "test_key",
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Get failed: %v", err)
	}

	if result.Data["value"].(string) != "test_value" {
		t.Errorf("Expected 'test_value', got %v", result.Data["value"])
	}
}

func TestStorageComplex(t *testing.T) {
	kernel := newMockKernel()
	storage := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()
	appCtx := &types.Context{AppID: strPtr("app2")}

	// Store complex object
	complexValue := map[string]interface{}{
		"name":   "test",
		"count":  42,
		"active": true,
		"tags":   []interface{}{"a", "b", "c"},
	}

	result, err := storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "complex",
		"value": complexValue,
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Set complex failed: %v", err)
	}

	// Retrieve complex object
	result, err = storage.Execute(ctx, "storage.get", map[string]interface{}{
		"key": "complex",
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Get complex failed: %v", err)
	}

	retrieved := result.Data["value"].(map[string]interface{})
	if retrieved["name"].(string) != "test" {
		t.Errorf("Expected name 'test', got %v", retrieved["name"])
	}
	// Handle both int and float64 types from JSON unmarshaling
	var count float64
	switch v := retrieved["count"].(type) {
	case int:
		count = float64(v)
	case float64:
		count = v
	default:
		t.Errorf("Expected count to be numeric, got %T", retrieved["count"])
	}
	if count != 42 {
		t.Errorf("Expected count 42, got %v", count)
	}
}

func TestStorageRemove(t *testing.T) {
	kernel := newMockKernel()
	storage := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()
	appCtx := &types.Context{AppID: strPtr("app3")}

	// Set a value
	storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "to_remove",
		"value": "will be deleted",
	}, appCtx)

	// Remove it
	result, err := storage.Execute(ctx, "storage.remove", map[string]interface{}{
		"key": "to_remove",
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Remove failed: %v", err)
	}

	// Try to get it (should return nil)
	result, err = storage.Execute(ctx, "storage.get", map[string]interface{}{
		"key": "to_remove",
	}, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Get after remove failed: %v", err)
	}

	if result.Data["value"] != nil {
		t.Errorf("Expected nil value after remove, got %v", result.Data["value"])
	}
}

func TestStorageClear(t *testing.T) {
	kernel := newMockKernel()
	storage := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()
	appCtx := &types.Context{AppID: strPtr("app4")}

	// Set multiple values
	storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "key1",
		"value": "value1",
	}, appCtx)
	storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "key2",
		"value": "value2",
	}, appCtx)

	// Clear all
	result, err := storage.Execute(ctx, "storage.clear", nil, appCtx)

	if err != nil || !result.Success {
		t.Fatalf("Clear failed: %v", err)
	}

	// Verify cleared
	result, _ = storage.Execute(ctx, "storage.get", map[string]interface{}{
		"key": "key1",
	}, appCtx)

	if result.Data["value"] != nil {
		t.Errorf("Expected nil after clear, got %v", result.Data["value"])
	}
}

func TestStorageNoContext(t *testing.T) {
	kernel := newMockKernel()
	storage := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	result, _ := storage.Execute(ctx, "storage.set", map[string]interface{}{
		"key":   "test",
		"value": "test",
	}, nil)

	if result.Success {
		t.Error("Expected failure without app context")
	}
}

func strPtr(s string) *string {
	return &s
}
