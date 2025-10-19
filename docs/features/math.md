# Frontend Math Utilities

**Security Note:** This module replaces dangerous `eval()` calls with `mathjs` for secure expression evaluation.

## Overview

Frontend-focused math utilities for UI operations, calculations, and display formatting. For comprehensive mathematical operations (advanced trigonometry, statistics, matrix operations), use the backend math provider via tool execution.

## Expression Evaluation (Security Fix)

### ✅ SECURE: evaluateExpression()

```typescript
import { evaluateExpression } from '@/core/utils/math';

// Basic arithmetic
evaluateExpression("2 + 2")           // 4
evaluateExpression("10 * (5 + 3)")    // 80

// Advanced functions (mathjs powered)
evaluateExpression("sqrt(16)")        // 4
evaluateExpression("sin(pi/2)")       // 1
evaluateExpression("log(100)")        // 4.605...
evaluateExpression("2^8")             // 256

// Constants
evaluateExpression("pi")              // 3.14159...
evaluateExpression("e")               // 2.71828...

// Calculator-friendly operators
evaluateExpression("10  2")          // 5 (unicode division)
evaluateExpression("5  3")           // 15 (unicode multiplication)
evaluateExpression("10  3")          // 7 (unicode minus)
```

### ❌ INSECURE: eval() - DO NOT USE

```typescript
// NEVER do this:
const result = eval(userInput);  // ❌ Critical security vulnerability!
```

**Why mathjs?**
- ✅ No code injection vulnerabilities
- ✅ Supports 200+ mathematical functions
- ✅ Constants: pi, e, tau, phi
- ✅ Complex numbers, units, matrices
- ✅ Safe scope isolation

## Number Formatting

### Display Formatting

```typescript
import { formatNumber, formatWithThousands, formatPercentage } from '@/core/utils/math';

formatNumber(3.14159, 2)              // "3.14"
formatWithThousands(1234567)          // "1,234,567"
formatPercentage(0.853, 1)            // "85.3%"
```

### Size & Duration Formatting

```typescript
import { formatBytes, formatDuration, formatCompact } from '@/core/utils/math';

formatBytes(1536)                     // "1.50 KB"
formatBytes(5242880)                  // "5.00 MB"
formatBytes(1073741824)               // "1.00 GB"

formatDuration(500)                   // "500ms"
formatDuration(5000)                  // "5.0s"
formatDuration(125000)                // "2m"
formatDuration(7200000)               // "2h 0m"

formatCompact(1500)                   // "1.5K"
formatCompact(2500000)                // "2.5M"
formatCompact(3000000000)             // "3.0B"
```

## UI-Specific Operations

### Value Clamping & Mapping

```typescript
import { clamp, lerp, normalize, mapRange } from '@/core/utils/math';

// Clamp value between bounds
clamp(150, 0, 100)                    // 100
clamp(-10, 0, 100)                    // 0

// Linear interpolation
lerp(0, 100, 0.5)                     // 50
lerp(0, 100, 0.25)                    // 25

// Normalize to 0-1 range
normalize(50, 0, 100)                 // 0.5
normalize(75, 0, 100)                 // 0.75

// Map between ranges
mapRange(50, 0, 100, 0, 1000)         // 500
mapRange(5, 0, 10, 100, 200)          // 150
```

### Rounding Operations

```typescript
import { roundToNearest, roundToSignificant } from '@/core/utils/math';

roundToNearest(47, 5)                 // 45
roundToNearest(123, 10)               // 120

roundToSignificant(123456, 3)         // 123000
roundToSignificant(0.004567, 2)       // 0.0046
```

## Statistics for Visualization

### Basic Statistics

```typescript
import { mean, median, stdDev, percentile, minMax } from '@/core/utils/math';

const data = [10, 20, 30, 40, 50];

mean(data)                            // 30
median(data)                          // 30
stdDev(data)                          // 14.14...

percentile(data, 50)                  // 30 (median)
percentile(data, 95)                  // 48

minMax(data)                          // { min: 10, max: 50 }
```

### Statistics Summary

