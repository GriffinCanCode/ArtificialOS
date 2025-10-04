package math

import (
	"context"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// ConversionsOps handles unit conversions
type ConversionsOps struct {
	*MathOps
}

// GetTools returns conversion tool definitions
func (c *ConversionsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "math.celsiusToFahrenheit",
			Name:        "Celsius to Fahrenheit",
			Description: "Convert temperature from Celsius to Fahrenheit",
			Parameters: []types.Parameter{
				{Name: "celsius", Type: "number", Description: "Temperature in Celsius", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.fahrenheitToCelsius",
			Name:        "Fahrenheit to Celsius",
			Description: "Convert temperature from Fahrenheit to Celsius",
			Parameters: []types.Parameter{
				{Name: "fahrenheit", Type: "number", Description: "Temperature in Fahrenheit", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.metersToFeet",
			Name:        "Meters to Feet",
			Description: "Convert length from meters to feet",
			Parameters: []types.Parameter{
				{Name: "meters", Type: "number", Description: "Length in meters", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.feetToMeters",
			Name:        "Feet to Meters",
			Description: "Convert length from feet to meters",
			Parameters: []types.Parameter{
				{Name: "feet", Type: "number", Description: "Length in feet", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.milesToKilometers",
			Name:        "Miles to Kilometers",
			Description: "Convert distance from miles to kilometers",
			Parameters: []types.Parameter{
				{Name: "miles", Type: "number", Description: "Distance in miles", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.kilometersToMiles",
			Name:        "Kilometers to Miles",
			Description: "Convert distance from kilometers to miles",
			Parameters: []types.Parameter{
				{Name: "kilometers", Type: "number", Description: "Distance in kilometers", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.celsiusToKelvin",
			Name:        "Celsius to Kelvin",
			Description: "Convert temperature from Celsius to Kelvin",
			Parameters: []types.Parameter{
				{Name: "celsius", Type: "number", Description: "Temperature in Celsius", Required: true},
			},
			Returns: "number",
		},
		{
			ID:          "math.kelvinToCelsius",
			Name:        "Kelvin to Celsius",
			Description: "Convert temperature from Kelvin to Celsius",
			Parameters: []types.Parameter{
				{Name: "kelvin", Type: "number", Description: "Temperature in Kelvin", Required: true},
			},
			Returns: "number",
		},
	}
}

// CelsiusToFahrenheit converts C to F
func (c *ConversionsOps) CelsiusToFahrenheit(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	celsius, ok := GetNumber(params, "celsius")
	if !ok {
		return Failure("celsius parameter required")
	}
	fahrenheit := (celsius * 9 / 5) + 32
	return Success(map[string]interface{}{"result": fahrenheit})
}

// FahrenheitToCelsius converts F to C
func (c *ConversionsOps) FahrenheitToCelsius(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	fahrenheit, ok := GetNumber(params, "fahrenheit")
	if !ok {
		return Failure("fahrenheit parameter required")
	}
	celsius := (fahrenheit - 32) * 5 / 9
	return Success(map[string]interface{}{"result": celsius})
}

// MetersToFeet converts meters to feet
func (c *ConversionsOps) MetersToFeet(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	meters, ok := GetNumber(params, "meters")
	if !ok {
		return Failure("meters parameter required")
	}
	feet := meters * 3.28084
	return Success(map[string]interface{}{"result": feet})
}

// FeetToMeters converts feet to meters
func (c *ConversionsOps) FeetToMeters(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	feet, ok := GetNumber(params, "feet")
	if !ok {
		return Failure("feet parameter required")
	}
	meters := feet / 3.28084
	return Success(map[string]interface{}{"result": meters})
}

// MilesToKilometers converts miles to kilometers
func (c *ConversionsOps) MilesToKilometers(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	miles, ok := GetNumber(params, "miles")
	if !ok {
		return Failure("miles parameter required")
	}
	kilometers := miles * 1.60934
	return Success(map[string]interface{}{"result": kilometers})
}

// KilometersToMiles converts kilometers to miles
func (c *ConversionsOps) KilometersToMiles(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	kilometers, ok := GetNumber(params, "kilometers")
	if !ok {
		return Failure("kilometers parameter required")
	}
	miles := kilometers / 1.60934
	return Success(map[string]interface{}{"result": miles})
}

// CelsiusToKelvin converts C to K
func (c *ConversionsOps) CelsiusToKelvin(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	celsius, ok := GetNumber(params, "celsius")
	if !ok {
		return Failure("celsius parameter required")
	}
	kelvin := celsius + 273.15
	return Success(map[string]interface{}{"result": kelvin})
}

// KelvinToCelsius converts K to C
func (c *ConversionsOps) KelvinToCelsius(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	kelvin, ok := GetNumber(params, "kelvin")
	if !ok {
		return Failure("kelvin parameter required")
	}
	celsius := kelvin - 273.15
	return Success(map[string]interface{}{"result": celsius})
}
