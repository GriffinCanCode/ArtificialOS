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

// TestMetadataOpsGetTools tests the metadata operations tool definitions
func TestMetadataOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	metadata := &filesystem.MetadataOps{FilesystemOps: ops}

	tools := metadata.GetTools()

	assert.Equal(t, 10, len(tools))

	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.stat"])
	assert.True(t, toolIDs["filesystem.size"])
	assert.True(t, toolIDs["filesystem.size_human"])
	assert.True(t, toolIDs["filesystem.total_size"])
	assert.True(t, toolIDs["filesystem.modified_time"])
	assert.True(t, toolIDs["filesystem.created_time"])
	assert.True(t, toolIDs["filesystem.accessed_time"])
	assert.True(t, toolIDs["filesystem.mime_type"])
	assert.True(t, toolIDs["filesystem.is_text"])
	assert.True(t, toolIDs["filesystem.is_binary"])
}

// TestMetadataOpsStat tests the Stat operation
func TestMetadataOpsStat(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	metadata := &filesystem.MetadataOps{FilesystemOps: ops}

	ctx := context.Background()

	statInfo := map[string]interface{}{
		"path":   "/test/file.txt",
		"size":   float64(1024),
		"is_dir": false,
		"mode":   float64(0644),
	}
	statJSON, _ := json.Marshal(statInfo)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.Anything).
		Return(statJSON, nil)

	result, err := metadata.Stat(ctx, map[string]interface{}{
		"path": "/test/file.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "/test/file.txt", result.Data["path"])

	info := result.Data["info"].(map[string]interface{})
	assert.Equal(t, float64(1024), info["size"])
	assert.Equal(t, false, info["is_dir"])
}

// TestMetadataOpsSize tests the Size operation
func TestMetadataOpsSize(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	metadata := &filesystem.MetadataOps{FilesystemOps: ops}

	ctx := context.Background()

	statInfo := map[string]interface{}{
		"path": "/test/file.txt",
		"size": float64(2048),
	}
	statJSON, _ := json.Marshal(statInfo)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.Anything).
		Return(statJSON, nil)

	result, err := metadata.Size(ctx, map[string]interface{}{
		"path": "/test/file.txt",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "/test/file.txt", result.Data["path"])
	assert.Equal(t, int64(2048), result.Data["size"])
}

// TestMetadataOpsSizeHuman tests the SizeHuman operation
func TestMetadataOpsSizeHuman(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	metadata := &filesystem.MetadataOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name      string
		size      float64
		wantHuman string
	}{
		{"small file", 512, "512 B"},
		{"kilobytes", 2048, "2.00 KB"},
		{"megabytes", 1048576, "1.00 MB"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			statInfo := map[string]interface{}{
				"path": "/test/file.txt",
				"size": tt.size,
			}
			statJSON, _ := json.Marshal(statInfo)

			mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "file_stat", mock.Anything).
				Return(statJSON, nil).Once()

			result, err := metadata.SizeHuman(ctx, map[string]interface{}{
				"path": "/test/file.txt",
			}, nil)

			assert.NoError(t, err)
			assert.True(t, result.Success)
			assert.Equal(t, tt.wantHuman, result.Data["size"])
			assert.Equal(t, int64(tt.size), result.Data["bytes"])
		})
	}
}

// TestMetadataOpsTotalSize tests the TotalSize operation
func TestMetadataOpsTotalSize(t *testing.T) {
	// Note: This test requires real filesystem access via fastwalk
	t.Skip("TotalSize test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsModifiedTime tests the ModifiedTime operation
func TestMetadataOpsModifiedTime(t *testing.T) {
	// Note: This test requires real filesystem access
	t.Skip("ModifiedTime test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsCreatedTime tests the CreatedTime operation
func TestMetadataOpsCreatedTime(t *testing.T) {
	// Note: This test requires real filesystem access and platform-specific syscalls
	t.Skip("CreatedTime test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsAccessedTime tests the AccessedTime operation
func TestMetadataOpsAccessedTime(t *testing.T) {
	// Note: This test requires real filesystem access and platform-specific syscalls
	t.Skip("AccessedTime test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsMIMEType tests the MIMEType operation
func TestMetadataOpsMIMEType(t *testing.T) {
	// Note: This test requires real filesystem access for MIME detection
	t.Skip("MIMEType test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsIsText tests the IsText operation
func TestMetadataOpsIsText(t *testing.T) {
	// Note: This test requires real filesystem access for MIME detection
	t.Skip("IsText test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsIsBinary tests the IsBinary operation
func TestMetadataOpsIsBinary(t *testing.T) {
	// Note: This test requires real filesystem access for MIME detection
	t.Skip("IsBinary test requires filesystem access - implement with proper mocking or integration test")
}

// TestMetadataOpsErrorHandling tests error handling for metadata operations
func TestMetadataOpsErrorHandling(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "stat without path",
			fn: func() (*types.Result, error) {
				mockKernel := testutil.NewMockKernelClient(t)
				ops := &filesystem.FilesystemOps{
					Kernel:      mockKernel,
					StoragePID:  1,
					StoragePath: "/storage",
				}
				metadata := &filesystem.MetadataOps{FilesystemOps: ops}
				return metadata.Stat(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "size without path",
			fn: func() (*types.Result, error) {
				mockKernel := testutil.NewMockKernelClient(t)
				ops := &filesystem.FilesystemOps{
					Kernel:      mockKernel,
					StoragePID:  1,
					StoragePath: "/storage",
				}
				metadata := &filesystem.MetadataOps{FilesystemOps: ops}
				return metadata.Size(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "size_human without path",
			fn: func() (*types.Result, error) {
				mockKernel := testutil.NewMockKernelClient(t)
				ops := &filesystem.FilesystemOps{
					Kernel:      mockKernel,
					StoragePID:  1,
					StoragePath: "/storage",
				}
				metadata := &filesystem.MetadataOps{FilesystemOps: ops}
				return metadata.SizeHuman(ctx, map[string]interface{}{}, nil)
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

// TestFormatBytes tests the formatBytes helper function
func TestFormatBytes(t *testing.T) {
	// Note: formatBytes is unexported, so we test it indirectly through SizeHuman
	// This test is covered by TestMetadataOpsSizeHuman
	t.Skip("formatBytes is tested indirectly through SizeHuman")
}

// TestMetadataOpsWithAppContext tests operations with app context
func TestMetadataOpsWithAppContext(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	metadata := &filesystem.MetadataOps{FilesystemOps: ops}

	ctx := context.Background()
	sandboxPID := uint32(42)
	appCtx := &types.Context{
		SandboxPID: &sandboxPID,
	}

	statInfo := map[string]interface{}{
		"path": "/test/file.txt",
		"size": float64(512),
	}
	statJSON, _ := json.Marshal(statInfo)

	// Should use app context PID instead of storage PID
	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(42), "file_stat", mock.Anything).
		Return(statJSON, nil)

	result, err := metadata.Size(ctx, map[string]interface{}{
		"path": "/test/file.txt",
	}, appCtx)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, int64(512), result.Data["size"])
}

// TestMetadataOpsTimeFormats tests time format options
func TestMetadataOpsTimeFormats(t *testing.T) {
	// Test that time operations respect the format parameter
	// Note: Requires filesystem access, covered by integration tests
	t.Skip("Time format tests require filesystem access - implement with integration tests")
}
