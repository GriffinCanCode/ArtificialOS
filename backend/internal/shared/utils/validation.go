package utils

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strings"
	"unicode/utf8"
)

// JSON size limits (in bytes)
const (
	MaxJSONSize    = 1 * 1024 * 1024 // 1MB - maximum JSON payload size
	MaxUISpecSize  = 512 * 1024      // 512KB - UI spec size limit
	MaxContextSize = 64 * 1024       // 64KB - context map size limit
	MaxMessageSize = 16 * 1024       // 16KB - single message size limit
)

// String length limits
const (
	MaxUsernameLength    = 64
	MinUsernameLength    = 3
	MaxPasswordLength    = 128
	MinPasswordLength    = 8
	MaxEmailLength       = 255
	MaxIDLength          = 128
	MaxNameLength        = 256
	MaxDescriptionLength = 2048
	MaxCategoryLength    = 64
	MaxTagLength         = 32
	MaxTagCount          = 20
)

// Regular expressions for validation
var (
	// SafeIDPattern allows alphanumeric, hyphens, underscores
	SafeIDPattern = regexp.MustCompile(`^[a-zA-Z0-9_-]+$`)
	// ToolIDPattern allows alphanumeric, hyphens, underscores, and dots (for service.tool format)
	ToolIDPattern = regexp.MustCompile(`^[a-zA-Z0-9._-]+$`)
	// UsernamePattern allows alphanumeric and underscores
	UsernamePattern = regexp.MustCompile(`^[a-zA-Z0-9_]+$`)
	// EmailPattern is a basic email validation
	EmailPattern = regexp.MustCompile(`^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`)
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

// ValidateString validates a string field with length and content checks
func ValidateString(value, fieldName string, minLen, maxLen int, required bool) error {
	if required && value == "" {
		return fmt.Errorf("%s is required", fieldName)
	}

	if value == "" && !required {
		return nil // Optional field, empty is OK
	}

	length := utf8.RuneCountInString(value)
	if length < minLen {
		return fmt.Errorf("%s must be at least %d characters", fieldName, minLen)
	}
	if length > maxLen {
		return fmt.Errorf("%s must not exceed %d characters", fieldName, maxLen)
	}

	// Check for null bytes (security issue)
	if strings.Contains(value, "\x00") {
		return fmt.Errorf("%s contains invalid characters", fieldName)
	}

	return nil
}

// ValidateID validates an ID field
func ValidateID(id, fieldName string, required bool) error {
	if err := ValidateString(id, fieldName, 1, MaxIDLength, required); err != nil {
		return err
	}

	if id != "" && !SafeIDPattern.MatchString(id) {
		return fmt.Errorf("%s contains invalid characters (only alphanumeric, hyphens, and underscores allowed)", fieldName)
	}

	return nil
}

// ValidateToolID validates a tool ID field (allows dots for service.tool format)
func ValidateToolID(id, fieldName string, required bool) error {
	if err := ValidateString(id, fieldName, 1, MaxIDLength, required); err != nil {
		return err
	}

	if id != "" && !ToolIDPattern.MatchString(id) {
		return fmt.Errorf("%s contains invalid characters (only alphanumeric, dots, hyphens, and underscores allowed)", fieldName)
	}

	return nil
}

// ValidateUsername validates a username
func ValidateUsername(username string) error {
	if err := ValidateString(username, "username", MinUsernameLength, MaxUsernameLength, true); err != nil {
		return err
	}

	if !UsernamePattern.MatchString(username) {
		return fmt.Errorf("username contains invalid characters (only alphanumeric and underscores allowed)")
	}

	return nil
}

// ValidatePassword validates a password
func ValidatePassword(password string) error {
	return ValidateString(password, "password", MinPasswordLength, MaxPasswordLength, true)
}

// ValidateEmail validates an email address
func ValidateEmail(email string, required bool) error {
	if err := ValidateString(email, "email", 0, MaxEmailLength, required); err != nil {
		return err
	}

	if email != "" && !EmailPattern.MatchString(email) {
		return fmt.Errorf("invalid email format")
	}

	return nil
}

// ValidateName validates a name field
func ValidateName(name, fieldName string) error {
	return ValidateString(name, fieldName, 1, MaxNameLength, true)
}

// ValidateDescription validates a description field
func ValidateDescription(description, fieldName string, required bool) error {
	return ValidateString(description, fieldName, 0, MaxDescriptionLength, required)
}

// ValidateCategory validates a category field
func ValidateCategory(category string, required bool) error {
	if err := ValidateString(category, "category", 0, MaxCategoryLength, required); err != nil {
		return err
	}

	// Category should only contain lowercase letters, numbers, and hyphens
	if category != "" && !regexp.MustCompile(`^[a-z0-9-]+$`).MatchString(category) {
		return fmt.Errorf("category must contain only lowercase letters, numbers, and hyphens")
	}

	return nil
}

// ValidateTags validates an array of tags
func ValidateTags(tags []string) error {
	if len(tags) > MaxTagCount {
		return fmt.Errorf("too many tags (maximum %d)", MaxTagCount)
	}

	for i, tag := range tags {
		if err := ValidateString(tag, fmt.Sprintf("tag[%d]", i), 1, MaxTagLength, false); err != nil {
			return err
		}
	}

	return nil
}

// ValidateMessage validates a chat message
func ValidateMessage(message string) error {
	if err := ValidateString(message, "message", 1, MaxMessageSize, true); err != nil {
		return err
	}

	// Check for excessive whitespace (potential DoS)
	whitespaceCount := 0
	for _, r := range message {
		if r == ' ' || r == '\t' || r == '\n' || r == '\r' {
			whitespaceCount++
		}
	}

	if whitespaceCount > len(message)/2 {
		return fmt.Errorf("message contains excessive whitespace")
	}

	return nil
}
