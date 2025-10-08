package unit

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/clipboard"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Mock kernel client for testing
type mockKernelClient struct {
	responses map[string][]byte
	calls     []mockCall
}

type mockCall struct {
	syscallType string
	params      map[string]interface{}
}

func (m *mockKernelClient) ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	m.calls = append(m.calls, mockCall{
		syscallType: syscallType,
		params:      params,
	})

	if response, ok := m.responses[syscallType]; ok {
		return response, nil
	}

	// Default response
	return []byte("{}"), nil
}

func TestClipboardProvider_Definition(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	def := provider.Definition()

	assert.Equal(t, "clipboard", def.ID)
	assert.Equal(t, "Clipboard Service", def.Name)
	assert.Equal(t, types.CategorySystem, def.Category)
	assert.NotEmpty(t, def.Capabilities)
	assert.NotEmpty(t, def.Tools)

	// Verify all tools are present
	toolIDs := make(map[string]bool)
	for _, tool := range def.Tools {
		toolIDs[tool.ID] = true
	}

	expectedTools := []string{
		"clipboard.copy",
		"clipboard.paste",
		"clipboard.history",
		"clipboard.get_entry",
		"clipboard.clear",
		"clipboard.subscribe",
		"clipboard.unsubscribe",
		"clipboard.stats",
	}

	for _, toolID := range expectedTools {
		assert.True(t, toolIDs[toolID], "Missing tool: %s", toolID)
	}
}

func TestClipboardProvider_Copy(t *testing.T) {
	kernel := &mockKernelClient{
		responses: map[string][]byte{
			"clipboard_copy": mustMarshal(uint64(123)),
		},
	}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.copy", map[string]interface{}{
		"data":   "Hello, World!",
		"format": "text",
		"global": false,
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, uint64(123), result.Data["entry_id"])
	assert.Equal(t, "text", result.Data["format"])

	// Verify kernel was called
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_copy", kernel.calls[0].syscallType)
}

func TestClipboardProvider_CopyMissingData(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.copy", map[string]interface{}{}, nil)

	require.Error(t, err)
	assert.False(t, result.Success)
	assert.NotNil(t, result.Error)
}

func TestClipboardProvider_Paste(t *testing.T) {
	entry := map[string]interface{}{
		"id":         float64(123),
		"data":       map[string]interface{}{"type": "Text", "data": "Hello!"},
		"source_pid": float64(100),
		"timestamp":  float64(1234567890),
	}

	kernel := &mockKernelClient{
		responses: map[string][]byte{
			"clipboard_paste": mustMarshal(entry),
		},
	}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.paste", map[string]interface{}{
		"global": false,
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, float64(123), result.Data["id"])

	// Verify kernel was called
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_paste", kernel.calls[0].syscallType)
}

func TestClipboardProvider_History(t *testing.T) {
	entries := []map[string]interface{}{
		{
			"id":         float64(1),
			"data":       map[string]interface{}{"type": "Text", "data": "First"},
			"source_pid": float64(100),
			"timestamp":  float64(1234567890),
		},
		{
			"id":         float64(2),
			"data":       map[string]interface{}{"type": "Text", "data": "Second"},
			"source_pid": float64(100),
			"timestamp":  float64(1234567891),
		},
	}

	kernel := &mockKernelClient{
		responses: map[string][]byte{
			"clipboard_history": mustMarshal(entries),
		},
	}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.history", map[string]interface{}{
		"limit":  float64(10),
		"global": false,
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, 2, result.Data["count"])

	entriesData := result.Data["entries"].([]map[string]interface{})
	assert.Len(t, entriesData, 2)

	// Verify kernel was called with correct params
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_history", kernel.calls[0].syscallType)
	assert.Equal(t, 10, kernel.calls[0].params["limit"])
}

func TestClipboardProvider_GetEntry(t *testing.T) {
	entry := map[string]interface{}{
		"id":         float64(123),
		"data":       map[string]interface{}{"type": "Text", "data": "Test"},
		"source_pid": float64(100),
		"timestamp":  float64(1234567890),
	}

	kernel := &mockKernelClient{
		responses: map[string][]byte{
			"clipboard_get_entry": mustMarshal(entry),
		},
	}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.get_entry", map[string]interface{}{
		"entry_id": float64(123),
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, float64(123), result.Data["id"])
}

func TestClipboardProvider_Clear(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.clear", map[string]interface{}{
		"global": true,
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.True(t, result.Data["cleared"].(bool))

	// Verify kernel was called
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_clear", kernel.calls[0].syscallType)
	assert.True(t, kernel.calls[0].params["global"].(bool))
}

func TestClipboardProvider_Subscribe(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.subscribe", map[string]interface{}{
		"formats": []interface{}{"text", "html"},
	}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.True(t, result.Data["subscribed"].(bool))

	// Verify kernel was called with formats
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_subscribe", kernel.calls[0].syscallType)
	formats := kernel.calls[0].params["formats"].([]string)
	assert.Contains(t, formats, "text")
	assert.Contains(t, formats, "html")
}

func TestClipboardProvider_Unsubscribe(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.unsubscribe", map[string]interface{}{}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.True(t, result.Data["unsubscribed"].(bool))

	// Verify kernel was called
	require.Len(t, kernel.calls, 1)
	assert.Equal(t, "clipboard_unsubscribe", kernel.calls[0].syscallType)
}

func TestClipboardProvider_Stats(t *testing.T) {
	stats := map[string]interface{}{
		"total_entries":  float64(10),
		"total_size":     float64(1024),
		"process_count":  float64(2),
		"global_entries": float64(1),
		"subscriptions":  float64(0),
	}

	kernel := &mockKernelClient{
		responses: map[string][]byte{
			"clipboard_stats": mustMarshal(stats),
		},
	}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.stats", map[string]interface{}{}, nil)

	require.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, float64(10), result.Data["total_entries"])
	assert.Equal(t, float64(1024), result.Data["total_size"])
}

func TestClipboardProvider_UnknownTool(t *testing.T) {
	kernel := &mockKernelClient{}
	provider := clipboard.NewProvider(kernel, 1)

	result, err := provider.Execute(context.Background(), "clipboard.unknown", map[string]interface{}{}, nil)

	require.Error(t, err)
	assert.False(t, result.Success)
}

// Helper function to marshal data
func mustMarshal(v interface{}) []byte {
	data, err := json.Marshal(v)
	if err != nil {
		panic(err)
	}
	return data
}
