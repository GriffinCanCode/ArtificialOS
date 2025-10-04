package filesystem

import (
	"context"
	"encoding/csv"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/bytedance/sonic"
	"github.com/goccy/go-yaml"
	"github.com/pelletier/go-toml/v2"
)

// FormatsOps handles file format operations
type FormatsOps struct {
	*FilesystemOps
}

// GetTools returns format operation tool definitions
func (f *FormatsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.yaml.read",
			Name:        "Read YAML",
			Description: "Parse YAML file (3-5x faster)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.yaml.write",
			Name:        "Write YAML",
			Description: "Write YAML file (optimized)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "object", Description: "Data to write", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.csv.read",
			Name:        "Read CSV",
			Description: "Parse CSV file to array of objects",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "has_header", Type: "boolean", Description: "First row is header (default true)", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.csv.write",
			Name:        "Write CSV",
			Description: "Write array of objects to CSV",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "array", Description: "Array of objects", Required: true},
				{Name: "headers", Type: "array", Description: "Column headers (optional)", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.json.merge",
			Name:        "Merge JSON Files",
			Description: "Merge multiple JSON files (fast)",
			Parameters: []types.Parameter{
				{Name: "files", Type: "array", Description: "Array of file paths", Required: true},
				{Name: "output", Type: "string", Description: "Output file path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.toml.read",
			Name:        "Read TOML",
			Description: "Parse TOML file (2x faster)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.toml.write",
			Name:        "Write TOML",
			Description: "Write TOML file (optimized)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "File path", Required: true},
				{Name: "data", Type: "object", Description: "Data to write", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.csv.to_json",
			Name:        "CSV to JSON",
			Description: "Convert CSV to JSON (fast encoding)",
			Parameters: []types.Parameter{
				{Name: "input", Type: "string", Description: "CSV file path", Required: true},
				{Name: "output", Type: "string", Description: "JSON file path", Required: true},
				{Name: "has_header", Type: "boolean", Description: "CSV has header (default true)", Required: false},
			},
			Returns: "boolean",
		},
	}
}

// YAMLRead parses YAML file (3-5x faster with goccy/go-yaml)
func (f *FormatsOps) YAMLRead(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := f.GetPID(appCtx)
	data, err := f.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	var parsed interface{}
	if err := yaml.Unmarshal(data, &parsed); err != nil {
		return Failure(fmt.Sprintf("YAML parse error: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "data": parsed})
}

// YAMLWrite writes YAML file
func (f *FormatsOps) YAMLWrite(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	data, ok := params["data"]
	if !ok {
		return Failure("data parameter required")
	}

	yamlData, err := yaml.Marshal(data)
	if err != nil {
		return Failure(fmt.Sprintf("YAML encoding error: %v", err))
	}

	pid := f.GetPID(appCtx)
	_, err = f.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": yamlData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{"written": true, "path": path, "size": len(yamlData)})
}

// CSVRead parses CSV file
func (f *FormatsOps) CSVRead(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	hasHeader := true
	if h, ok := params["has_header"].(bool); ok {
		hasHeader = h
	}

	pid := f.GetPID(appCtx)
	data, err := f.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	reader := csv.NewReader(strings.NewReader(string(data)))
	records, err := reader.ReadAll()
	if err != nil {
		return Failure(fmt.Sprintf("CSV parse error: %v", err))
	}

	if len(records) == 0 {
		return Success(map[string]interface{}{"path": path, "rows": []interface{}{}, "count": 0})
	}

	var headers []string
	startRow := 0

	if hasHeader {
		headers = records[0]
		startRow = 1
	} else {
		// Generate headers: col0, col1, ...
		for i := 0; i < len(records[0]); i++ {
			headers = append(headers, fmt.Sprintf("col%d", i))
		}
	}

	rows := []map[string]interface{}{}
	for i := startRow; i < len(records); i++ {
		row := make(map[string]interface{})
		for j, value := range records[i] {
			if j < len(headers) {
				row[headers[j]] = value
			}
		}
		rows = append(rows, row)
	}

	return Success(map[string]interface{}{"path": path, "rows": rows, "count": len(rows), "headers": headers})
}

// CSVWrite writes CSV file
func (f *FormatsOps) CSVWrite(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	dataArr, ok := params["data"].([]interface{})
	if !ok || len(dataArr) == 0 {
		return Failure("data array required")
	}

	// Extract headers
	var headers []string
	if headersParam, ok := params["headers"].([]interface{}); ok {
		for _, h := range headersParam {
			if hStr, ok := h.(string); ok {
				headers = append(headers, hStr)
			}
		}
	} else {
		// Auto-detect from first row
		if firstRow, ok := dataArr[0].(map[string]interface{}); ok {
			for key := range firstRow {
				headers = append(headers, key)
			}
		}
	}

	if len(headers) == 0 {
		return Failure("no headers found")
	}

	var buf strings.Builder
	writer := csv.NewWriter(&buf)

	// Write header
	if err := writer.Write(headers); err != nil {
		return Failure(fmt.Sprintf("CSV write error: %v", err))
	}

	// Write rows
	for _, rowData := range dataArr {
		rowMap, ok := rowData.(map[string]interface{})
		if !ok {
			continue
		}

		row := make([]string, len(headers))
		for i, header := range headers {
			if val, ok := rowMap[header]; ok {
				row[i] = fmt.Sprintf("%v", val)
			}
		}

		if err := writer.Write(row); err != nil {
			return Failure(fmt.Sprintf("CSV write error: %v", err))
		}
	}

	writer.Flush()
	if err := writer.Error(); err != nil {
		return Failure(fmt.Sprintf("CSV flush error: %v", err))
	}

	csvData := []byte(buf.String())
	pid := f.GetPID(appCtx)
	_, err := f.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": csvData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{"written": true, "path": path, "rows": len(dataArr)})
}

// JSONMerge merges multiple JSON files (uses sonic for large files)
func (f *FormatsOps) JSONMerge(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	filesParam, ok := params["files"].([]interface{})
	if !ok || len(filesParam) == 0 {
		return Failure("files array required")
	}

	output, ok := params["output"].(string)
	if !ok || output == "" {
		return Failure("output parameter required")
	}

	pid := f.GetPID(appCtx)
	merged := make(map[string]interface{})

	// Read and merge all files
	for _, fileParam := range filesParam {
		filePath, ok := fileParam.(string)
		if !ok {
			continue
		}

		data, err := f.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{"path": filePath})
		if err != nil {
			continue
		}

		var parsed map[string]interface{}
		// Use sonic for files >10KB, encoding/json for smaller
		if len(data) > 10240 {
			if err := sonic.Unmarshal(data, &parsed); err != nil {
				continue
			}
		} else {
			if err := json.Unmarshal(data, &parsed); err != nil {
				continue
			}
		}

		// Deep merge
		for key, value := range parsed {
			merged[key] = value
		}
	}

	// Write merged result (use sonic for large output)
	var jsonData []byte
	var err error

	if len(merged) > 100 { // Use sonic for larger objects
		jsonData, err = sonic.MarshalIndent(merged, "", "  ")
	} else {
		jsonData, err = json.MarshalIndent(merged, "", "  ")
	}

	if err != nil {
		return Failure(fmt.Sprintf("JSON encoding error: %v", err))
	}

	_, err = f.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": output,
		"data": jsonData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{"written": true, "path": output, "keys": len(merged), "size": len(jsonData)})
}

// TOMLRead parses TOML file (2x faster with pelletier/go-toml/v2)
func (f *FormatsOps) TOMLRead(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	pid := f.GetPID(appCtx)
	data, err := f.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{"path": path})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	var parsed map[string]interface{}
	if err := toml.Unmarshal(data, &parsed); err != nil {
		return Failure(fmt.Sprintf("TOML parse error: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "data": parsed})
}

// TOMLWrite writes TOML file
func (f *FormatsOps) TOMLWrite(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	data, ok := params["data"]
	if !ok {
		return Failure("data parameter required")
	}

	tomlData, err := toml.Marshal(data)
	if err != nil {
		return Failure(fmt.Sprintf("TOML encoding error: %v", err))
	}

	pid := f.GetPID(appCtx)
	_, err = f.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": path,
		"data": tomlData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{"written": true, "path": path, "size": len(tomlData)})
}

// CSVToJSON converts CSV to JSON
func (f *FormatsOps) CSVToJSON(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	input, ok := params["input"].(string)
	if !ok || input == "" {
		return Failure("input parameter required")
	}

	output, ok := params["output"].(string)
	if !ok || output == "" {
		return Failure("output parameter required")
	}

	hasHeader := true
	if h, ok := params["has_header"].(bool); ok {
		hasHeader = h
	}

	// Read CSV
	pid := f.GetPID(appCtx)
	data, err := f.Kernel.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{"path": input})
	if err != nil {
		return Failure(fmt.Sprintf("read failed: %v", err))
	}

	reader := csv.NewReader(strings.NewReader(string(data)))
	records, err := reader.ReadAll()
	if err != nil {
		return Failure(fmt.Sprintf("CSV parse error: %v", err))
	}

	if len(records) == 0 {
		return Failure("empty CSV file")
	}

	var headers []string
	startRow := 0

	if hasHeader {
		headers = records[0]
		startRow = 1
	} else {
		for i := 0; i < len(records[0]); i++ {
			headers = append(headers, fmt.Sprintf("col%d", i))
		}
	}

	rows := []map[string]interface{}{}
	for i := startRow; i < len(records); i++ {
		row := make(map[string]interface{})
		for j, value := range records[i] {
			if j < len(headers) {
				row[headers[j]] = value
			}
		}
		rows = append(rows, row)
	}

	// Write JSON (use sonic for large datasets)
	var jsonData []byte
	if len(rows) > 100 {
		jsonData, err = sonic.MarshalIndent(rows, "", "  ")
	} else {
		jsonData, err = json.MarshalIndent(rows, "", "  ")
	}

	if err != nil {
		return Failure(fmt.Sprintf("JSON encoding error: %v", err))
	}

	_, err = f.Kernel.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
		"path": output,
		"data": jsonData,
	})
	if err != nil {
		return Failure(fmt.Sprintf("write failed: %v", err))
	}

	return Success(map[string]interface{}{"converted": true, "input": input, "output": output, "rows": len(rows)})
}
