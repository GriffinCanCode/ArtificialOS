package operations

import (
	"context"
	gomath "math"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// TrigOps handles trigonometric operations
type TrigOps struct {
	*common.MathOps
}

// GetTools returns trig tool definitions
func (t *TrigOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.sin",
			Name:        "Sine",
			Description: "Calculate sine (in radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Angle in radians", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.cos",
			Name:        "Cosine",
			Description: "Calculate cosine (in radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Angle in radians", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.tan",
			Name:        "Tangent",
			Description: "Calculate tangent (in radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Angle in radians", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.asin",
			Name:        "Arcsine",
			Description: "Calculate inverse sine (returns radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Value between -1 and 1", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.acos",
			Name:        "Arccosine",
			Description: "Calculate inverse cosine (returns radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Value between -1 and 1", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.atan",
			Name:        "Arctangent",
			Description: "Calculate inverse tangent (returns radians)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.radians",
			Name:        "Degrees to Radians",
			Description: "Convert degrees to radians",
			Parameters: []types.Parameter{
				{Name: "degrees", Type: "number", Description: "Angle in degrees", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.degrees",
			Name:        "Radians to Degrees",
			Description: "Convert radians to degrees",
			Parameters: []types.Parameter{
				{Name: "radians", Type: "number", Description: "Angle in radians", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.atan2",
			Name:        "Arctangent2",
			Description: "Calculate atan2(y, x) - two-argument arctangent",
			Parameters: []types.Parameter{
				{Name: "y", Type: "number", Description: "Y coordinate", Required: true},
				{Name: "x", Type: "number", Description: "X coordinate", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.sinh",
			Name:        "Hyperbolic Sine",
			Description: "Calculate hyperbolic sine",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.cosh",
			Name:        "Hyperbolic Cosine",
			Description: "Calculate hyperbolic cosine",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.tanh",
			Name:        "Hyperbolic Tangent",
			Description: "Calculate hyperbolic tangent",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Number", Required: true},
			},
			Returns: "number",
		},
	}
}

// Sin calculates sine
func (t *TrigOps) Sin(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Sin(x)})
}

// Cos calculates cosine
func (t *TrigOps) Cos(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Cos(x)})
}

// Tan calculates tangent
func (t *TrigOps) Tan(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Tan(x)})
}

// Asin calculates arcsine
func (t *TrigOps) Asin(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x < -1 || x > 1 {
		return common.Failure("x must be between -1 and 1")
	}
	return common.Success(map[string]interface{}{"result": gomath.Asin(x)})
}

// Acos calculates arccosine
func (t *TrigOps) Acos(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	if x < -1 || x > 1 {
		return common.Failure("x must be between -1 and 1")
	}
	return common.Success(map[string]interface{}{"result": gomath.Acos(x)})
}

// Atan calculates arctangent
func (t *TrigOps) Atan(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Atan(x)})
}

// DegreesToRadians converts degrees to radians
func (t *TrigOps) DegreesToRadians(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	degrees, ok := common.GetNumber(params, "degrees")
	if !ok {
		return common.Failure("degrees parameter required")
	}
	radians := degrees * gomath.Pi / 180
	return common.Success(map[string]interface{}{"result": radians})
}

// RadiansToDegrees converts radians to degrees
func (t *TrigOps) RadiansToDegrees(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	radians, ok := common.GetNumber(params, "radians")
	if !ok {
		return common.Failure("radians parameter required")
	}
	degrees := radians * 180 / gomath.Pi
	return common.Success(map[string]interface{}{"result": degrees})
}

// Atan2 calculates two-argument arctangent
func (t *TrigOps) Atan2(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	y, ok := common.GetNumber(params, "y")
	if !ok {
		return common.Failure("y parameter required")
	}
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Atan2(y, x)})
}

// Sinh calculates hyperbolic sine
func (t *TrigOps) Sinh(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Sinh(x)})
}

// Cosh calculates hyperbolic cosine
func (t *TrigOps) Cosh(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Cosh(x)})
}

// Tanh calculates hyperbolic tangent
func (t *TrigOps) Tanh(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}
	return common.Success(map[string]interface{}{"result": gomath.Tanh(x)})
}
