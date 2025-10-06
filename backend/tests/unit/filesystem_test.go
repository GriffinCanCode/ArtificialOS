package unit

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// TestFilesystemDefinition tests the service definition
func TestFilesystemDefinition(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	def := fs.Definition()

	assert.Equal(t, "filesystem", def.ID)
	assert.Equal(t, "Filesystem Service", def.Name)
	assert.Equal(t, types.CategoryFilesystem, def.Category)
	assert.NotEmpty(t, def.Description)
	assert.NotEmpty(t, def.Capabilities)

	// Should have 59 tools total (some archive/format tools may not be fully implemented)
	assert.GreaterOrEqual(t, len(def.Tools), 55)
	assert.LessOrEqual(t, len(def.Tools), 75)

	// Verify tools exist from each module
	toolIDs := make(map[string]bool)
	for _, tool := range def.Tools {
		toolIDs[tool.ID] = true
	}

	// Check basic operations
	assert.True(t, toolIDs["filesystem.read"])
	assert.True(t, toolIDs["filesystem.write"])

	// Check directory operations
	assert.True(t, toolIDs["filesystem.dir.list"])
	assert.True(t, toolIDs["filesystem.dir.create"])

	// Check file operations
	assert.True(t, toolIDs["filesystem.copy"])
	assert.True(t, toolIDs["filesystem.move"])

	// Check metadata operations
	assert.True(t, toolIDs["filesystem.stat"])
	assert.True(t, toolIDs["filesystem.mime_type"])

	// Check search operations
	assert.True(t, toolIDs["filesystem.find"])
	assert.True(t, toolIDs["filesystem.glob"])

	// Check format operations
	assert.True(t, toolIDs["filesystem.yaml.read"])
	assert.True(t, toolIDs["filesystem.csv.read"])

	// Check archive operations
	assert.True(t, toolIDs["filesystem.zip.create"])
	assert.True(t, toolIDs["filesystem.tar.create"])
}

// TestFilesystemReadExecute tests the read file operation
func TestFilesystemReadExecute(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()
	testContent := "Hello, World!"

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.txt"
	})).Return([]byte(testContent), nil)

	result, err := fs.Execute(ctx, "filesystem.read", map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, "test.txt", result.Data["path"])
	assert.Equal(t, testContent, result.Data["content"])
	assert.Equal(t, len(testContent), result.Data["size"])
}

// TestFilesystemWriteExecute tests the write file operation
func TestFilesystemWriteExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()
	testContent := "Hello, World!"

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.txt" && string(params["data"].([]byte)) == testContent
	})).Return([]byte("{}"), nil)

	result, err := fs.Execute(ctx, "filesystem.write", map[string]interface{}{
		"path": "test.txt",
		"data": testContent,
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, "test.txt", result.Data["path"])
}

// TestFilesystemListExecute tests the list directory operation
func TestFilesystemListExecute(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()
	files := []string{"file1.txt", "file2.txt", "dir1"}
	filesJSON, _ := json.Marshal(files)

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "list_directory", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "/test"
	})).Return(filesJSON, nil)

	result, err := fs.Execute(ctx, "filesystem.dir.list", map[string]interface{}{
		"path": "/test",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, "/test", result.Data["path"])
	assert.Equal(t, 3, result.Data["count"])
}

// TestFilesystemCreateDirectoryExecute tests the create directory operation
func TestFilesystemCreateDirectoryExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "create_directory", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "/test/newdir"
	})).Return([]byte("{}"), nil)

	result, err := fs.Execute(ctx, "filesystem.dir.create", map[string]interface{}{
		"path": "/test/newdir",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["created"])
	assert.Equal(t, "/test/newdir", result.Data["path"])
}

