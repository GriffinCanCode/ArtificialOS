package unit

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// TestBasicOpsGetTools tests the basic operations tool definitions
func TestBasicOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	tools := basic.GetTools()

	assert.Equal(t, 12, len(tools))

	// Check specific tools
	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.read"])
	assert.True(t, toolIDs["filesystem.write"])
	assert.True(t, toolIDs["filesystem.append"])
	assert.True(t, toolIDs["filesystem.create"])
	assert.True(t, toolIDs["filesystem.delete"])
	assert.True(t, toolIDs["filesystem.exists"])
	assert.True(t, toolIDs["filesystem.read_lines"])
	assert.True(t, toolIDs["filesystem.read_json"])
	assert.True(t, toolIDs["filesystem.write_json"])
	assert.True(t, toolIDs["filesystem.read_binary"])
	assert.True(t, toolIDs["filesystem.write_binary"])
	assert.True(t, toolIDs["filesystem.write_lines"])
}

// TestBasicOpsRead tests the Read operation
func TestBasicOpsRead(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()
	testContent := "Hello, World!"

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return([]byte(testContent), nil)

	result, err := basic.Read(ctx, map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "test.txt", result.Data["path"])
	assert.Equal(t, testContent, result.Data["content"])
	assert.Equal(t, len(testContent), result.Data["size"])
}

// TestBasicOpsWrite tests the Write operation
func TestBasicOpsWrite(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()
	testContent := "Hello, World!"

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.txt" && string(params["data"].([]byte)) == testContent
	})).Return([]byte("{}"), nil)

	result, err := basic.Write(ctx, map[string]interface{}{
		"path": "test.txt",
		"data": testContent,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, "test.txt", result.Data["path"])
	assert.Equal(t, len(testContent), result.Data["size"])
}

// TestBasicOpsAppend tests the Append operation
func TestBasicOpsAppend(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()
	existingContent := "Hello, "
	appendContent := "World!"

	// Mock read existing content
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "read_file", mock.Anything).
		Return([]byte(existingContent), nil).Once()

	// Mock write combined content
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return string(params["data"].([]byte)) == existingContent+appendContent
	})).Return([]byte("{}"), nil).Once()

	result, err := basic.Append(ctx, map[string]interface{}{
		"path": "test.txt",
		"data": appendContent,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["appended"])
}

// TestBasicOpsCreate tests the Create operation
func TestBasicOpsCreate(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "create_file", mock.Anything).
		Return([]byte("{}"), nil)

	result, err := basic.Create(ctx, map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["created"])
}

// TestBasicOpsDelete tests the Delete operation
func TestBasicOpsDelete(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "delete_file", mock.Anything).
		Return([]byte("{}"), nil)

	result, err := basic.Delete(ctx, map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["deleted"])
}

// TestBasicOpsExists tests the Exists operation
func TestBasicOpsExists(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	// Test file exists
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_exists", mock.Anything).
		Return([]byte{1}, nil).Once()

	result, err := basic.Exists(ctx, map[string]interface{}{
		"path": "exists.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["exists"])

	// Test file doesn't exist
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_exists", mock.Anything).
		Return([]byte{0}, nil).Once()

	result, err = basic.Exists(ctx, map[string]interface{}{
		"path": "notexists.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, false, result.Data["exists"])
}

// TestBasicOpsReadLines tests the ReadLines operation
func TestBasicOpsReadLines(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()
	testContent := "line1\nline2\nline3"

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return([]byte(testContent), nil)

	result, err := basic.ReadLines(ctx, map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)

	lines := result.Data["lines"].([]string)
	assert.Equal(t, 3, len(lines))
	assert.Equal(t, "line1", lines[0])
	assert.Equal(t, "line2", lines[1])
	assert.Equal(t, "line3", lines[2])
}

// TestBasicOpsReadJSON tests the ReadJSON operation
func TestBasicOpsReadJSON(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	testData := map[string]interface{}{"key": "value", "number": float64(42)}
	jsonContent, _ := json.Marshal(testData)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(jsonContent, nil)

	result, err := basic.ReadJSON(ctx, map[string]interface{}{
		"path": "test.json",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)

	data := result.Data["data"].(map[string]interface{})
	assert.Equal(t, "value", data["key"])
	assert.Equal(t, float64(42), data["number"])
}

// TestBasicOpsWriteJSON tests the WriteJSON operation
func TestBasicOpsWriteJSON(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	testData := map[string]interface{}{"key": "value"}

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.Anything).
		Return([]byte("{}"), nil)

	result, err := basic.WriteJSON(ctx, map[string]interface{}{
		"path": "test.json",
		"data": testData,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
}

// TestBasicOpsWriteLines tests the WriteLines operation
func TestBasicOpsWriteLines(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	lines := []interface{}{"line1", "line2", "line3"}

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		content := string(params["data"].([]byte))
		return content == "line1\nline2\nline3"
	})).Return([]byte("{}"), nil)

	result, err := basic.WriteLines(ctx, map[string]interface{}{
		"path":  "test.txt",
		"lines": lines,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, 3, result.Data["lines"])
}

// TestBasicOpsErrorHandling tests error handling for basic operations
func TestBasicOpsErrorHandling(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	basic := &filesystem.BasicOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "read without path",
			fn: func() (*types.Result, error) {
				return basic.Read(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "write without data",
			fn: func() (*types.Result, error) {
				return basic.Write(ctx, map[string]interface{}{"path": "test.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "write_json without data",
			fn: func() (*types.Result, error) {
				return basic.WriteJSON(ctx, map[string]interface{}{"path": "test.json"}, nil)
			},
			wantErr: true,
		},
		{
			name: "write_lines without lines",
			fn: func() (*types.Result, error) {
				return basic.WriteLines(ctx, map[string]interface{}{"path": "test.txt"}, nil)
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := tt.fn()
			assert.NoError(t, err)
			if tt.wantErr {
				assert.False(t, result.Success)
				assert.NotNil(t, result.Error)
			}
		})
	}
}
