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

// TestFormatsOpsGetTools tests the format operations tool definitions
func TestFormatsOpsGetTools(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	tools := formats.GetTools()

	assert.Equal(t, 8, len(tools))

	toolIDs := make(map[string]bool)
	for _, tool := range tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	assert.True(t, toolIDs["filesystem.yaml.read"])
	assert.True(t, toolIDs["filesystem.yaml.write"])
	assert.True(t, toolIDs["filesystem.csv.read"])
	assert.True(t, toolIDs["filesystem.csv.write"])
	assert.True(t, toolIDs["filesystem.json.merge"])
	assert.True(t, toolIDs["filesystem.toml.read"])
	assert.True(t, toolIDs["filesystem.toml.write"])
	assert.True(t, toolIDs["filesystem.csv.to_json"])
}

// TestFormatsOpsYAMLRead tests the YAMLRead operation
func TestFormatsOpsYAMLRead(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	yamlContent := []byte("name: test\nversion: 1.0\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(yamlContent, nil)

	result, err := formats.YAMLRead(ctx, map[string]interface{}{
		"path": "config.yaml",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "config.yaml", result.Data["path"])

	data := result.Data["data"].(map[string]interface{})
	assert.Equal(t, "test", data["name"])
	assert.Equal(t, 1.0, data["version"])
}

// TestFormatsOpsYAMLWrite tests the YAMLWrite operation
func TestFormatsOpsYAMLWrite(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	testData := map[string]interface{}{
		"name":    "test",
		"version": "1.0",
	}

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "config.yaml"
	})).Return([]byte("{}"), nil)

	result, err := formats.YAMLWrite(ctx, map[string]interface{}{
		"path": "config.yaml",
		"data": testData,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, "config.yaml", result.Data["path"])
	assert.NotNil(t, result.Data["size"])
}

// TestFormatsOpsCSVRead tests the CSVRead operation
func TestFormatsOpsCSVRead(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	csvContent := []byte("name,age,city\nAlice,30,NYC\nBob,25,LA\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(csvContent, nil)

	result, err := formats.CSVRead(ctx, map[string]interface{}{
		"path":       "data.csv",
		"has_header": true,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "data.csv", result.Data["path"])
	assert.Equal(t, 2, result.Data["count"])

	rows := result.Data["rows"].([]map[string]interface{})
	assert.Equal(t, 2, len(rows))
	assert.Equal(t, "Alice", rows[0]["name"])
	assert.Equal(t, "30", rows[0]["age"])
	assert.Equal(t, "NYC", rows[0]["city"])
}

// TestFormatsOpsCSVReadWithoutHeader tests CSVRead without header
func TestFormatsOpsCSVReadWithoutHeader(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	csvContent := []byte("Alice,30,NYC\nBob,25,LA\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(csvContent, nil)

	result, err := formats.CSVRead(ctx, map[string]interface{}{
		"path":       "data.csv",
		"has_header": false,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)

	rows := result.Data["rows"].([]map[string]interface{})
	assert.Equal(t, 2, len(rows))
	assert.Equal(t, "Alice", rows[0]["col0"])
	assert.Equal(t, "30", rows[0]["col1"])
	assert.Equal(t, "NYC", rows[0]["col2"])
}

// TestFormatsOpsCSVWrite tests the CSVWrite operation
func TestFormatsOpsCSVWrite(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	testData := []interface{}{
		map[string]interface{}{"name": "Alice", "age": "30"},
		map[string]interface{}{"name": "Bob", "age": "25"},
	}

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "output.csv"
	})).Return([]byte("{}"), nil)

	result, err := formats.CSVWrite(ctx, map[string]interface{}{
		"path": "output.csv",
		"data": testData,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, 2, result.Data["rows"])
}

// TestFormatsOpsJSONMerge tests the JSONMerge operation
func TestFormatsOpsJSONMerge(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	file1Data := map[string]interface{}{"key1": "value1"}
	file2Data := map[string]interface{}{"key2": "value2"}

	file1JSON, _ := json.Marshal(file1Data)
	file2JSON, _ := json.Marshal(file2Data)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "file1.json"
	})).Return(file1JSON, nil)

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "file2.json"
	})).Return(file2JSON, nil)

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "merged.json"
	})).Return([]byte("{}"), nil)

	result, err := formats.JSONMerge(ctx, map[string]interface{}{
		"files":  []interface{}{"file1.json", "file2.json"},
		"output": "merged.json",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, "merged.json", result.Data["path"])
	assert.Equal(t, 2, result.Data["keys"])
}

// TestFormatsOpsTOMLRead tests the TOMLRead operation
func TestFormatsOpsTOMLRead(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	tomlContent := []byte("name = \"test\"\nversion = \"1.0\"\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(tomlContent, nil)

	result, err := formats.TOMLRead(ctx, map[string]interface{}{
		"path": "config.toml",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, "config.toml", result.Data["path"])

	data := result.Data["data"].(map[string]interface{})
	assert.Equal(t, "test", data["name"])
	assert.Equal(t, "1.0", data["version"])
}

// TestFormatsOpsTOMLWrite tests the TOMLWrite operation
func TestFormatsOpsTOMLWrite(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	testData := map[string]interface{}{
		"name":    "test",
		"version": "1.0",
	}

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "config.toml"
	})).Return([]byte("{}"), nil)

	result, err := formats.TOMLWrite(ctx, map[string]interface{}{
		"path": "config.toml",
		"data": testData,
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["written"])
	assert.Equal(t, "config.toml", result.Data["path"])
}

