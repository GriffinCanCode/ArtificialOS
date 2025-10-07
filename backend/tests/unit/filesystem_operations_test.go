package unit

import (
	"context"
	"os"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// TestOperationsOpsGetTools tests the file operations tool definitions
func TestOperationsOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	tools := operations.GetTools()

	assert.Equal(t, 6, len(tools))

	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.copy"])
	assert.True(t, toolIDs["filesystem.move"])
	assert.True(t, toolIDs["filesystem.rename"])
	assert.True(t, toolIDs["filesystem.symlink"])
	assert.True(t, toolIDs["filesystem.readlink"])
	assert.True(t, toolIDs["filesystem.hardlink"])
}

// TestOperationsOpsCopy tests the Copy operation
func TestOperationsOpsCopy(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "copy_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["source"] == "/test/source.txt" && params["destination"] == "/test/dest.txt"
	})).Return([]byte("{}"), nil)

	result, err := operations.Copy(ctx, map[string]interface{}{
		"source":      "/test/source.txt",
		"destination": "/test/dest.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["copied"])
	assert.Equal(t, "/test/source.txt", result.Data["source"])
	assert.Equal(t, "/test/dest.txt", result.Data["destination"])
}

// TestOperationsOpsMove tests the Move operation
func TestOperationsOpsMove(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "move_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["source"] == "/test/old.txt" && params["destination"] == "/test/new.txt"
	})).Return([]byte("{}"), nil)

	result, err := operations.Move(ctx, map[string]interface{}{
		"source":      "/test/old.txt",
		"destination": "/test/new.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["moved"])
	assert.Equal(t, "/test/old.txt", result.Data["source"])
	assert.Equal(t, "/test/new.txt", result.Data["destination"])
}

// TestOperationsOpsRename tests the Rename operation
func TestOperationsOpsRename(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "move_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["source"] == "/test/old.txt" && params["destination"] == "/test/renamed.txt"
	})).Return([]byte("{}"), nil)

	result, err := operations.Rename(ctx, map[string]interface{}{
		"path":     "/test/old.txt",
		"new_name": "renamed.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["renamed"])
	assert.Equal(t, "/test/old.txt", result.Data["old_path"])
	assert.Equal(t, "/test/renamed.txt", result.Data["new_path"])
}

// TestOperationsOpsSymlink tests the Symlink operation
func TestOperationsOpsSymlink(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)

	// Create a temporary directory for testing
	tmpDir := t.TempDir()

	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: tmpDir,
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	// Create a target file
	targetPath := "/test/target.txt"
	linkPath := "/test/link.txt"

	// Mock the resolvePath calls for both target and link
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		path, ok := params["path"].(string)
		return ok && path == targetPath
	})).Return([]byte(`{"path":"`+tmpDir+`/target.txt"}`), nil)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		path, ok := params["path"].(string)
		return ok && path == "/test"
	})).Return([]byte(`{"path":"`+tmpDir+`"}`), nil)

	// Create the actual target file
	targetFile := tmpDir + "/target.txt"
	if err := os.WriteFile(targetFile, []byte("test content"), 0644); err != nil {
		t.Fatal(err)
	}

	result, err := operations.Symlink(ctx, map[string]interface{}{
		"target": targetPath,
		"link":   linkPath,
	}, nil)

	assert.NoError(t, err)
	if !assert.True(t, result.Success) {
		if result.Error != nil {
			t.Logf("Operation failed with error: %s", *result.Error)
		}
		return
	}
	assert.Equal(t, true, result.Data["created"])

	// Verify symlink was created
	linkFile := tmpDir + "/link.txt"
	info, err := os.Lstat(linkFile)
	if assert.NoError(t, err) {
		assert.Equal(t, os.ModeSymlink, info.Mode()&os.ModeSymlink)
	}
}

