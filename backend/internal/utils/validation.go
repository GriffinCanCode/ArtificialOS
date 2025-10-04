package utils

import (
	"encoding/json"
	"fmt"
)

// JSON size limits (in bytes)
const (
	MaxJSONSize    = 1 * 1024 * 1024 // 1MB - maximum JSON payload size
	MaxUISpecSize  = 512 * 1024      // 512KB - UI spec size limit
	MaxContextSize = 64 * 1024       // 64KB - context map size limit
	MaxMessageSize = 16 * 1024       // 16KB - single message size limit
)

// JSONSizeValidator validates JSON size limits
type JSONSizeValidator struct {
	maxSize int
}

// NewJSONSizeValidator creates a new validator with the specified max size
func NewJSONSizeValidator(maxSize int) *JSONSizeValidator {
	return &JSONSizeValidator{maxSize: maxSize}
}

// DefaultJSONValidator returns a validator with the default 1MB limit
func DefaultJSONValidator() *JSONSizeValidator {
	return NewJSONSizeValidator(MaxJSONSize)
}

// ValidateSize checks if the data size is within limits
func (v *JSONSizeValidator) ValidateSize(data []byte) error {
	size := len(data)
	if size > v.maxSize {
		return fmt.Errorf("JSON size %d bytes exceeds maximum %d bytes", size, v.maxSize)
	}
	return nil
}

// ValidateJSON validates both size and JSON structure
func (v *JSONSizeValidator) ValidateJSON(data []byte) error {
	// Check size first (faster than parsing)
	if err := v.ValidateSize(data); err != nil {
		return err
	}

	// Validate it's valid JSON
	var js interface{}
	if err := json.Unmarshal(data, &js); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}

	return nil
}

// ValidateJSONString validates a JSON string
func (v *JSONSizeValidator) ValidateJSONString(jsonStr string) error {
	return v.ValidateJSON([]byte(jsonStr))
}

// ValidateDepth checks if JSON nesting depth is within limits
func ValidateJSONDepth(data interface{}, maxDepth int) error {
	return checkDepth(data, 0, maxDepth)
}

func checkDepth(data interface{}, currentDepth int, maxDepth int) error {
	if currentDepth > maxDepth {
		return fmt.Errorf("JSON nesting depth %d exceeds maximum %d", currentDepth, maxDepth)
	}

	switch v := data.(type) {
	case map[string]interface{}:
		for _, value := range v {
			if err := checkDepth(value, currentDepth+1, maxDepth); err != nil {
				return err
			}
		}
	case []interface{}:
		for _, value := range v {
			if err := checkDepth(value, currentDepth+1, maxDepth); err != nil {
				return err
			}
		}
	}

	return nil
}

// ValidateUISpec validates a UI specification
func ValidateUISpec(uiSpecJSON string) error {
	validator := NewJSONSizeValidator(MaxUISpecSize)

	// Validate size and structure
	if err := validator.ValidateJSONString(uiSpecJSON); err != nil {
		return fmt.Errorf("UI spec validation failed: %w", err)
	}

	// Parse and check depth
	var uiSpec map[string]interface{}
	if err := json.Unmarshal([]byte(uiSpecJSON), &uiSpec); err != nil {
		return fmt.Errorf("failed to parse UI spec: %w", err)
	}

	// Check nesting depth (prevent DoS from deeply nested structures)
	if err := ValidateJSONDepth(uiSpec, 20); err != nil {
		return fmt.Errorf("UI spec validation failed: %w", err)
	}

	return nil
}

// ValidateContext validates a context map before processing
func ValidateContext(context map[string]string) error {
	// Serialize to JSON to check size
	data, err := json.Marshal(context)
	if err != nil {
		return fmt.Errorf("failed to marshal context: %w", err)
	}

	validator := NewJSONSizeValidator(MaxContextSize)
	return validator.ValidateSize(data)
}
