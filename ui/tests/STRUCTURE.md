# Test Structure Documentation

## Overview

All tests have been centralized in the `/tests` directory for better organization and maintainability.

## Directory Structure

```
tests/
├── setup/                      # Test configuration and utilities
│   ├── setup.ts               # Global test setup (runs before all tests)
│   └── utils.tsx              # Custom render functions and test helpers
│
├── components/                 # Component tests
│   ├── chat/
│   │   └── ChatInterface.test.tsx
│   └── dynamics/
│       └── DynamicRenderer.state.test.ts
│
├── store/                      # State management tests
│   └── appStore.test.ts
│
├── hooks/                      # Custom hook tests
│   └── useLogger.test.ts
│
├── utils/                      # Utility function tests
│   └── api/
│       └── websocketClient.test.ts
│
├── README.md                   # Testing guide and best practices
└── STRUCTURE.md               # This file
```

## Benefits of Centralized Testing

### 1. **Clear Separation**
- Source code in `src/`
- All tests in `tests/`
- Easy to exclude from builds
- Clear mental model

### 2. **Consistent Imports**
Tests import from source using consistent relative paths:
```tsx
// From tests/components/chat/
import Component from "../../../src/components/chat/Component";
import { useStore } from "../../../src/store/appStore";
import { render } from "../../setup/utils";
```

### 3. **Shared Test Utilities**
All tests use the same setup and utilities from `tests/setup/`:
- `setup.ts`: Global mocks, DOM setup
- `utils.tsx`: Custom render, test helpers, mock factories

### 4. **Easy Navigation**
- Test structure mirrors source structure
- Find component test by replacing `src/` with `tests/`
- Example: `src/store/appStore.ts` → `tests/store/appStore.test.ts`

### 5. **Better CI/CD**
- Single directory to target for test coverage
- Clear exclude patterns in coverage config
- Easier to cache test results

## Running Tests

### Using npm scripts:
```bash
npm test              # Watch mode
npm run test:run      # Run once
npm run test:ui       # Visual UI
npm run test:coverage # With coverage
```

### Using Makefile:
```bash
make test             # Watch mode
make test-run         # Run once
make test-ui          # Visual UI
make test-coverage    # With coverage
```

## Configuration Files

### vitest.config.ts
```typescript
{
  setupFiles: ["./tests/setup/setup.ts"],
  include: ["tests/**/*.{test,spec}.{ts,tsx}"],
  coverage: {
    exclude: ["tests/", "node_modules/", ...]
  }
}
```

### Test File Naming
- Component tests: `ComponentName.test.tsx`
- Store tests: `storeName.test.ts`
- Hook tests: `hookName.test.ts`
- Utility tests: `utilityName.test.ts`

## Migration Notes

### Before (Scattered):
```
src/
├── components/
│   └── chat/
│       ├── ChatInterface.tsx
│       └── ChatInterface.test.tsx  ❌ Mixed with source
├── store/
│   ├── appStore.ts
│   └── appStore.test.ts            ❌ Mixed with source
└── test/                            ❌ Shared utilities in src/
    ├── setup.ts
    └── utils.tsx
```

### After (Centralized):
```
src/                  ✅ Only source code
├── components/
├── store/
└── utils/

tests/                ✅ All tests together
├── setup/
├── components/
├── store/
└── utils/
```

## Best Practices

1. **Mirror Source Structure**
   - Tests in `tests/` mirror structure in `src/`
   - Makes finding tests intuitive

2. **Consistent Imports**
   - Always import from `../../src/...`
   - Use `../setup/utils` for test utilities

3. **Shared Setup**
   - Global setup in `tests/setup/setup.ts`
   - Shared utilities in `tests/setup/utils.tsx`

4. **Clear Test Names**
   - Use `.test.ts` or `.test.tsx` suffix
   - Name matches source file being tested

5. **Organized by Feature**
   - Group related tests in same directory
   - Matches feature organization in source

## Adding New Tests

1. **Identify the feature location** in `src/`
2. **Create parallel path** in `tests/`
3. **Import from source** using relative paths
4. **Use shared utilities** from `tests/setup/`

Example:
```tsx
// New file: tests/components/newFeature/NewComponent.test.tsx

import { render, screen } from "../../setup/utils";
import NewComponent from "../../../src/components/newFeature/NewComponent";

describe("NewComponent", () => {
  it("renders correctly", () => {
    render(<NewComponent />);
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });
});
```

## Coverage Reports

Coverage reports are generated in `coverage/` directory:
```bash
make test-coverage
open coverage/index.html  # View in browser
```

Coverage excludes:
- `tests/` directory
- `node_modules/`
- Configuration files
- Type definitions

## Troubleshooting

### Import errors
- Check relative path depth
- Verify file exists in `src/`
- Update path if structure changes

### Setup not running
- Verify `vitest.config.ts` points to `tests/setup/setup.ts`
- Check setup file syntax

### Coverage not excluding tests
- Verify `exclude` pattern in `vitest.config.ts`
- Should include `"tests/"` in exclude list

## Future Improvements

- [ ] Add E2E tests directory
- [ ] Add performance benchmarks
- [ ] Add visual regression tests
- [ ] Add API integration tests
- [ ] Add accessibility tests