// TestFormatsOpsCSVToJSON tests the CSVToJSON operation
func TestFormatsOpsCSVToJSON(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	csvContent := []byte("name,age\nAlice,30\nBob,25\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(csvContent, nil)

	mockKernel.On("ExecuteSyscall", ctx, uint32(1), "write_file", mock.MatchedBy(func(params map[string]interface{}) bool {
		return params["path"] == "output.json"
	})).Return([]byte("{}"), nil)

	result, err := formats.CSVToJSON(ctx, map[string]interface{}{
		"input":  "data.csv",
		"output": "output.json",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, true, result.Data["converted"])
	assert.Equal(t, 2, result.Data["rows"])
}

// TestFormatsOpsErrorHandling tests error handling for format operations
func TestFormatsOpsErrorHandling(t *testing.T) {
	mockKernel := testutil.NewMockKernelClient(t)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	tests := []struct {
		name    string
		fn      func() (*types.Result, error)
		wantErr bool
	}{
		{
			name: "yaml.read without path",
			fn: func() (*types.Result, error) {
				return formats.YAMLRead(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "yaml.write without path",
			fn: func() (*types.Result, error) {
				return formats.YAMLWrite(ctx, map[string]interface{}{"data": map[string]interface{}{}}, nil)
			},
			wantErr: true,
		},
		{
			name: "yaml.write without data",
			fn: func() (*types.Result, error) {
				return formats.YAMLWrite(ctx, map[string]interface{}{"path": "test.yaml"}, nil)
			},
			wantErr: true,
		},
		{
			name: "csv.read without path",
			fn: func() (*types.Result, error) {
				return formats.CSVRead(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "csv.write without path",
			fn: func() (*types.Result, error) {
				return formats.CSVWrite(ctx, map[string]interface{}{"data": []interface{}{}}, nil)
			},
			wantErr: true,
		},
		{
			name: "csv.write without data",
			fn: func() (*types.Result, error) {
				return formats.CSVWrite(ctx, map[string]interface{}{"path": "test.csv"}, nil)
			},
			wantErr: true,
		},
		{
			name: "json.merge without files",
			fn: func() (*types.Result, error) {
				return formats.JSONMerge(ctx, map[string]interface{}{"output": "out.json"}, nil)
			},
			wantErr: true,
		},
		{
			name: "json.merge without output",
			fn: func() (*types.Result, error) {
				return formats.JSONMerge(ctx, map[string]interface{}{"files": []interface{}{"f1.json"}}, nil)
			},
			wantErr: true,
		},
		{
			name: "toml.read without path",
			fn: func() (*types.Result, error) {
				return formats.TOMLRead(ctx, map[string]interface{}{}, nil)
			},
			wantErr: true,
		},
		{
			name: "toml.write without path",
			fn: func() (*types.Result, error) {
				return formats.TOMLWrite(ctx, map[string]interface{}{"data": map[string]interface{}{}}, nil)
			},
			wantErr: true,
		},
		{
			name: "toml.write without data",
			fn: func() (*types.Result, error) {
				return formats.TOMLWrite(ctx, map[string]interface{}{"path": "test.toml"}, nil)
			},
			wantErr: true,
		},
		{
			name: "csv_to_json without input",
			fn: func() (*types.Result, error) {
				return formats.CSVToJSON(ctx, map[string]interface{}{"output": "out.json"}, nil)
			},
			wantErr: true,
		},
		{
			name: "csv_to_json without output",
			fn: func() (*types.Result, error) {
				return formats.CSVToJSON(ctx, map[string]interface{}{"input": "in.csv"}, nil)
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

// TestFormatsOpsYAMLParseError tests YAML parse error handling
func TestFormatsOpsYAMLParseError(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	invalidYAML := []byte("invalid: yaml: content: [\n")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(invalidYAML, nil)

	result, err := formats.YAMLRead(ctx, map[string]interface{}{
		"path": "bad.yaml",
	}, nil)

	assert.NoError(t, err)
	assert.False(t, result.Success)
	assert.NotNil(t, result.Error)
}

// TestFormatsOpsCSVEmptyFile tests CSV with empty file
func TestFormatsOpsCSVEmptyFile(t *testing.T) {
	mockKernel := new(testutil.MockKernelClient)
	ops := &filesystem.FilesystemOps{
		Kernel:      mockKernel,
		StoragePID:  1,
		StoragePath: "/storage",
	}
	formats := &filesystem.FormatsOps{FilesystemOps: ops}

	ctx := context.Background()

	emptyCSV := []byte("")

	mockKernel.On("ExecuteSyscall", mock.Anything, uint32(1), "read_file", mock.Anything).
		Return(emptyCSV, nil)

	result, err := formats.CSVRead(ctx, map[string]interface{}{
		"path": "empty.csv",
	}, nil)

	assert.NoError(t, err)
	assert.True(t, result.Success)
	assert.Equal(t, 0, result.Data["count"])
}
