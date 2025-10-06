package unit

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
)

// TestSearchOpsGetTools tests the search operations tool definitions
func TestSearchOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	search := &filesystem.SearchOps{FilesystemOps: ops}

	tools := search.GetTools()

	assert.Equal(t, 8, len(tools))

	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.find"])
	assert.True(t, toolIDs["filesystem.glob"])
	assert.True(t, toolIDs["filesystem.filter_by_extension"])
	assert.True(t, toolIDs["filesystem.filter_by_size"])
	assert.True(t, toolIDs["filesystem.search_content"])
	assert.True(t, toolIDs["filesystem.regex_search"])
	assert.True(t, toolIDs["filesystem.filter_by_date"])
	assert.True(t, toolIDs["filesystem.recent_files"])
}

// TestSearchOpsFind tests the Find operation
func TestSearchOpsFind(t *testing.T) {
	// Note: This test requires real filesystem access via fastwalk
	t.Skip("Find test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsGlob tests the Glob operation
func TestSearchOpsGlob(t *testing.T) {
	// Note: This test requires real filesystem access via doublestar
	t.Skip("Glob test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsFilterByExtension tests the FilterByExtension operation
func TestSearchOpsFilterByExtension(t *testing.T) {
	// Note: This test requires real filesystem access via fastwalk
	t.Skip("FilterByExtension test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsFilterBySize tests the FilterBySize operation
func TestSearchOpsFilterBySize(t *testing.T) {
	// Note: This test requires real filesystem access via fastwalk
	t.Skip("FilterBySize test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsSearchContent tests the SearchContent operation
func TestSearchOpsSearchContent(t *testing.T) {
	// Note: This test requires real filesystem access for content scanning
	t.Skip("SearchContent test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsRegexSearch tests the RegexSearch operation
func TestSearchOpsRegexSearch(t *testing.T) {
	// Note: This test requires real filesystem access
	t.Skip("RegexSearch test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsFilterByDate tests the FilterByDate operation
func TestSearchOpsFilterByDate(t *testing.T) {
	// Note: This test requires real filesystem access
	t.Skip("FilterByDate test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsRecentFiles tests the RecentFiles operation
func TestSearchOpsRecentFiles(t *testing.T) {
	// Note: This test requires real filesystem access
	t.Skip("RecentFiles test requires filesystem access - implement with integration test using temp directory")
}

// TestSearchOpsErrorHandling tests error handling for search operations
func TestSearchOpsErrorHandling(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	search := &filesystem.SearchOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "find without path",
			fn: func() (*types.Result, error) {
				return search.Find(ctx, map[string]interface{}{"pattern": "*.go"}, nil)
			},
			wantErr: true,
		},
		{
			name: "find without pattern",
			fn: func() (*types.Result, error) {
				return search.Find(ctx, map[string]interface{}{"path": "/test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "glob without path",
			fn: func() (*types.Result, error) {
				return search.Glob(ctx, map[string]interface{}{"pattern": "**/*.go"}, nil)
			},
			wantErr: true,
		},
		{
			name: "glob without pattern",
			fn: func() (*types.Result, error) {
				return search.Glob(ctx, map[string]interface{}{"path": "/test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_extension without path",
			fn: func() (*types.Result, error) {
				return search.FilterByExtension(ctx, map[string]interface{}{"extensions": []interface{}{".go"}}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_extension without extensions",
			fn: func() (*types.Result, error) {
				return search.FilterByExtension(ctx, map[string]interface{}{"path": "/test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_size without path",
			fn: func() (*types.Result, error) {
				return search.FilterBySize(ctx, map[string]interface{}{"min_size": float64(100)}, nil)
			},
			wantErr: true,
		},
		{
			name: "search_content without path",
			fn: func() (*types.Result, error) {
				return search.SearchContent(ctx, map[string]interface{}{"query": "test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "search_content without query",
			fn: func() (*types.Result, error) {
				return search.SearchContent(ctx, map[string]interface{}{"path": "/test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "regex_search without path",
			fn: func() (*types.Result, error) {
				return search.RegexSearch(ctx, map[string]interface{}{"regex": ".*\\.go"}, nil)
			},
			wantErr: true,
		},
		{
			name: "regex_search without regex",
			fn: func() (*types.Result, error) {
				return search.RegexSearch(ctx, map[string]interface{}{"path": "/test"}, nil)
			},
			wantErr: true,
		},
		{
			name: "regex_search with invalid regex",
			fn: func() (*types.Result, error) {
				return search.RegexSearch(ctx, map[string]interface{}{
					"path":  "/test",
					"regex": "[invalid(",
				}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_date without path",
			fn: func() (*types.Result, error) {
				return search.FilterByDate(ctx, map[string]interface{}{"after": "2024-01-01T00:00:00Z"}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_date with invalid after date",
			fn: func() (*types.Result, error) {
				return search.FilterByDate(ctx, map[string]interface{}{
					"path":  "/test",
					"after": "invalid-date",
				}, nil)
			},
			wantErr: true,
		},
		{
			name: "filter_by_date with invalid before date",
			fn: func() (*types.Result, error) {
				return search.FilterByDate(ctx, map[string]interface{}{
					"path":   "/test",
					"before": "invalid-date",
				}, nil)
			},
			wantErr: true,
		},
		{
			name: "recent_files without path",
			fn: func() (*types.Result, error) {
				return search.RecentFiles(ctx, map[string]interface{}{"hours": float64(24)}, nil)
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

// TestSearchOpsWithAppContext tests search operations with app context
func TestSearchOpsWithAppContext(t *testing.T) {
	// Note: Requires filesystem access to test properly
	t.Skip("App context tests require filesystem access - implement with integration tests")
}

// TestSearchOpsExtensionNormalization tests that extensions are normalized correctly
func TestSearchOpsExtensionNormalization(t *testing.T) {
	// Test that extensions like "go" are normalized to ".go"
	// Note: This is an implementation detail tested through FilterByExtension
	t.Skip("Extension normalization tested indirectly through FilterByExtension integration tests")
}

// TestSearchOpsContentSearchPerformance tests content search with large files
func TestSearchOpsContentSearchPerformance(t *testing.T) {
	// Note: Performance/benchmark test
	t.Skip("Performance test - implement as benchmark test")
}

// TestSearchOpsRecentFilesLimits tests that RecentFiles respects limits
func TestSearchOpsRecentFilesLimits(t *testing.T) {
	// Note: Requires filesystem access
	t.Skip("Limit tests require filesystem access - implement with integration tests")
}

// TestSearchOpsRecentFilesDefaults tests default values for RecentFiles
func TestSearchOpsRecentFilesDefaults(t *testing.T) {
	// Note: Requires filesystem access
	t.Skip("Default value tests require filesystem access - implement with integration tests")
}
