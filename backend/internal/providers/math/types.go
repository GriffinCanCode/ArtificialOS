package math

import (
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// MathOps provides common math helpers
type MathOps struct{}

// Success creates a successful result
func Success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

// Failure creates a failed result
func Failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}

// GetNumber extracts float64 from params with validation
func GetNumber(params map[string]interface{}, key string) (float64, bool) {
	val, ok := params[key]
	if !ok {
		return 0, false
	}

	switch v := val.(type) {
	case float64:
		return v, true
	case int:
		return float64(v), true
	case int64:
		return float64(v), true
	case float32:
		return float64(v), true
	default:
		return 0, false
	}
}

// GetNumbers extracts array of numbers with type coercion
func GetNumbers(params map[string]interface{}, key string) ([]float64, bool) {
	arr, ok := params[key].([]interface{})
	if !ok {
		return nil, false
	}

	numbers := make([]float64, 0, len(arr))
	for _, v := range arr {
		switch num := v.(type) {
		case float64:
			numbers = append(numbers, num)
		case int:
			numbers = append(numbers, float64(num))
		case int64:
			numbers = append(numbers, float64(num))
		case float32:
			numbers = append(numbers, float64(num))
		default:
			return nil, false
		}
	}
	return numbers, true
}

// GetString extracts string from params
func GetString(params map[string]interface{}, key string) (string, bool) {
	val, ok := params[key].(string)
	return val, ok
}

// GetBool extracts bool from params
func GetBool(params map[string]interface{}, key string) (bool, bool) {
	val, ok := params[key].(bool)
	return val, ok
}

// ValidateNumber checks if a number is valid (not NaN or Inf)
func ValidateNumber(x float64, name string) error {
	if x != x { // NaN check
		return fmt.Errorf("%s is NaN", name)
	}
	if x > 1e308 || x < -1e308 { // Infinity check
		return fmt.Errorf("%s is infinite", name)
	}
	return nil
}

// ValidateNumbers validates an array of numbers
func ValidateNumbers(nums []float64, name string) error {
	for i, x := range nums {
		if err := ValidateNumber(x, fmt.Sprintf("%s[%d]", name, i)); err != nil {
			return err
		}
	}
	return nil
}
