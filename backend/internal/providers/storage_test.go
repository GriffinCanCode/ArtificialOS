package providers

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
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
	path := params["path"].(string)

	switch syscallType {
	case "write_file":
		data := params["data"].([]byte)
		m.storage[path] = data
		return nil, nil
	case "read_file":
		if data, ok := m.storage[path]; ok {
			return data, nil
		}
		return nil, &MockError{"file not found"}
	case "delete_file":
		delete(m.storage, path)
		return nil, nil
	}

	return nil, &MockError{"unknown syscall"}
}

type MockError struct {
	msg string
}

func (e *MockError) Error() string {
	return e.msg
}

func TestStorageSetGet(t *testing.T) {
	kernel := newMockKernel()
	storage := NewStorage(kernel, 1, "/tmp/test")

	bgCtx := context.Background()
	appID := "test-app"
	ctx := &types.Context{AppID: &appID}

	// Set a value
	result, err := storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "username",
		"value": "john_doe",
	}, ctx)

	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	if !result.Success {
		t.Fatalf("Set result not successful")
	}

	// Get the value
	result, err = storage.Execute(bgCtx, "storage.get", map[string]interface{}{
		"key": "username",
	}, ctx)

	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}

	if !result.Success {
		t.Fatalf("Get result not successful")
	}

	value := result.Data["value"]
	if value != "john_doe" {
		t.Errorf("Expected 'john_doe', got %v", value)
	}
}

func TestStorageComplex(t *testing.T) {
	kernel := newMockKernel()
	storage := NewStorage(kernel, 1, "/tmp/test")

	bgCtx := context.Background()
	appID := "test-app"
	ctx := &types.Context{AppID: &appID}

	// Store complex object
	complexValue := map[string]interface{}{
		"name":  "Alice",
		"age":   30,
		"items": []string{"a", "b", "c"},
	}

	result, err := storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "user_data",
		"value": complexValue,
	}, ctx)

	if err != nil || !result.Success {
		t.Fatalf("Set complex failed")
	}

	// Retrieve and verify
	result, err = storage.Execute(bgCtx, "storage.get", map[string]interface{}{
		"key": "user_data",
	}, ctx)

	if err != nil || !result.Success {
		t.Fatalf("Get complex failed")
	}

	retrieved := result.Data["value"].(map[string]interface{})
	if retrieved["name"] != "Alice" {
		t.Errorf("Expected name 'Alice', got %v", retrieved["name"])
	}
}

func TestStorageRemove(t *testing.T) {
	kernel := newMockKernel()
	storage := NewStorage(kernel, 1, "/tmp/test")

	bgCtx := context.Background()
	appID := "test-app"
	ctx := &types.Context{AppID: &appID}

	// Set then remove
	storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "temp",
		"value": "data",
	}, ctx)

	result, err := storage.Execute(bgCtx, "storage.remove", map[string]interface{}{
		"key": "temp",
	}, ctx)

	if err != nil || !result.Success {
		t.Fatalf("Remove failed")
	}

	// Verify removed
	result, err = storage.Execute(bgCtx, "storage.get", map[string]interface{}{
		"key": "temp",
	}, ctx)

	if err != nil || !result.Success {
		t.Fatalf("Get after remove failed")
	}

	if result.Data["value"] != nil {
		t.Errorf("Expected nil after remove, got %v", result.Data["value"])
	}
}

func TestStorageClear(t *testing.T) {
	kernel := newMockKernel()
	storage := NewStorage(kernel, 1, "/tmp/test")

	bgCtx := context.Background()
	appID := "test-app"
	ctx := &types.Context{AppID: &appID}

	// Set multiple values
	storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "key1",
		"value": "val1",
	}, ctx)
	storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "key2",
		"value": "val2",
	}, ctx)

	// Clear all
	result, err := storage.Execute(bgCtx, "storage.clear", nil, ctx)

	if err != nil {
		t.Fatalf("Clear error: %v", err)
	}
	if !result.Success {
		t.Fatalf("Clear failed")
	}

	count := int(result.Data["count"].(int))
	if count != 2 {
		t.Errorf("Expected 2 cleared items, got %d", count)
	}
}

func TestStorageNoContext(t *testing.T) {
	kernel := newMockKernel()
	storage := NewStorage(kernel, 1, "/tmp/test")

	bgCtx := context.Background()
	result, _ := storage.Execute(bgCtx, "storage.set", map[string]interface{}{
		"key":   "test",
		"value": "data",
	}, nil)

	if result.Success {
		t.Error("Expected failure without context")
	}
}
