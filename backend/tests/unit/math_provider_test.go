package unit

import (
	"context"
	"math"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMathProvider(t *testing.T) {
	mathProvider := providers.NewMath()
	ctx := context.Background()

	t.Run("Arithmetic Operations", func(t *testing.T) {
		t.Run("Add", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.add", map[string]interface{}{
				"numbers": []interface{}{1.0, 2.0, 3.0, 4.0, 5.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 15.0, result.Data["result"])
		})

		t.Run("Add with integers", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.add", map[string]interface{}{
				"numbers": []interface{}{1, 2, 3},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 6.0, result.Data["result"])
		})

		t.Run("Add with empty array", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.add", map[string]interface{}{
				"numbers": []interface{}{},
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Subtract", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.subtract", map[string]interface{}{
				"a": 10.0,
				"b": 3.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 7.0, result.Data["result"])
		})

		t.Run("Multiply", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.multiply", map[string]interface{}{
				"numbers": []interface{}{2.0, 3.0, 4.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 24.0, result.Data["result"])
		})

		t.Run("Divide", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.divide", map[string]interface{}{
				"a": 10.0,
				"b": 2.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 5.0, result.Data["result"])
		})

		t.Run("Divide by zero", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.divide", map[string]interface{}{
				"a": 10.0,
				"b": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Power", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.power", map[string]interface{}{
				"base":     2.0,
				"exponent": 3.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 8.0, result.Data["result"])
		})

		t.Run("Sqrt", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.sqrt", map[string]interface{}{
				"x": 16.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 4.0, result.Data["result"])
		})

		t.Run("Sqrt of negative", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.sqrt", map[string]interface{}{
				"x": -1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Abs", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.abs", map[string]interface{}{
				"x": -5.5,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 5.5, result.Data["result"])
		})

		t.Run("Floor", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.floor", map[string]interface{}{
				"x": 3.7,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 3.0, result.Data["result"])
		})

		t.Run("Ceil", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.ceil", map[string]interface{}{
				"x": 3.2,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 4.0, result.Data["result"])
		})

		t.Run("Round", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.round", map[string]interface{}{
				"x": 3.5,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 4.0, result.Data["result"])
		})

		t.Run("Exp", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.exp", map[string]interface{}{
				"x": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, math.E, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Log", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.log", map[string]interface{}{
				"x": math.E,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Log of negative", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.log", map[string]interface{}{
				"x": -1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Log10", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.log10", map[string]interface{}{
				"x": 100.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 2.0, result.Data["result"])
		})

		t.Run("Log2", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.log2", map[string]interface{}{
				"x": 8.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 3.0, result.Data["result"])
		})

		t.Run("Mod", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.mod", map[string]interface{}{
				"a": 10.0,
				"b": 3.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 1.0, result.Data["result"])
		})

		t.Run("Factorial", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.factorial", map[string]interface{}{
				"n": 5.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 120.0, result.Data["result"])
		})

		t.Run("Factorial of negative", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.factorial", map[string]interface{}{
				"n": -1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("GCD", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.gcd", map[string]interface{}{
				"a": 48.0,
				"b": 18.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 6.0, result.Data["result"])
		})

		t.Run("LCM", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.lcm", map[string]interface{}{
				"a": 12.0,
				"b": 18.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 36.0, result.Data["result"])
		})
	})

	t.Run("Trigonometric Operations", func(t *testing.T) {
		t.Run("Sin", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.sin", map[string]interface{}{
				"x": math.Pi / 2,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Cos", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.cos", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 1.0, result.Data["result"])
		})

		t.Run("Tan", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.tan", map[string]interface{}{
				"x": math.Pi / 4,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Asin", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.asin", map[string]interface{}{
				"x": 0.5,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, math.Pi/6, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Asin out of range", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.asin", map[string]interface{}{
				"x": 2.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Acos", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.acos", map[string]interface{}{
				"x": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 0.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Atan", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.atan", map[string]interface{}{
				"x": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, math.Pi/4, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Atan2", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.atan2", map[string]interface{}{
				"y": 1.0,
				"x": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, math.Pi/4, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Degrees to Radians", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.radians", map[string]interface{}{
				"degrees": 180.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, math.Pi, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Radians to Degrees", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.degrees", map[string]interface{}{
				"radians": math.Pi,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 180.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Sinh", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.sinh", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 0.0, result.Data["result"])
		})

		t.Run("Cosh", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.cosh", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 1.0, result.Data["result"])
		})

		t.Run("Tanh", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.tanh", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 0.0, result.Data["result"])
		})
	})

	t.Run("Statistical Operations", func(t *testing.T) {
		testData := []interface{}{1.0, 2.0, 3.0, 4.0, 5.0}

		t.Run("Mean", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.mean", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 3.0, result.Data["result"])
		})

		t.Run("Median", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.median", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 3.0, result.Data["result"])
		})

		t.Run("Min", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.min", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 1.0, result.Data["result"])
		})

		t.Run("Max", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.max", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 5.0, result.Data["result"])
		})

		t.Run("Sum", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.sum", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 15.0, result.Data["result"])
		})

		t.Run("Stdev", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.stdev", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.581, result.Data["result"].(float64), 0.001)
		})

		t.Run("Variance", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.variance", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 2.5, result.Data["result"].(float64), 0.001)
		})

		t.Run("Range", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.range", map[string]interface{}{
				"numbers": testData,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 4.0, result.Data["result"])
		})

		t.Run("Mode", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.mode", map[string]interface{}{
				"numbers": []interface{}{1.0, 2.0, 2.0, 3.0, 4.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 2.0, result.Data["result"])
			assert.Equal(t, 2, result.Data["frequency"])
		})

		t.Run("Percentile", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.percentile", map[string]interface{}{
				"numbers": testData,
				"p":       50.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 3.0, result.Data["result"])
		})

		t.Run("Correlation", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.correlation", map[string]interface{}{
				"x": []interface{}{1.0, 2.0, 3.0, 4.0, 5.0},
				"y": []interface{}{2.0, 4.0, 6.0, 8.0, 10.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.001)
		})

		t.Run("Correlation with mismatched arrays", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.correlation", map[string]interface{}{
				"x": []interface{}{1.0, 2.0, 3.0},
				"y": []interface{}{2.0, 4.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})

		t.Run("Covariance", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.covariance", map[string]interface{}{
				"x": []interface{}{1.0, 2.0, 3.0, 4.0, 5.0},
				"y": []interface{}{2.0, 4.0, 6.0, 8.0, 10.0},
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 5.0, result.Data["result"].(float64), 0.001)
		})
	})

	t.Run("Constants", func(t *testing.T) {
		t.Run("Pi", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.pi", map[string]interface{}{}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, math.Pi, result.Data["result"])
		})

		t.Run("E", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.e", map[string]interface{}{}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, math.E, result.Data["result"])
		})

		t.Run("Tau", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.tau", map[string]interface{}{}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 2*math.Pi, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Phi", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.phi", map[string]interface{}{}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.618, result.Data["result"].(float64), 0.001)
		})
	})

	t.Run("Conversions", func(t *testing.T) {
		t.Run("Celsius to Fahrenheit", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.celsiusToFahrenheit", map[string]interface{}{
				"celsius": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 32.0, result.Data["result"])
		})

		t.Run("Fahrenheit to Celsius", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.fahrenheitToCelsius", map[string]interface{}{
				"fahrenheit": 32.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 0.0, result.Data["result"].(float64), 0.001)
		})

		t.Run("Celsius to Kelvin", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.celsiusToKelvin", map[string]interface{}{
				"celsius": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 273.15, result.Data["result"])
		})

		t.Run("Kelvin to Celsius", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.kelvinToCelsius", map[string]interface{}{
				"kelvin": 273.15,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 0.0, result.Data["result"])
		})

		t.Run("Meters to Feet", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.metersToFeet", map[string]interface{}{
				"meters": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 3.28084, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Feet to Meters", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.feetToMeters", map[string]interface{}{
				"feet": 3.28084,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Miles to Kilometers", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.milesToKilometers", map[string]interface{}{
				"miles": 1.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.60934, result.Data["result"].(float64), 0.0001)
		})

		t.Run("Kilometers to Miles", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.kilometersToMiles", map[string]interface{}{
				"kilometers": 1.60934,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 1.0, result.Data["result"].(float64), 0.0001)
		})
	})

	t.Run("Precision Operations", func(t *testing.T) {
		t.Run("Precise Add", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.precise.add", map[string]interface{}{
				"numbers":   []interface{}{"0.1", "0.2"},
				"precision": 10.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			// Check that result is a string and starts with "0.3"
			resultStr, ok := result.Data["result"].(string)
			require.True(t, ok, "Result should be a string")
			assert.Contains(t, resultStr, "0.3")
		})

		t.Run("Precise Subtract", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.precise.subtract", map[string]interface{}{
				"a":         "1.0",
				"b":         "0.3",
				"precision": 10.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			resultStr, ok := result.Data["result"].(string)
			require.True(t, ok)
			assert.Contains(t, resultStr, "0.7")
		})

		t.Run("Precise Multiply", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.precise.multiply", map[string]interface{}{
				"numbers":   []interface{}{"0.1", "0.2"},
				"precision": 10.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			resultStr, ok := result.Data["result"].(string)
			require.True(t, ok)
			assert.Contains(t, resultStr, "0.02")
		})

		t.Run("Precise Divide", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.precise.divide", map[string]interface{}{
				"a":         "1.0",
				"b":         "3.0",
				"precision": 10.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			resultStr, ok := result.Data["result"].(string)
			require.True(t, ok)
			assert.Contains(t, resultStr, "0.333")
		})

		t.Run("Precise Divide by zero", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.precise.divide", map[string]interface{}{
				"a": "1.0",
				"b": "0.0",
			}, nil)
			require.NoError(t, err)
			testutil.AssertError(t, result)
		})
	})

	t.Run("Special Functions", func(t *testing.T) {
		t.Run("Gamma", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.gamma", map[string]interface{}{
				"x": 5.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.InDelta(t, 24.0, result.Data["result"].(float64), 0.001)
		})

		t.Run("Beta", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.beta", map[string]interface{}{
				"a": 2.0,
				"b": 3.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			// Beta(2,3) = Gamma(2)*Gamma(3)/Gamma(5) = 1*2/24 = 1/12 ≈ 0.0833
			assert.InDelta(t, 0.0833, result.Data["result"].(float64), 0.001)
		})

		t.Run("Erf", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.erf", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 0.0, result.Data["result"])
		})

		t.Run("Erfc", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.erfc", map[string]interface{}{
				"x": 0.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			assert.Equal(t, 1.0, result.Data["result"])
		})

		t.Run("Lgamma", func(t *testing.T) {
			result, err := mathProvider.Execute(ctx, "math.lgamma", map[string]interface{}{
				"x": 5.0,
			}, nil)
			require.NoError(t, err)
			testutil.AssertSuccess(t, result)
			// ln(Gamma(5)) = ln(24) ≈ 3.178
			assert.InDelta(t, 3.178, result.Data["result"].(float64), 0.001)
		})
	})

	t.Run("Service Definition", func(t *testing.T) {
		def := mathProvider.Definition()
		assert.Equal(t, "math", def.ID)
		assert.Equal(t, "Math Service", def.Name)
		assert.Equal(t, types.CategoryMath, def.Category)
		assert.NotEmpty(t, def.Tools)
		assert.GreaterOrEqual(t, len(def.Tools), 62) // 62 total tools
	})

	t.Run("Unknown Tool", func(t *testing.T) {
		result, err := mathProvider.Execute(ctx, "math.unknown", map[string]interface{}{}, nil)
		// The failure function returns both a Result and an error
		if err == nil {
			// If no error, the result should have an error
			testutil.AssertError(t, result)
		} else {
			// If error is returned, that's also valid
			assert.Error(t, err)
		}
	})
}
