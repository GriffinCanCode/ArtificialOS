package math

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/advanced"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/operations"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/statistics"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/utilities"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements mathematical operations
type Provider struct {
	// Module instances
	arithmetic  *operations.ArithmeticOps
	trig        *operations.TrigOps
	stats       *statistics.StatsOps
	constants   *utilities.ConstantsOps
	conversions *utilities.ConversionsOps
	precision   *advanced.PrecisionOps
	special     *advanced.SpecialOps
}

// NewProvider creates a modular math provider
func NewProvider() *Provider {
	ops := &common.MathOps{}

	return &Provider{
		arithmetic:  &operations.ArithmeticOps{MathOps: ops},
		trig:        &operations.TrigOps{MathOps: ops},
		stats:       &statistics.StatsOps{MathOps: ops},
		constants:   &utilities.ConstantsOps{MathOps: ops},
		conversions: &utilities.ConversionsOps{MathOps: ops},
		precision:   &advanced.PrecisionOps{MathOps: ops},
		special:     &advanced.SpecialOps{MathOps: ops},
	}
}

// Definition returns service metadata with all module tools
func (m *Provider) Definition() types.Service {
	// Collect tools from all modules
	tools := []types.Tool{}
	tools = append(tools, m.arithmetic.GetTools()...)
	tools = append(tools, m.trig.GetTools()...)
	tools = append(tools, m.stats.GetTools()...)
	tools = append(tools, m.constants.GetTools()...)
	tools = append(tools, m.conversions.GetTools()...)
	tools = append(tools, m.precision.GetTools()...)
	tools = append(tools, m.special.GetTools()...)

	return types.Service{
		ID:          "math",
		Name:        "Math Service",
		Description: "Mathematical operations (arithmetic, trig, stats, precision, special functions)",
		Category:    types.CategoryMath,
		Capabilities: []string{
			"arithmetic",
			"trigonometry",
			"statistics",
			"conversions",
			"precision",
			"special",
		},
		Tools: tools,
	}
}