// TestOperationsOpsReadlink tests the Readlink operation
func TestOperationsOpsReadlink(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)

	// Create a temporary directory for testing
	tmpDir := t.TempDir()

	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: tmpDir,
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	// Create a target file and symlink
	targetFile := tmpDir + "/target.txt"
	linkFile := tmpDir + "/link.txt"

	if err := os.WriteFile(targetFile, []byte("test content"), 0644); err != nil {
		t.Fatal(err)
	}

	if err := os.Symlink(targetFile, linkFile); err != nil {
		t.Fatal(err)
	}

	linkPath := "/test/link.txt"

	// Mock the resolvePath call
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		path, ok := params["path"].(string)
		return ok && path == linkPath
	})).Return([]byte(`{"path":"`+linkFile+`"}`), nil)

	result, err := operations.Readlink(ctx, map[string]interface{}{
		"path": linkPath,
	}, nil)

	assert.NoError(t, err)
	if !assert.True(t, result.Success) {
		if result.Error != nil {
			t.Logf("Operation failed with error: %s", *result.Error)
		}
		return
	}
	assert.Equal(t, targetFile, result.Data["target"])
	assert.Equal(t, linkPath, result.Data["path"])
}

// TestOperationsOpsHardlink tests the Hardlink operation
func TestOperationsOpsHardlink(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)

	// Create a temporary directory for testing
	tmpDir := t.TempDir()

	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: tmpDir,
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	// Create a target file
	targetPath := "/test/target.txt"
	linkPath := "/test/hardlink.txt"

	// Mock the resolvePath calls
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		path, ok := params["path"].(string)
		return ok && path == targetPath
	})).Return([]byte(`{"path":"`+tmpDir+`/target.txt"}`), nil)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.MatchedBy(func(params map[string]interface{}) bool {
		path, ok := params["path"].(string)
		return ok && path == "/test"
	})).Return([]byte(`{"path":"`+tmpDir+`"}`), nil)

	// Create the actual target file
	targetFile := tmpDir + "/target.txt"
	if err := os.WriteFile(targetFile, []byte("test content"), 0644); err != nil {
		t.Fatal(err)
	}

	result, err := operations.Hardlink(ctx, map[string]interface{}{
		"target": targetPath,
		"link":   linkPath,
	}, nil)

	assert.NoError(t, err)
	if !assert.True(t, result.Success) {
		if result.Error != nil {
			t.Logf("Operation failed with error: %s", *result.Error)
		}
		return
	}
	assert.Equal(t, true, result.Data["created"])

	// Verify hardlink was created
	linkFile := tmpDir + "/hardlink.txt"
	info, err := os.Stat(linkFile)
	if !assert.NoError(t, err) {
		return
	}
	assert.False(t, info.Mode()&os.ModeSymlink == os.ModeSymlink) // Not a symlink

	// Both files should have the same content
	content, err := os.ReadFile(linkFile)
	assert.NoError(t, err)
	assert.Equal(t, "test content", string(content))
}

// TestOperationsOpsErrorHandling tests error handling for file operations
func TestOperationsOpsErrorHandling(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	operations := &filesystem.OperationsOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "copy without source",
			fn: func() (*types.Result, error) {
				return operations.Copy(ctx, map[string]interface{}{"destination": "/test/dest.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "copy without destination",
			fn: func() (*types.Result, error) {
				return operations.Copy(ctx, map[string]interface{}{"source": "/test/source.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "move without source",
			fn: func() (*types.Result, error) {
				return operations.Move(ctx, map[string]interface{}{"destination": "/test/dest.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "move without destination",
			fn: func() (*types.Result, error) {
				return operations.Move(ctx, map[string]interface{}{"source": "/test/source.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "rename without path",
			fn: func() (*types.Result, error) {
				return operations.Rename(ctx, map[string]interface{}{"new_name": "new.txt"}, nil)
			},
			wantErr: true,
		},
		{
			name: "rename without new_name",
			fn: func() (*types.Result, error) {
				return operations.Rename(ctx, map[string]interface{}{"path": "/test/old.txt"}, nil)
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
