package unit

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
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
	// Note: This test uses direct OS calls, not kernel syscalls
	// In a real scenario, this might need kernel integration
	t.Skip("Symlink test requires filesystem access - implement with proper mocking")
}

// TestOperationsOpsReadlink tests the Readlink operation
func TestOperationsOpsReadlink(t *testing.T) {
	// Note: This test uses direct OS calls, not kernel syscalls
	t.Skip("Readlink test requires filesystem access - implement with proper mocking")
}

// TestOperationsOpsHardlink tests the Hardlink operation
func TestOperationsOpsHardlink(t *testing.T) {
	// Note: This test uses direct OS calls, not kernel syscalls
	t.Skip("Hardlink test requires filesystem access - implement with proper mocking")
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