// Execute routes to appropriate module
func (m *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	// Arithmetic operations
	case "math.add":
		return m.arithmetic.Add(ctx, params, appCtx)
	case "math.subtract":
		return m.arithmetic.Subtract(ctx, params, appCtx)
	case "math.multiply":
		return m.arithmetic.Multiply(ctx, params, appCtx)
	case "math.divide":
		return m.arithmetic.Divide(ctx, params, appCtx)
	case "math.power":
		return m.arithmetic.Power(ctx, params, appCtx)
	case "math.sqrt":
		return m.arithmetic.Sqrt(ctx, params, appCtx)
	case "math.abs":
		return m.arithmetic.Abs(ctx, params, appCtx)
	case "math.floor":
		return m.arithmetic.Floor(ctx, params, appCtx)
	case "math.ceil":
		return m.arithmetic.Ceil(ctx, params, appCtx)
	case "math.round":
		return m.arithmetic.Round(ctx, params, appCtx)
	case "math.exp":
		return m.arithmetic.Exp(ctx, params, appCtx)
	case "math.log":
		return m.arithmetic.Log(ctx, params, appCtx)
	case "math.log10":
		return m.arithmetic.Log10(ctx, params, appCtx)
	case "math.log2":
		return m.arithmetic.Log2(ctx, params, appCtx)
	case "math.mod":
		return m.arithmetic.Mod(ctx, params, appCtx)
	case "math.factorial":
		return m.arithmetic.Factorial(ctx, params, appCtx)
	case "math.gcd":
		return m.arithmetic.GCD(ctx, params, appCtx)
	case "math.lcm":
		return m.arithmetic.LCM(ctx, params, appCtx)

	// Trig operations
	case "math.sin":
		return m.trig.Sin(ctx, params, appCtx)
	case "math.cos":
		return m.trig.Cos(ctx, params, appCtx)
	case "math.tan":
		return m.trig.Tan(ctx, params, appCtx)
	case "math.asin":
		return m.trig.Asin(ctx, params, appCtx)
	case "math.acos":
		return m.trig.Acos(ctx, params, appCtx)
	case "math.atan":
		return m.trig.Atan(ctx, params, appCtx)
	case "math.atan2":
		return m.trig.Atan2(ctx, params, appCtx)
	case "math.sinh":
		return m.trig.Sinh(ctx, params, appCtx)
	case "math.cosh":
		return m.trig.Cosh(ctx, params, appCtx)
	case "math.tanh":
		return m.trig.Tanh(ctx, params, appCtx)
	case "math.radians":
		return m.trig.DegreesToRadians(ctx, params, appCtx)
	case "math.degrees":
		return m.trig.RadiansToDegrees(ctx, params, appCtx)

	// Stats operations
	case "math.mean":
		return m.stats.Mean(ctx, params, appCtx)
	case "math.median":
		return m.stats.Median(ctx, params, appCtx)
	case "math.min":
		return m.stats.Min(ctx, params, appCtx)
	case "math.max":
		return m.stats.Max(ctx, params, appCtx)
	case "math.sum":
		return m.stats.Sum(ctx, params, appCtx)
	case "math.stdev":
		return m.stats.Stdev(ctx, params, appCtx)
	case "math.variance":
		return m.stats.Variance(ctx, params, appCtx)
	case "math.range":
		return m.stats.Range(ctx, params, appCtx)
	case "math.mode":
		return m.stats.Mode(ctx, params, appCtx)
	case "math.percentile":
		return m.stats.Percentile(ctx, params, appCtx)

	// Constants
	case "math.pi":
		return m.constants.Pi(ctx, params, appCtx)
	case "math.e":
		return m.constants.E(ctx, params, appCtx)
	case "math.tau":
		return m.constants.Tau(ctx, params, appCtx)
	case "math.phi":
		return m.constants.Phi(ctx, params, appCtx)

	// Conversions
	case "math.celsiusToFahrenheit":
		return m.conversions.CelsiusToFahrenheit(ctx, params, appCtx)
	case "math.fahrenheitToCelsius":
		return m.conversions.FahrenheitToCelsius(ctx, params, appCtx)
	case "math.celsiusToKelvin":
		return m.conversions.CelsiusToKelvin(ctx, params, appCtx)
	case "math.kelvinToCelsius":
		return m.conversions.KelvinToCelsius(ctx, params, appCtx)
	case "math.metersToFeet":
		return m.conversions.MetersToFeet(ctx, params, appCtx)
	case "math.feetToMeters":
		return m.conversions.FeetToMeters(ctx, params, appCtx)
	case "math.milesToKilometers":
		return m.conversions.MilesToKilometers(ctx, params, appCtx)
	case "math.kilometersToMiles":
		return m.conversions.KilometersToMiles(ctx, params, appCtx)

	// Precision operations
	case "math.precise.add":
		return m.precision.PreciseAdd(ctx, params, appCtx)
	case "math.precise.subtract":
		return m.precision.PreciseSubtract(ctx, params, appCtx)
	case "math.precise.multiply":
		return m.precision.PreciseMultiply(ctx, params, appCtx)
	case "math.precise.divide":
		return m.precision.PreciseDivide(ctx, params, appCtx)

	// Special functions
	case "math.gamma":
		return m.special.Gamma(ctx, params, appCtx)
	case "math.beta":
		return m.special.Beta(ctx, params, appCtx)
	case "math.erf":
		return m.special.Erf(ctx, params, appCtx)
	case "math.erfc":
		return m.special.Erfc(ctx, params, appCtx)
	case "math.lgamma":
		return m.special.Lgamma(ctx, params, appCtx)

	// Stats - add new functions
	case "math.correlation":
		return m.stats.Correlation(ctx, params, appCtx)
	case "math.covariance":
		return m.stats.Covariance(ctx, params, appCtx)

	default:
		return common.Failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}
