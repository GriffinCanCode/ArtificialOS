package utilities

import (
	"context"
	gomath "math"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// ConstantsOps provides mathematical constants
type ConstantsOps struct {
	*common.MathOps
}

// GetTools returns constant tool definitions
func (c *ConstantsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.pi",
			Name:        "Pi (π)",
			Description: "Get value of π",
			Parameters:  []types.Parameter{},
			Returns:     "number",
		},
		{
			ID:          "math.e",
			Name:        "Euler's Number (e)",
			Description: "Get value of e",
			Parameters:  []types.Parameter{},
			Returns:     "number",
		},
		{
			ID:          "math.tau",
			Name:        "Tau (τ)",
			Description: "Get value of τ (2π)",
			Parameters:  []types.Parameter{},
			Returns:     "number",
		},
		{
			ID:          "math.phi",
			Name:        "Golden Ratio (φ)",
			Description: "Get value of φ (golden ratio)",
			Parameters:  []types.Parameter{},
			Returns:     "number",
		},
	}
}

// Pi returns π
func (c *ConstantsOps) Pi(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	return common.Success(map[string]interface{}{"result": gomath.Pi})
}

// E returns e
func (c *ConstantsOps) E(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	return common.Success(map[string]interface{}{"result": gomath.E})
}

// Tau returns τ (2π)
func (c *ConstantsOps) Tau(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	return common.Success(map[string]interface{}{"result": 2 * gomath.Pi})
}

// Phi returns φ (golden ratio)
func (c *ConstantsOps) Phi(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	phi := (1 + gomath.Sqrt(5)) / 2
	return common.Success(map[string]interface{}{"result": phi})
}
