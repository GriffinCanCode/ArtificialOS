// Package math provides comprehensive mathematical operations for AI-powered applications.
//
// This package is organized into specialized modules:
//   - arithmetic: Basic operations (add, subtract, multiply, divide, power, sqrt)
//   - trig: Trigonometric functions (sin, cos, tan, asin, acos, atan)
//   - stats: Statistical functions (mean, median, stdev, variance, correlation)
//   - constants: Mathematical constants (pi, e, tau, phi)
//   - conversions: Unit conversions (temperature, distance)
//   - precision: High-precision decimal arithmetic
//   - special: Special functions (gamma, beta, erf, erfc)
//
// Built on gonum.org/v1/gonum for scientific computing:
//   - IEEE 754 floating-point accuracy
//   - NaN and Infinity handling
//   - Statistical algorithms from R and NumPy
//
// Features:
//   - Input validation for edge cases
//   - Proper error handling for invalid operations
//   - Consistent JSON result format
//   - Support for arrays and scalar operations
//
// Example Usage:
//
//	arithmetic := &math.ArithmeticOps{MathOps: &math.MathOps{}}
//	result, err := arithmetic.Add(ctx, params, appCtx)
package common