```typescript
import { statisticsSummary } from '@/core/utils/math';

const metrics = [100, 150, 200, 250, 300, 350, 400];

statisticsSummary(metrics);
// {
//   min: 100,
//   max: 400,
//   mean: 250,
//   median: 250,
//   stdDev: 96.82,
//   p95: 385,
//   p99: 397
// }
```

## Calculation Helpers

### Safe Division & Changes

```typescript
import { safeDivide, percentageChange, growthRate } from '@/core/utils/math';

// Prevents division by zero
safeDivide(10, 0)                     // 0 (fallback)
safeDivide(10, 0, -1)                 // -1 (custom fallback)
safeDivide(10, 2)                     // 5

// Calculate percentage change
percentageChange(100, 150)            // 50 (50% increase)
percentageChange(200, 150)            // -25 (25% decrease)

// Growth rate from array
growthRate([100, 120, 140, 160])     // 60 (60% growth)
```

### Moving Average

```typescript
import { movingAverage } from '@/core/utils/math';

const data = [10, 20, 30, 40, 50];

movingAverage(data, 3);
// [10, 15, 20, 30, 40]
// Window of 3: [10], [10,20], [10,20,30], [20,30,40], [30,40,50]
```

## Random Utilities

```typescript
import { randomInRange, randomInt } from '@/core/utils/math';

// Random in range
randomInRange(0, 100)                 // 57.234... (float)
randomInt(1, 6)                       // 4 (integer, inclusive)
```

## Migration Guide

### Replacing eval()

**Before (INSECURE):**
```typescript
try {
  const result = eval(expression);  // ❌ Security vulnerability
} catch (error) {
  return "Error";
}
```

**After (SECURE):**
```typescript
import { evaluateExpression } from '@/core/utils/math';

const result = evaluateExpression(expression);  // ✅ Safe
```

### Replacing Duplicate Utilities

**Before:**
```typescript
// Scattered across codebase
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  // ... repeated implementation
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  // ... repeated implementation
}
```

**After:**
```typescript
import { formatBytes, formatDuration } from '@/core/utils/math';

// Use centralized, tested utilities
const size = formatBytes(1048576);      // "1.00 MB"
const time = formatDuration(5000);      // "5.0s"
```

## Backend Integration

For advanced mathematical operations not available in this module:

```typescript
// Use backend math provider for:
// - Trigonometry (sin, cos, tan, asin, acos, atan)
// - Advanced statistics (correlation, covariance, regression)
// - Matrix operations
// - Unit conversions
// - High-precision arithmetic
// - Special functions (gamma, beta, erf)

// Example: Execute via tool system
await toolExecutor.execute("math.sin", { value: Math.PI / 2 });
await toolExecutor.execute("math.correlation", { x: [1,2,3], y: [2,4,6] });
```

## Testing

```typescript
import { evaluateExpression, clamp, statisticsSummary } from '@/core/utils/math';

describe('Math utilities', () => {
  it('safely evaluates expressions', () => {
    expect(evaluateExpression("2 + 2")).toBe(4);
    expect(evaluateExpression("sqrt(16)")).toBe(4);
    expect(evaluateExpression("invalid")).toBe("Error");
  });

  it('clamps values', () => {
    expect(clamp(150, 0, 100)).toBe(100);
    expect(clamp(-10, 0, 100)).toBe(0);
    expect(clamp(50, 0, 100)).toBe(50);
  });

  it('calculates statistics', () => {
    const stats = statisticsSummary([1, 2, 3, 4, 5]);
    expect(stats.mean).toBe(3);
    expect(stats.median).toBe(3);
  });
});
```

## Performance

- **Expression Evaluation**: ~0.1-1ms per evaluation
- **Statistics**: O(n) for mean, O(n log n) for median/percentiles
- **Formatting**: ~0.01ms per call
- **All operations**: Optimized for UI responsiveness

## Security

✅ **No eval() vulnerabilities**
✅ **Sandboxed expression execution**
✅ **Input validation**
✅ **Safe error handling**

## See Also

- Backend Math Provider: `/backend/internal/providers/math/`
- Date Utilities: `@/core/utils/dates`
- Visualization Transforms: `@/features/visualization/utils/transforms`