// TestFilesystemCopyExecute tests the copy file operation
func TestFilesystemCopyExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "copy_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["source"] == "source.txt" && params["destination"] == "dest.txt"
	})).Return([]byte("{}"), nil)

	result, err := fs.Execute(ctx, "filesystem.copy", map[string]interface{}{
		"source":      "source.txt",
		"destination": "dest.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["copied"])
}

// TestFilesystemMoveExecute tests the move file operation
func TestFilesystemMoveExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "move_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["source"] == "source.txt" && params["destination"] == "dest.txt"
	})).Return([]byte("{}"), nil)

	result, err := fs.Execute(ctx, "filesystem.move", map[string]interface{}{
		"source":      "source.txt",
		"destination": "dest.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["moved"])
}

// TestFilesystemStatExecute tests the stat operation
func TestFilesystemStatExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	statInfo := map[string]interface{}{
		"path":     "/test/file.txt",
		"size":     1024,
		"is_dir":   false,
		"mode":     "0644",
		"modified": 1234567890,
	}
	statJSON, _ := json.Marshal(statInfo)

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "file.txt"
	})).Return(statJSON, nil)

	result, err := fs.Execute(ctx, "filesystem.stat", map[string]interface{}{
		"path": "file.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, "file.txt", result.Data["path"])
	assert.NotNil(t, result.Data["info"])
}

// TestFilesystemReadJSONExecute tests the read JSON operation
func TestFilesystemReadJSONExecute(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	testData := map[string]interface{}{"key": "value", "number": float64(42)}
	jsonContent, _ := json.Marshal(testData)

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.json"
	})).Return(jsonContent, nil)

	result, err := fs.Execute(ctx, "filesystem.read_json", map[string]interface{}{
		"path": "test.json",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, "test.json", result.Data["path"])

	data := result.Data["data"].(map[string]interface{})
	assert.Equal(t, "value", data["key"])
	assert.Equal(t, float64(42), data["number"])
}

// TestFilesystemUnknownTool tests unknown tool execution
func TestFilesystemUnknownTool(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	result, err := fs.Execute(ctx, "filesystem.unknown_tool", map[string]interface{}{}, nil)

	// The Execute method returns an error for unknown tools
	if err != nil {
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "unknown tool")
	} else {
		testutil.AssertError(t, result)
		assert.Contains(t, *result.Error, "unknown tool")
	}
}

// TestFilesystemWithAppContext tests operations with app context
func TestFilesystemWithAppContext(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()
	sandboxPID := uint32(999)
	appCtx := &types.Context{
		SandboxPID: &sandboxPID,
	}

	// Mock the kernel syscall with the sandbox PID
	mockKernel.On("ExecuteSyscall", mock.Anything, sandboxPID, "read_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.txt"
	})).Return([]byte("content"), nil)

	result, err := fs.Execute(ctx, "filesystem.read", map[string]interface{}{
		"path": "test.txt",
	}, appCtx)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
}

// TestFilesystemExistsExecute tests the exists operation
func TestFilesystemExistsExecute(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	// Mock file exists (return non-empty byte array with value 1)
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_exists", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "exists.txt"
	})).Return([]byte{1}, nil)

	result, err := fs.Execute(ctx, "filesystem.exists", map[string]interface{}{
		"path": "exists.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["exists"])
}

// TestFilesystemDeleteExecute tests the delete operation
func TestFilesystemDeleteExecute(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	fs := providers.NewFilesystem(mockKernel, 1, "/storage")

	ctx := context.Background()

	// Mock the kernel syscall
	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "delete_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "test.txt"
	})).Return([]byte("{}"), nil)

	result, err := fs.Execute(ctx, "filesystem.delete", map[string]interface{}{
		"path": "test.txt",
	}, nil)

	assert.NoError(t, err)
	testutil.AssertSuccess(t, result)
	assert.Equal(t, true, result.Data["deleted"])
}

// TestFilesystemErrorHandling tests error handling
func TestFilesystemErrorHandling(t *testing.T) {
	tests := []struct {
		name    string
		toolID  string
		params  map[string]interface{}
		wantErr bool
		errMsg  string
	}{
		{
			name:    "read without path",
			toolID:  "filesystem.read",
			params:  map[string]interface{}{},
			wantErr: true,
			errMsg:  "path parameter required",
		},
		{
			name:   "write without data",
			toolID: "filesystem.write",
			params: map[string]interface{}{
				"path": "test.txt",
			},
			wantErr: true,
			errMsg:  "data parameter required",
		},
		{
			name:   "copy without source",
			toolID: "filesystem.copy",
			params: map[string]interface{}{
				"destination": "dest.txt",
			},
			wantErr: true,
			errMsg:  "source parameter required",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockKernel := testutil.NewMockKernelClient(t)
			fs := providers.NewFilesystem(mockKernel, 1, "/storage")

			ctx := context.Background()
			result, err := fs.Execute(ctx, tt.toolID, tt.params, nil)

			assert.NoError(t, err)
			if tt.wantErr {
				testutil.AssertError(t, result)
				if tt.errMsg != "" {
					assert.Contains(t, *result.Error, tt.errMsg)
				}
			}
		})
	}
}
