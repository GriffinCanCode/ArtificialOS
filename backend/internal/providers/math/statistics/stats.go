package statistics

import (
	"context"
	gomath "math"
	"sort"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/math/common"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"gonum.org/v1/gonum/stat"
)

// StatsOps handles statistical operations using gonum
type StatsOps struct {
	*common.MathOps
}

// GetTools returns stats tool definitions
func (s *StatsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.mean",
			Name:        "Mean",
			Description: "Calculate arithmetic mean",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.median",
			Name:        "Median",
			Description: "Calculate median value",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.min",
			Name:        "Minimum",
			Description: "Find minimum value",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.max",
			Name:        "Maximum",
			Description: "Find maximum value",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.sum",
			Name:        "Sum",
			Description: "Calculate sum of all numbers",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.stdev",
			Name:        "Standard Deviation",
			Description: "Calculate sample standard deviation",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.variance",
			Name:        "Variance",
			Description: "Calculate sample variance",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.range",
			Name:        "Range",
			Description: "Calculate range (max - min)",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.mode",
			Name:        "Mode",
			Description: "Find most frequent value",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.percentile",
			Name:        "Percentile",
			Description: "Calculate nth percentile",
			Parameters: []types.Parameter{
				{Name: "numbers", Type: "array", Description: "Array of numbers", Required: true},
				{Name: "p", Type: "number", Description: "Percentile (0-100)", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.correlation",
			Name:        "Correlation",
			Description: "Calculate Pearson correlation coefficient",
			Parameters: []types.Parameter{
				{Name: "x", Type: "array", Description: "First dataset", Required: true},
				{Name: "y", Type: "array", Description: "Second dataset", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.covariance",
			Name:        "Covariance",
			Description: "Calculate sample covariance",
			Parameters: []types.Parameter{
				{Name: "x", Type: "array", Description: "First dataset", Required: true},
				{Name: "y", Type: "array", Description: "Second dataset", Required: true},
			},
			Returns: "number",
		},
	}
}

// Mean calculates arithmetic mean using gonum
func (s *StatsOps) Mean(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	mean := stat.Mean(numbers, nil)
	return common.Success(map[string]interface{}{"result": mean})
}

// Median calculates median using gonum quantile
func (s *StatsOps) Median(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	sorted := make([]float64, len(numbers))
	copy(sorted, numbers)
	sort.Float64s(sorted)

	median := stat.Quantile(0.5, stat.Empirical, sorted, nil)
	return common.Success(map[string]interface{}{"result": median})
}

// Min finds minimum value
func (s *StatsOps) Min(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	min := numbers[0]
	for _, n := range numbers[1:] {
		min = gomath.Min(min, n)
	}

	return common.Success(map[string]interface{}{"result": min})
}

// Max finds maximum value
func (s *StatsOps) Max(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	max := numbers[0]
	for _, n := range numbers[1:] {
		max = gomath.Max(max, n)
	}

	return common.Success(map[string]interface{}{"result": max})
}

// Sum calculates sum
func (s *StatsOps) Sum(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	sum := 0.0
	for _, n := range numbers {
		sum += n
	}

	return common.Success(map[string]interface{}{"result": sum})
}

// Stdev calculates sample standard deviation using gonum
func (s *StatsOps) Stdev(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) < 2 {
		return common.Failure("numbers array with at least 2 elements required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	mean := stat.Mean(numbers, nil)
	variance := stat.Variance(numbers, nil)
	stdev := gomath.Sqrt(variance)

	return common.Success(map[string]interface{}{
		"result":   stdev,
		"variance": variance,
		"mean":     mean,
	})
}

// Variance calculates sample variance using gonum
func (s *StatsOps) Variance(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) < 2 {
		return common.Failure("numbers array with at least 2 elements required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	variance := stat.Variance(numbers, nil)
	return common.Success(map[string]interface{}{"result": variance})
}

// Range calculates range
func (s *StatsOps) Range(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	min := numbers[0]
	max := numbers[0]
	for _, n := range numbers[1:] {
		min = gomath.Min(min, n)
		max = gomath.Max(max, n)
	}

	return common.Success(map[string]interface{}{"result": max - min})
}

// Mode finds most frequent value
func (s *StatsOps) Mode(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	freqMap := make(map[float64]int)
	for _, n := range numbers {
		freqMap[n]++
	}

	var mode float64
	maxFreq := 0
	for num, freq := range freqMap {
		if freq > maxFreq {
			maxFreq = freq
			mode = num
		}
	}

	return common.Success(map[string]interface{}{
		"result":    mode,
		"frequency": maxFreq,
	})
}

// Percentile calculates nth percentile using gonum quantile
func (s *StatsOps) Percentile(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	numbers, ok := common.GetNumbers(params, "numbers")
	if !ok || len(numbers) == 0 {
		return common.Failure("numbers array required")
	}

	p, ok := common.GetNumber(params, "p")
	if !ok || p < 0 || p > 100 {
		return common.Failure("p parameter required (0-100)")
	}

	if err := common.ValidateNumbers(numbers, "numbers"); err != nil {
		return common.Failure(err.Error())
	}

	sorted := make([]float64, len(numbers))
	copy(sorted, numbers)
	sort.Float64s(sorted)

	// Convert percentile to quantile (0-1 range)
	quantile := p / 100.0
	result := stat.Quantile(quantile, stat.Empirical, sorted, nil)

	return common.Success(map[string]interface{}{"result": result})
}

// Correlation calculates Pearson correlation coefficient using gonum
func (s *StatsOps) Correlation(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumbers(params, "x")
	if !ok || len(x) == 0 {
		return common.Failure("x array required")
	}

	y, ok := common.GetNumbers(params, "y")
	if !ok || len(y) == 0 {
		return common.Failure("y array required")
	}

	if len(x) != len(y) {
		return common.Failure("x and y arrays must have same length")
	}

	if len(x) < 2 {
		return common.Failure("arrays must have at least 2 elements")
	}

	if err := common.ValidateNumbers(x, "x"); err != nil {
		return common.Failure(err.Error())
	}
	if err := common.ValidateNumbers(y, "y"); err != nil {
		return common.Failure(err.Error())
	}

	correlation := stat.Correlation(x, y, nil)

	return common.Success(map[string]interface{}{"result": correlation})
}

// Covariance calculates sample covariance using gonum
func (s *StatsOps) Covariance(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	x, ok := common.GetNumbers(params, "x")
	if !ok || len(x) == 0 {
		return common.Failure("x array required")
	}

	y, ok := common.GetNumbers(params, "y")
	if !ok || len(y) == 0 {
		return common.Failure("y array required")
	}

	if len(x) != len(y) {
		return common.Failure("x and y arrays must have same length")
	}

	if len(x) < 2 {
		return common.Failure("arrays must have at least 2 elements")
	}

	if err := common.ValidateNumbers(x, "x"); err != nil {
		return common.Failure(err.Error())
	}
	if err := common.ValidateNumbers(y, "y"); err != nil {
		return common.Failure(err.Error())
	}

	covariance := stat.Covariance(x, y, nil)

	return common.Success(map[string]interface{}{"result": covariance})
}
