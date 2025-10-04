package http

import (
	"context"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// ParseOps handles response parsing
type ParseOps struct {
	*HTTPOps
}

// GetTools returns parsing tool definitions
func (p *ParseOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.parseJSON",
			Name:        "Parse JSON",
			Description: "Parse JSON string into object with validation",
			Parameters: []types.Parameter{
				{Name: "json", Type: "string", Description: "JSON string", Required: true},
				{Name: "strict", Type: "boolean", Description: "Strict parsing (default: false)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.parseXML",
			Name:        "Parse XML",
			Description: "Parse XML string into object",
			Parameters: []types.Parameter{
				{Name: "xml", Type: "string", Description: "XML string", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "http.stringifyJSON",
			Name:        "Stringify JSON",
			Description: "Convert object to JSON string",
			Parameters: []types.Parameter{
				{Name: "data", Type: "object", Description: "Data to stringify", Required: true},
				{Name: "pretty", Type: "boolean", Description: "Pretty print (default: false)", Required: false},
			},
			Returns: "string",
		},
	}
}

// ParseJSON parses JSON with comprehensive error handling
func (p *ParseOps) ParseJSON(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	jsonStr, err := GetString(params, "json", true)
	if err != nil {
		return Failure(err.Error())
	}

	if strings.TrimSpace(jsonStr) == "" {
		return Failure("json string is empty")
	}

	strict := GetBool(params, "strict", false)

	var parsed interface{}
	decoder := json.NewDecoder(strings.NewReader(jsonStr))

	if strict {
		// Disallow unknown fields in strict mode
		decoder.DisallowUnknownFields()
	}

	if err := decoder.Decode(&parsed); err != nil {
		return Failure(fmt.Sprintf("invalid JSON: %v", err))
	}

	// Check for trailing data
	if decoder.More() {
		return Failure("JSON contains trailing data")
	}

	result := map[string]interface{}{
		"data":   parsed,
		"parsed": true,
		"type":   getJSONType(parsed),
	}

	if strict {
		result["strict"] = true
	}

	return Success(result)
}

// ParseXML parses XML with error handling
func (p *ParseOps) ParseXML(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	xmlStr, err := GetString(params, "xml", true)
	if err != nil {
		return Failure(err.Error())
	}

	if strings.TrimSpace(xmlStr) == "" {
		return Failure("xml string is empty")
	}

	// Parse into generic map structure
	var parsed map[string]interface{}
	if err := xml.Unmarshal([]byte(xmlStr), &parsed); err != nil {
		// Try parsing as array
		var parsedArray []interface{}
		if err2 := xml.Unmarshal([]byte(xmlStr), &parsedArray); err2 != nil {
			return Failure(fmt.Sprintf("invalid XML: %v", err))
		}
		return Success(map[string]interface{}{
			"data":   parsedArray,
			"parsed": true,
			"type":   "array",
		})
	}

	return Success(map[string]interface{}{
		"data":   parsed,
		"parsed": true,
		"type":   "object",
	})
}

// StringifyJSON converts object to JSON string
func (p *ParseOps) StringifyJSON(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	data := params["data"]
	if data == nil {
		return Failure("data parameter required")
	}

	pretty := GetBool(params, "pretty", false)

	var jsonBytes []byte
	var err error

	if pretty {
		jsonBytes, err = json.MarshalIndent(data, "", "  ")
	} else {
		jsonBytes, err = json.Marshal(data)
	}

	if err != nil {
		return Failure(fmt.Sprintf("failed to stringify: %v", err))
	}

	return Success(map[string]interface{}{
		"json":   string(jsonBytes),
		"size":   len(jsonBytes),
		"pretty": pretty,
	})
}

// getJSONType determines the type of parsed JSON data
func getJSONType(data interface{}) string {
	switch data.(type) {
	case map[string]interface{}:
		return "object"
	case []interface{}:
		return "array"
	case string:
		return "string"
	case float64:
		return "number"
	case bool:
		return "boolean"
	case nil:
		return "null"
	default:
		return "unknown"
	}
}
