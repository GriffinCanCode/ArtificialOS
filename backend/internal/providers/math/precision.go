package math

import (
	"context"
	"math/big"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// PrecisionOps handles high-precision arithmetic using big.Float
type PrecisionOps struct {
	*MathOps
}

// GetTools returns precision arithmetic tool definitions
func (p *PrecisionOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.precise.add",
			Name:        "Precise Addition",
			Description: "Add numbers with arbitrary precision (for financial calculations)",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Numbers to add", Required: true},
				{Name: "precision", Type: "number", Description: "Decimal precision (default: 10)", Required: false},
			},
			Returns: "string",
		},
		{
			ID:          "math.precise.subtract",
			Name:        "Precise Subtraction",
			Description: "Subtract with arbitrary precision",
			Parameters: []types.Parameter{
				{Name: "a", Type: "string", Description: "First number (as string)", Required: true},
				{Name: "b", Type: "string", Description: "Second number (as string)", Required: true},
				{Name: "precision", Type: "number", Description: "Decimal precision (default: 10)", Required: false},
			},
			Returns: "string",
		},
		{
			ID:          "math.precise.multiply",
			Name:        "Precise Multiplication",
			Description: "Multiply with arbitrary precision",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Numbers to multiply (as strings)", Required: true},
				{Name: "precision", Type: "number", Description: "Decimal precision (default: 10)", Required: false},
			},
			Returns: "string",
		},
		{
			ID:          "math.precise.divide",
			Name:        "Precise Division",
			Description: "Divide with arbitrary precision",
			Parameters: []types.Parameter{
				{Name: "a", Type: "string", Description: "Dividend (as string)", Required: true},
				{Name: "b", Type: "string", Description: "Divisor (as string)", Required: true},
				{Name: "precision", Type: "number", Description: "Decimal precision (default: 10)", Required: false},
			},
			Returns: "string",
		},
	}
}

// getPrecision extracts precision parameter or returns default
func getPrecision(params map[string]interface{}) uint {
	if prec, ok := GetNumber(params, "precision"); ok && prec > 0 {
		return uint(prec * 3.32) // Convert decimal places to binary precision
	}
	return 53 // Default: float64 precision
}

// PreciseAdd adds numbers with high precision
func (p *PrecisionOps) PreciseAdd(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbersParam, ok := params["numbers"]
	if !ok {
		return Failure("numbers parameter required")
	}

	var numbers []string
	switch v := numbersParam.(type) {
	case []string:
		numbers = v
	case []interface{}:
		for _, item := range v {
			if str, ok := item.(string); ok {
				numbers = append(numbers, str)
			} else {
				return Failure("all numbers must be strings for precision arithmetic")
			}
		}
	default:
		return Failure("numbers must be array of strings")
	}

	if len(numbers) == 0 {
		return Failure("numbers array required")
	}

	prec := getPrecision(params)
	result := big.NewFloat(0).SetPrec(prec)

	for _, numStr := range numbers {
		num := new(big.Float).SetPrec(prec)
		if _, ok := num.SetString(numStr); !ok {
			return Failure("invalid number format: " + numStr)
		}
		result.Add(result, num)
	}

	decimals := int(float64(prec) / 3.32)
	return Success(map[string]interface{}{
		"result":    result.Text('f', decimals),
		"precision": prec,
	})
}

// PreciseSubtract subtracts with high precision
func (p *PrecisionOps) PreciseSubtract(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	aStr, ok := GetString(params, "a")
	if !ok {
		return Failure("a parameter required (as string)")
	}

	bStr, ok := GetString(params, "b")
	if !ok {
		return Failure("b parameter required (as string)")
	}

	prec := getPrecision(params)

	a := new(big.Float).SetPrec(prec)
	if _, ok := a.SetString(aStr); !ok {
		return Failure("invalid number format for a")
	}

	b := new(big.Float).SetPrec(prec)
	if _, ok := b.SetString(bStr); !ok {
		return Failure("invalid number format for b")
	}

	result := new(big.Float).SetPrec(prec).Sub(a, b)

	decimals := int(float64(prec) / 3.32)
	return Success(map[string]interface{}{
		"result":    result.Text('f', decimals),
		"precision": prec,
	})
}

// PreciseMultiply multiplies with high precision
func (p *PrecisionOps) PreciseMultiply(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbersParam, ok := params["numbers"]
	if !ok {
		return Failure("numbers parameter required")
	}

	var numbers []string
	switch v := numbersParam.(type) {
	case []string:
		numbers = v
	case []interface{}:
		for _, item := range v {
			if str, ok := item.(string); ok {
				numbers = append(numbers, str)
			} else {
				return Failure("all numbers must be strings for precision arithmetic")
			}
		}
	default:
		return Failure("numbers must be array of strings")
	}

	if len(numbers) == 0 {
		return Failure("numbers array required")
	}

	prec := getPrecision(params)
	result := big.NewFloat(1).SetPrec(prec)

	for _, numStr := range numbers {
		num := new(big.Float).SetPrec(prec)
		if _, ok := num.SetString(numStr); !ok {
			return Failure("invalid number format: " + numStr)
		}
		result.Mul(result, num)
	}

	decimals := int(float64(prec) / 3.32)
	return Success(map[string]interface{}{
		"result":    result.Text('f', decimals),
		"precision": prec,
	})
}

// PreciseDivide divides with high precision
func (p *PrecisionOps) PreciseDivide(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	aStr, ok := GetString(params, "a")
	if !ok {
		return Failure("a parameter required (as string)")
	}

	bStr, ok := GetString(params, "b")
	if !ok {
		return Failure("b parameter required (as string)")
	}

	prec := getPrecision(params)

	a := new(big.Float).SetPrec(prec)
	if _, ok := a.SetString(aStr); !ok {
		return Failure("invalid number format for a")
	}

	b := new(big.Float).SetPrec(prec)
	if _, ok := b.SetString(bStr); !ok {
		return Failure("invalid number format for b")
	}

	if b.Sign() == 0 {
		return Failure("division by zero")
	}

	result := new(big.Float).SetPrec(prec).Quo(a, b)

	decimals := int(float64(prec) / 3.32)
	return Success(map[string]interface{}{
		"result":    result.Text('f', decimals),
		"precision": prec,
	})
}
