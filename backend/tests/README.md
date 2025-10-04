# Backend Tests

Centralized testing directory for the AI-OS Go backend.

## Directory Structure

```
tests/
├── README.md              # This file
├── helpers/               # Shared test utilities
│   └── testutil/         # Mock implementations, test data factories
├── integration/          # Integration tests (full stack)
└── fixtures/             # Test data files (JSON, YAML, etc.)
```

## Test Organization

### Unit Tests (Co-located)
Unit tests live alongside the code they test:
- `internal/config/config_test.go` - Tests for config package
- `internal/middleware/middleware_test.go` - Tests for middleware
- `internal/providers/storage_test.go` - Tests for storage provider

**Why?** This is Go's idiomatic approach and provides:
- Easy access to package-private functions
- Clear association between code and tests
- Support for both white-box and black-box testing

### Integration Tests (Centralized)
Integration tests that span multiple packages go in `tests/integration/`:
- `tests/integration/api_test.go` - Full API workflow tests
- `tests/integration/grpc_test.go` - gRPC service integration
- `tests/integration/e2e_test.go` - End-to-end tests

### Test Helpers (Centralized)
Shared test utilities in `tests/helpers/`:
- `tests/helpers/testutil/` - Mock implementations and factories
- Reusable across all test files

### Test Fixtures (Centralized)
Static test data in `tests/fixtures/`:
- `tests/fixtures/apps/` - Sample app UI specs
- `tests/fixtures/configs/` - Test configurations
- `tests/fixtures/blueprints/` - Blueprint test files

## Running Tests

```bash
# Run all tests
make test

# Run only unit tests (fast)
make test-unit

# Run only integration tests
make test-integration

# Run with coverage
make coverage

# Run with race detection
make test-race

# Run specific package
go test -v ./internal/config/...

# Run specific integration test
go test -v ./tests/integration/... -run TestFullAPI
```

## Writing Tests

### Unit Tests

Place in the same directory as the code:

```go
// internal/mypackage/mypackage_test.go
package mypackage

import (
    "testing"
    "github.com/stretchr/testify/assert"
)

func TestMyFunction(t *testing.T) {
    result := MyFunction()
    assert.Equal(t, expected, result)
}
```

### Integration Tests

Place in `tests/integration/`:

```go
// tests/integration/api_test.go
// +build integration

package integration

import (
    "testing"
    "github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
)

func TestFullAPIWorkflow(t *testing.T) {
    if testing.Short() {
        t.Skip("Skipping integration test")
    }
    
    // Full stack test
}
```

### Using Test Helpers

```go
import "github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"

func TestWithMock(t *testing.T) {
    mockKernel := testutil.NewMockKernelClient(t)
    mockKernel.On("CreateProcess", ...).Return(...)
    
    // Use mock in test
}
```

### Using Test Fixtures

```go
import "os"

func TestWithFixture(t *testing.T) {
    data, err := os.ReadFile("../../tests/fixtures/apps/sample.json")
    require.NoError(t, err)
    
    // Use fixture data
}
```

## Test Tags

Use build tags for different test types:

```go
// +build integration
```

Run with: `go test -tags=integration ./...`

## Coverage Goals

- **Critical paths**: 90%+ coverage
- **Business logic**: 80%+ coverage  
- **Handlers**: 70%+ coverage
- **Overall**: 75%+ coverage

Current coverage by package:
- `config`: 100% ✅
- `service`: 89.7%
- `providers`: High coverage
- `middleware`: Well tested

## Best Practices

1. **Co-locate unit tests** with code they test
2. **Centralize integration tests** in `tests/integration/`
3. **Share utilities** via `tests/helpers/`
4. **Use fixtures** for static test data
5. **Use table-driven tests** for multiple scenarios
6. **Mock external dependencies** properly
7. **Test error cases** and edge cases
8. **Keep tests fast** - mock slow operations
9. **Use meaningful names** - describe what's being tested
10. **Clean up resources** with `t.Cleanup()` or `defer`

## CI/CD Integration

Tests run automatically on:
- Pull requests
- Main branch pushes
- Manual workflow dispatch

### GitHub Actions Example

```yaml
- name: Run unit tests
  run: make test-unit

- name: Run integration tests
  run: make test-integration

- name: Generate coverage
  run: make coverage

- name: Upload coverage
  uses: codecov/codecov-action@v3
```

## Troubleshooting

### Tests are slow
- Use `testing.Short()` to skip slow tests
- Run with `-short` flag: `go test -short ./...`
- Mock external dependencies

### Flaky tests
- Avoid time-based assertions
- Use proper synchronization (channels, mutexes)
- Use deterministic test data

### Race conditions
- Always run with `-race` flag
- Fix data races immediately
- Use proper locking

## Resources

- [Go Testing Package](https://golang.org/pkg/testing/)
- [Testify Documentation](https://github.com/stretchr/testify)
- [Table Driven Tests](https://github.com/golang/go/wiki/TableDrivenTests)
- [Project Testing Guide](../TESTING.md)

