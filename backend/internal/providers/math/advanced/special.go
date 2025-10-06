package advanced

import (
	"context"
	gomath "math"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"gonum.org/v1/gonum/mathext"
)

// SpecialOps handles special mathematical functions using gonum
type SpecialOps struct {
	*common.MathOps
}

// GetTools returns special function tool definitions
func (sp *SpecialOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.gamma",
			Name:        "Gamma Function",
			Description: "Calculate gamma function Γ(x)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Input value", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.beta",
			Name:        "Beta Function",
			Description: "Calculate beta function B(a,b)",
			Parameters: []types.Parameter{
				{Name: "a", Type: "number", Description: "First parameter", Required: true},
				{Name: "b", Type: "number", Description: "Second parameter", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.erf",
			Name:        "Error Function",
			Description: "Calculate error function erf(x)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Input value", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.erfc",
			Name:        "Complementary Error Function",
			Description: "Calculate complementary error function erfc(x)",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Input value", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.lgamma",
			Name:        "Log Gamma",
			Description: "Calculate natural log of gamma function ln(Γ(x))",
			Parameters: []types.Parameter{
				{Name: "x", Type: "number", Description: "Input value", Required: true},
			},
			Returns: "number",
		},
	}
}

// Gamma calculates gamma function using gonum
func (sp *SpecialOps) Gamma(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	if err := common.ValidateNumber(x, "x"); err != nil {
		return common.Failure(err.Error())
	}

	result := gomath.Gamma(x)

	if err := common.ValidateNumber(result, "result"); err != nil {
		return common.Failure("gamma function overflow")
	}

	return common.Success(map[string]interface{}{"result": result})
}

// Beta calculates beta function using gonum
func (sp *SpecialOps) Beta(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	a, ok := common.GetNumber(params, "a")
	if !ok {
		return common.Failure("a parameter required")
	}

	b, ok := common.GetNumber(params, "b")
	if !ok {
		return common.Failure("b parameter required")
	}

	if err := common.ValidateNumber(a, "a"); err != nil {
		return common.Failure(err.Error())
	}
	if err := common.ValidateNumber(b, "b"); err != nil {
		return common.Failure(err.Error())
	}

	// Beta(a,b) = Gamma(a)*Gamma(b)/Gamma(a+b)
	result := mathext.Beta(a, b)

	if err := common.ValidateNumber(result, "result"); err != nil {
		return common.Failure("beta function overflow")
	}

	return common.Success(map[string]interface{}{"result": result})
}

// Erf calculates error function
func (sp *SpecialOps) Erf(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	if err := common.ValidateNumber(x, "x"); err != nil {
		return common.Failure(err.Error())
	}

	result := gomath.Erf(x)
	return common.Success(map[string]interface{}{"result": result})
}

// Erfc calculates complementary error function
func (sp *SpecialOps) Erfc(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	if err := common.ValidateNumber(x, "x"); err != nil {
		return common.Failure(err.Error())
	}

	result := gomath.Erfc(x)
	return common.Success(map[string]interface{}{"result": result})
}

// Lgamma calculates log gamma function
func (sp *SpecialOps) Lgamma(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumber(params, "x")
	if !ok {
		return common.Failure("x parameter required")
	}

	if err := common.ValidateNumber(x, "x"); err != nil {
		return common.Failure(err.Error())
	}

	result, _ := gomath.Lgamma(x)

	if err := common.ValidateNumber(result, "result"); err != nil {
		return common.Failure("lgamma function overflow")
	}

	return common.Success(map[string]interface{}{"result": result})
}
