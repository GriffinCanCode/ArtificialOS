// Package testutil provides testing utilities and helpers for backend tests.
package testutil

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/stretchr/testify/mock"
)

// MockKernelClient is a mock implementation of KernelClient for testing.
type MockKernelClient struct {
	mock.Mock
}

// CreateProcess mocks the CreateProcess method.
func (m *MockKernelClient) CreateProcess(ctx context.Context, name string, priority uint32, sandboxLevel string) (*uint32, error) {
	args := m.Called(ctx, name, priority, sandboxLevel)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*uint32), args.Error(1)
}

// ExecuteSyscall mocks the ExecuteSyscall method.
func (m *MockKernelClient) ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	args := m.Called(ctx, pid, syscallType, params)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]byte), args.Error(1)
}

// MockServiceProvider is a mock implementation of service.Provider for testing.
type MockServiceProvider struct {
	mock.Mock
}

// Definition mocks the Definition method.
func (m *MockServiceProvider) Definition() types.Service {
	args := m.Called()
	return args.Get(0).(types.Service)
}

// Execute mocks the Execute method.
func (m *MockServiceProvider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	args := m.Called(ctx, toolID, params, appCtx)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*types.Result), args.Error(1)
}

// NewMockKernelClient creates a new mock kernel client with default behaviors.
func NewMockKernelClient(t *testing.T) *MockKernelClient {
	t.Helper()
	m := new(MockKernelClient)

	// Default behavior: create process succeeds
	pid := uint32(123)
	m.On("CreateProcess", mock.Anything, mock.Anything, mock.Anything, mock.Anything).
		Return(&pid, nil).
		Maybe()

	// Default behavior: execute syscall returns empty JSON
	m.On("ExecuteSyscall", mock.Anything, mock.Anything, mock.Anything, mock.Anything).
		Return([]byte("{}"), nil).
		Maybe()

	return m
}

// NewMockServiceProvider creates a new mock service provider with default behaviors.
func NewMockServiceProvider(t *testing.T, serviceID string) *MockServiceProvider {
	t.Helper()
	m := new(MockServiceProvider)

	// Default behavior: return a simple service definition
	m.On("Definition").Return(types.Service{
		ID:          serviceID,
		Name:        "Mock Service",
		Description: "Mock service for testing",
		Category:    types.CategoryStorage,
		Tools:       []types.Tool{},
	}).Maybe()

	return m
}

// CreateTestApp creates a test app with default values.
func CreateTestApp(t *testing.T, overrides map[string]interface{}) *types.App {
	t.Helper()

	app := &types.App{
		ID:       "test-app-id",
		Hash:     "test-hash",
		Title:    "Test App",
		State:    types.StateActive,
		UISpec:   map[string]interface{}{"title": "Test App"},
		Metadata: map[string]interface{}{},
		Services: []string{},
	}

	// Apply overrides
	if title, ok := overrides["title"].(string); ok {
		app.Title = title
	}
	if state, ok := overrides["state"].(types.State); ok {
		app.State = state
	}
	if uiSpec, ok := overrides["ui_spec"].(map[string]interface{}); ok {
		app.UISpec = uiSpec
	}

	return app
}

// CreateTestService creates a test service definition.
func CreateTestService(t *testing.T, id string, category types.Category) types.Service {
	t.Helper()

	return types.Service{
		ID:           id,
		Name:         "Test Service",
		Description:  "A test service for unit testing",
		Category:     category,
		Capabilities: []string{"test"},
		Tools: []types.Tool{
			{
				ID:          id + ".test",
				Name:        "test",
				Description: "Test tool",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
		},
	}
}

// CreateTestPackage creates a test package for app registry.
func CreateTestPackage(t *testing.T, id string) *types.Package {
	t.Helper()

	return &types.Package{
		ID:          id,
		Name:        "Test Package",
		Description: "A test package",
		Icon:        "📦",
		Category:    "general",
		Version:     "1.0.0",
		Author:      "test",
		UISpec:      map[string]interface{}{"title": "Test App"},
		Services:    []string{},
		Permissions: []string{"STANDARD"},
		Tags:        []string{"test"},
	}
}

// AssertSuccess is a helper to assert a successful result.
func AssertSuccess(t *testing.T, result *types.Result) {
	t.Helper()
	if result == nil {
		t.Fatal("Result is nil")
	}
	if !result.Success {
		t.Fatalf("Expected success, got error: %v", result.Error)
	}
}

// AssertError is a helper to assert an error result.
func AssertError(t *testing.T, result *types.Result) {
	t.Helper()
	if result == nil {
		t.Fatal("Result is nil")
	}
	if result.Success {
		t.Fatal("Expected error, got success")
	}
	if result.Error == nil {
		t.Fatal("Expected error message, got nil")
	}
}

// AssertDataField is a helper to assert a data field exists and matches expected value.
func AssertDataField(t *testing.T, result *types.Result, field string, expected interface{}) {
	t.Helper()
	AssertSuccess(t, result)

	if result.Data == nil {
		t.Fatal("Result data is nil")
	}

	actual, ok := result.Data[field]
	if !ok {
		t.Fatalf("Field %s not found in result data", field)
	}

	if actual != expected {
		t.Fatalf("Field %s: expected %v, got %v", field, expected, actual)
	}
}
