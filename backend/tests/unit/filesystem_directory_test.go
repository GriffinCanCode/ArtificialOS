package unit

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// TestDirectoryOpsGetTools tests the directory operations tool definitions
func TestDirectoryOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	tools := dir.GetTools()

	assert.Equal(t, 7, len(tools))

	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.dir.list"])
	assert.True(t, toolIDs["filesystem.dir.create"])
	assert.True(t, toolIDs["filesystem.dir.delete"])
	assert.True(t, toolIDs["filesystem.dir.exists"])
	assert.True(t, toolIDs["filesystem.dir.walk"])
	assert.True(t, toolIDs["filesystem.dir.tree"])
	assert.True(t, toolIDs["filesystem.dir.flatten"])
}

// TestDirectoryOpsList tests the List operation
func TestDirectoryOpsList(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	ctx := context.Background()
	// Kernel returns entries with metadata
	entries := []map[string]interface{}{
		{"name": "file1.txt", "is_dir": false, "type": "file"},
		{"name": "file2.txt", "is_dir": false, "type": "file"},
		{"name": "dir1", "is_dir": true, "type": "directory"},
	}
	entriesJSON, _ := json.Marshal(entries)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "list_directory", mock.Anything).
		Return(entriesJSON, nil)

	result, err := dir.List(ctx, map[string]interface{}{
		"path": "/test",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "/test", result.Data["path"])
	assert.Equal(t, 3, result.Data["count"])

	returnedFiles := result.Data["files"].([]string)
	expectedFiles := []string{"file1.txt", "file2.txt", "dir1"}
	assert.Equal(t, expectedFiles, returnedFiles)

	returnedEntries := result.Data["entries"].([]map[string]interface{})
	assert.Equal(t, 3, len(returnedEntries))
}

// TestDirectoryOpsCreate tests the Create operation
func TestDirectoryOpsCreate(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "create_directory", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "/test/newdir"
	})).Return([]byte("{}"), nil)

	result, err := dir.Create(ctx, map[string]interface{}{
		"path": "/test/newdir",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["created"])
	assert.Equal(t, "/test/newdir", result.Data["path"])
}

// TestDirectoryOpsDelete tests the Delete operation
func TestDirectoryOpsDelete(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "delete_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "/test/dir"
	})).Return([]byte("{}"), nil)

	result, err := dir.Delete(ctx, map[string]interface{}{
		"path": "/test/dir",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["deleted"])
}

// TestDirectoryOpsExists tests the Exists operation
func TestDirectoryOpsExists(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	ctx := context.Background()

	statInfo := map[string]interface{}{
		"path":   "/test/dir",
		"is_dir": true,
		"size":   float64(0),
	}
	statJSON, _ := json.Marshal(statInfo)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.Anything).
		Return(statJSON, nil)

	result, err := dir.Exists(ctx, map[string]interface{}{
		"path": "/test/dir",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["exists"])
	assert.Equal(t, true, result.Data["is_dir"])
}

// TestDirectoryOpsErrorHandling tests error handling for directory operations
func TestDirectoryOpsErrorHandling(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	dir := &filesystem.DirectoryOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "list without path",
			fn: func() (*types.Result, error) {
				return dir.List(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "create without path",
			fn: func() (*types.Result, error) {
				return dir.Create(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "delete without path",
			fn: func() (*types.Result, error) {
				return dir.Delete(ctx, map[string]interface{}{}, nil)
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
