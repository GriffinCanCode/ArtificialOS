# Math Provider

Production-grade mathematical operations using specialized Go libraries.

## Architecture

```
math/
├── types.go        # Shared types, validation, helpers
├── arithmetic.go   # Basic arithmetic (17 tools)
├── trig.go         # Trigonometric functions (12 tools)
├── stats.go        # Statistics with gonum (12 tools)
├── constants.go    # Mathematical constants (4 tools)
├── conversions.go  # Unit conversions (8 tools)
├── precision.go    # High-precision arithmetic (4 tools)
└── special.go      # Special functions with gonum (5 tools)
```

## Libraries Used

### gonum.org/v1/gonum/stat
**Purpose**: Production-grade statistical operations
**Benefits**:
- Numerical stability optimizations
- Multiple quantile estimation methods
- Weighted statistics support
- Battle-tested in scientific computing
- ~3-10x faster than naive implementations

**Tools**:
- `math.mean` - Arithmetic mean
- `math.median` - Median using empirical quantile
- `math.stdev` - Sample standard deviation
- `math.variance` - Sample variance
- `math.percentile` - Percentile calculation
- `math.correlation` - Pearson correlation coefficient
- `math.covariance` - Sample covariance

### gonum.org/v1/gonum/mathext
**Purpose**: Advanced mathematical functions
**Benefits**:
- Specialized implementations of special functions
- Better accuracy than naive approximations
- Handles edge cases properly

**Tools**:
- `math.beta` - Beta function
- `math.gamma` - Gamma function (via math.Gamma)
- `math.lgamma` - Log gamma function
- `math.erf` - Error function
- `math.erfc` - Complementary error function

### math/big
**Purpose**: Arbitrary precision arithmetic
**Benefits**:
- Exact decimal arithmetic (no floating point errors)
- Critical for financial calculations
- Configurable precision
- Prevents "0.1 + 0.2 ≠ 0.3" issues

**Tools**:
- `math.precise.add` - High-precision addition
- `math.precise.subtract` - High-precision subtraction
- `math.precise.multiply` - High-precision multiplication
- `math.precise.divide` - High-precision division

## Design Principles

### 1. Strong Typing
- Type coercion for int/float variants
- Explicit validation functions
- No `interface{}` abuse

### 2. Error Handling
- Validate all inputs (NaN, Inf checks)
- Clear error messages
- Domain validation (e.g., sqrt of negative)

### 3. Extensibility
- Modular design - each file is independent
- Easy to add new operations
- Consistent patterns across modules

### 4. Performance
- Zero allocations for hot paths
- Pre-allocated slices where possible
- Efficient sorting (gonum optimizations)

### 5. Testability
- Pure functions (no side effects)
- Minimal dependencies
- Clear input/output contracts

## Usage Examples

### Standard Operations
```go
// Using standard math operations
params := map[string]interface{}{
    "numbers": []interface{}{1.0, 2.0, 3.0, 4.0, 5.0},
}
result, _ := stats.Mean(ctx, params, nil)
// result.Data["result"] = 3.0
```

### High-Precision Arithmetic
```go
// For financial calculations requiring exact precision
params := map[string]interface{}{
    "numbers":   []string{"0.1", "0.2"},
    "precision": 20, // decimal places
}
result, _ := precision.PreciseAdd(ctx, params, nil)
// result.Data["result"] = "0.30000000000000000000" (exact!)
```

### Statistical Analysis
```go
// Using gonum for production-grade stats
x := []interface{}{1.0, 2.0, 3.0, 4.0, 5.0}
y := []interface{}{2.0, 4.0, 5.0, 4.0, 5.0}
params := map[string]interface{}{"x": x, "y": y}
result, _ := stats.Correlation(ctx, params, nil)
// Pearson correlation coefficient
```

## Tool Count by Module

| Module      | Tools | Primary Library        |
|-------------|-------|------------------------|
| Arithmetic  | 17    | math (stdlib)          |
| Trig        | 12    | math (stdlib)          |
| Stats       | 12    | gonum/stat             |
| Constants   | 4     | math (stdlib)          |
| Conversions | 8     | math (stdlib)          |
| Precision   | 4     | math/big               |
| Special     | 5     | gonum/mathext          |
| **TOTAL**   | **62**| **Specialized**        |

## Performance Characteristics

### Standard Operations
- **Arithmetic**: O(1) constant time
- **Trigonometry**: O(1) assembly-optimized
- **Min/Max**: O(n) single pass
- **Sum**: O(n) single pass

### Statistical Operations
- **Mean**: O(n) single pass
- **Median**: O(n log n) with sort (gonum optimized)
- **Variance/Stdev**: O(n) two-pass (numerically stable)
- **Percentile**: O(n log n) with quantile interpolation
- **Correlation**: O(n) single pass (gonum optimized)

### Precision Operations
- **All operations**: O(n * p) where p = precision bits
- Default precision: 53 bits (float64 equivalent)
- Configurable up to arbitrary precision

## Future Enhancements

### Potential Additions
- Matrix operations (gonum/mat)
- Linear algebra (eigenvalues, SVD)
- Numerical integration (gonum/integrate)
- Optimization routines (gonum/optimize)
- Random distributions (gonum/stat/distuv)
- Polynomial operations
- Complex number operations

### When to Add
- Matrix ops: When UI apps need linear algebra
- Integration: When apps need area under curve
- Optimization: When apps need function minimization
- Distributions: When apps need probability functions

## Testing Strategy

### Unit Tests
- Test each function independently
- Cover edge cases (NaN, Inf, zero, negative)
- Validate error messages
- Compare with known values