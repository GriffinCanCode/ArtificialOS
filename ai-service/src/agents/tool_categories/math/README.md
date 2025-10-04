# Math Tools Package

Comprehensive mathematical operations organized by domain.

## Architecture

Uses specialized libraries for different mathematical domains:
- **mpmath** (≥1.3.0): Arbitrary-precision floating-point arithmetic
- **sympy** (≥1.12): Symbolic mathematics (algebra, calculus)
- **Python built-ins**: `math`, `statistics` modules

## Organization

```
math/
├── __init__.py           # Package registration
├── arithmetic.py         # Basic ops, logarithms, factorials
├── trigonometry.py       # Sin, cos, tan, hyperbolic functions
├── statistics.py         # Mean, median, stdev, correlation
├── algebra.py            # Solve, factor, matrix operations
├── calculus.py           # Derivatives, integrals, limits
├── constants.py          # π, e, φ with precision control
└── conversions.py        # Temperature, length conversions
```

## Tool Categories

### Arithmetic (24 tools)
- **Basic**: add, subtract, multiply, divide
- **Powers**: power, sqrt, root
- **Rounding**: floor, ceil, round, abs, mod
- **Logarithms**: exp, log, log10, log2
- **Special**: factorial, gcd, lcm

### Trigonometry (13 tools)
- **Basic**: sin, cos, tan (radians/degrees)
- **Inverse**: asin, acos, atan, atan2
- **Hyperbolic**: sinh, cosh, tanh
- **Conversions**: radians, degrees

### Statistics (15 tools)
- **Descriptive**: mean, median, mode, stdev, variance
- **Range**: min, max, range, sum
- **Percentiles**: percentile, quantile, quartiles
- **Correlation**: correlation, covariance

### Algebra (11 tools)
- **Equations**: solve, solve_linear, quadratic
- **Polynomials**: expand, factor, simplify
- **Matrices**: multiply, determinant, inverse, transpose

### Calculus (9 tools)
- **Derivatives**: derivative, derivative_at, partial
- **Integrals**: integrate, integrate_definite
- **Limits**: limit
- **Series**: series, taylor

### Constants (6 tools)
- Fundamental: pi, e, tau, phi
- Special: infinity, nan

### Conversions (8 tools)
- **Temperature**: celsius↔fahrenheit, celsius↔kelvin
- **Length**: meters↔feet, miles↔kilometers

## Usage Examples

```python
# Basic arithmetic
math.add(numbers=[1, 2, 3, 4])  # 10
math.power(base=2, exponent=8)  # 256

# Trigonometry
math.sin(x=1.5708, unit="rad")  # ≈1.0 (90°)
math.degrees(radians=3.14159)   # ≈180

# Statistics
math.mean(numbers=[1, 2, 3, 4, 5])  # 3.0
math.stdev(numbers=[1, 2, 3, 4, 5]) # ≈1.41

# Algebra
math.solve(equation="x^2 - 4 = 0", variable="x")  # [-2, 2]
math.simplify(expression="(x^2 - 1)/(x - 1)")     # x + 1

# Calculus
math.derivative(expression="x^2", variable="x")  # "2*x"
math.integrate(expression="2*x", variable="x")   # "x^2"

# Constants
math.pi(precision=5)  # 3.14159

# Conversions
math.celsius_to_fahrenheit(celsius=100)  # 212
```

## Design Principles

1. **Small Functions**: Each tool does one thing well
2. **Strong Typing**: Pydantic models with explicit types
3. **Zero Duplication**: No overlapping functionality
4. **Extensibility**: Easy to add new tools/categories
5. **Testability**: Pure functions, minimal dependencies

## Relationship to `ui.compute`

**NOT duplicates** - they serve different purposes:

- **`ui.compute`**: UI integration tool
  - Updates UI state fields (e.g., calculator display)
  - Example: Button click → evaluate → update display
  
- **`math.*` tools**: Pure computation
  - Return values directly
  - Example: API call → compute → return result

Both tools are complementary and should coexist.

## Future Extensions

Potential additions:
- **Geometry**: area, volume, distance calculations
- **Probability**: distributions, random sampling
- **Number Theory**: primes, divisors, modular arithmetic
- **Linear Algebra**: eigenvalues, SVD, rank
- **Complex Numbers**: complex arithmetic, polar form
- **Optimization**: minimize, maximize, solve constraints
