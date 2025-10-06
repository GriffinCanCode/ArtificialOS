package operations

import (
	"context"
	gomath "math"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// ArithmeticOps handles basic arithmetic operations
type ArithmeticOps struct {
	*common.MathOps
}

// GetTools returns arithmetic tool definitions
func (a *ArithmeticOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.add",
			Name:        "Add",
			Description: "Add two or more numbers",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Numbers to add", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.subtract",
			Name:        "Subtract",
			Description: "Subtract b from a",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "First number", Required: true},
				{Name: "b", Type: "number", Description: "Second number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.multiply",
			Name:        "Multiply",
			Description: "Multiply two or more numbers",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Numbers to multiply", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.divide",
			Name:        "Divide",
			Description: "Divide a by b with precision",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "Dividend", Required: true},
				{Name: "b", Type: "number", Description: "Divisor", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.power",
			Name:        "Power",
			Description: "Raise a to the power of b (a^b)",
			Parameters: []types.Parameter{
				{Name: "base", Type: "number", Description: "Base", Required: true},
				{Name: "exponent", Type: "number", Description: "Exponent", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.sqrt",
			Name:        "Square Root",
			Description: "Calculate square root of a number",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.abs",
			Name:        "Absolute Value",
			Description: "Get absolute value of a number",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.floor",
			Name:        "Floor",
			Description: "Round down to nearest integer",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.ceil",
			Name:        "Ceiling",
			Description: "Round up to nearest integer",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.round",
			Name:        "Round",
			Description: "Round to nearest integer",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.exp",
			Name:        "Exponential",
			Description: "Calculate e^x",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Exponent", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.log",
			Name:        "Natural Logarithm",
			Description: "Calculate ln(x) (base e)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.log10",
			Name:        "Base-10 Logarithm",
			Description: "Calculate log₁₀(x)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.log2",
			Name:        "Base-2 Logarithm",
			Description: "Calculate log₂(x)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.mod",
			Name:        "Modulo",
			Description: "Calculate remainder of a/b",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "Dividend", Required: true},
				{Name: "b", Type: "number", Description: "Divisor", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.factorial",
			Name:        "Factorial",
			Description: "Calculate factorial (n!)",
			Parameters: []types.Parameter{
				{Name: "n", Type: "number", Description: "Non-negative integer", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.gcd",
			Name:        "Greatest Common Divisor",
			Description: "Calculate GCD of two numbers",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "First number", Required: true},
				{Name: "b", Type: "number", Description: "Second number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.lcm",
			Name:        "Least Common Multiple",
			Description: "Calculate LCM of two numbers",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "First number", Required: true},
				{Name: "b", Type: "number", Description: "Second number", Required: true},
			},
			Returns: "number",
		},
	}
}

// Add adds numbers
func (a *ArithmeticOps) Add(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	sum := 0.0
	for _, n := range numbers {
		sum += n
	}

	return common.Success(map[string]interface{}{"result": sum})
}

// Subtract subtracts b from a
func (a *ArithmeticOps) Subtract(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	valA, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}
	valB, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}

	return common.Success(map[string]interface{}{"result": valA - valB})
}

// Multiply multiplies numbers
func (a *ArithmeticOps) Multiply(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	product := 1.0
	for _, n := range numbers {
		product *= n
	}

	return common.Success(map[string]interface{}{"result": product})
}

// Divide divides a by b
func (a *ArithmeticOps) Divide(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	valA, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}
	valB, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}
	if valB == 0 {
		return common.Failure("division by zero")
	}

	return common.Success(map[string]interface{}{"result": valA / valB})
}

// Power raises base to exponent
func (a *ArithmeticOps) Power(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	base, ok := common.GetNumber(params, "base")
	if !ok {
		return common.Failure("base parameter required")
	}
	exponent, ok := common.GetNumber(params, "exponent")
	if !ok {
		return common.Failure("exponent parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Pow(base, exponent)})
}

// Sqrt calculates square root
func (a *ArithmeticOps) Sqrt(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x < 0 {
		return common.Failure("cannot take square root of negative number")
	}

	return common.Success(map[string]interface{}{"result": gomath.Sqrt(x)})
}

// Abs calculates absolute value
func (a *ArithmeticOps) Abs(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Abs(x)})
}

// Floor rounds down
func (a *ArithmeticOps) Floor(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Floor(x)})
}

// Ceil rounds up
func (a *ArithmeticOps) Ceil(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Ceil(x)})
}

// Round rounds to nearest integer
func (a *ArithmeticOps) Round(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Round(x)})
}

// Exp calculates e^x
func (a *ArithmeticOps) Exp(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	return common.Success(map[string]interface{}{"result": gomath.Exp(x)})
}

// Log calculates natural logarithm
func (a *ArithmeticOps) Log(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x <= 0 {
		return common.Failure("logarithm undefined for non-positive numbers")
	}

	return common.Success(map[string]interface{}{"result": gomath.Log(x)})
}

// Log10 calculates base-10 logarithm
func (a *ArithmeticOps) Log10(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x <= 0 {
		return common.Failure("logarithm undefined for non-positive numbers")
	}

	return common.Success(map[string]interface{}{"result": gomath.Log10(x)})
}

// Log2 calculates base-2 logarithm
func (a *ArithmeticOps) Log2(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x <= 0 {
		return common.Failure("logarithm undefined for non-positive numbers")
	}

	return common.Success(map[string]interface{}{"result": gomath.Log2(x)})
}

// Mod calculates modulo
func (a *ArithmeticOps) Mod(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	valA, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}
	valB, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}
	if valB == 0 {
		return common.Failure("division by zero")
	}

	return common.Success(map[string]interface{}{"result": gomath.Mod(valA, valB)})
}

// Factorial calculates factorial
func (a *ArithmeticOps) Factorial(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	n, ok := common.GetNumber(params, "n")
	if !ok {
		return common.Failure("n parameter required")
	}
	if n < 0 || n != gomath.Floor(n) {
		return common.Failure("n must be non-negative integer")
	}

	result := 1.0
	for i := 2; i <= int(n); i++ {
		result *= float64(i)
	}

	return common.Success(map[string]interface{}{"result": result})
}

// GCD calculates greatest common divisor
func (a *ArithmeticOps) GCD(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	valA, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}
	valB, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}

	// Euclidean algorithm
	numA := int(gomath.Abs(valA))
	numB := int(gomath.Abs(valB))

	for numB != 0 {
		numA, numB = numB, numA%numB
	}

	return common.Success(map[string]interface{}{"result": float64(numA)})
}

// LCM calculates least common multiple
func (a *ArithmeticOps) LCM(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	valA, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}
	valB, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}

	// LCM = (a * b) / GCD(a, b)
	numA := int(gomath.Abs(valA))
	numB := int(gomath.Abs(valB))

	// Calculate GCD
	gcd := numA
	remainder := numB
	for remainder != 0 {
		gcd, remainder = remainder, gcd%remainder
	}

	lcm := (numA * numB) / gcd

	return common.Success(map[string]interface{}{"result": float64(lcm)})
}
