# Testing Quick Reference

## 🚀 Quick Commands

```bash
# Fast development cycle
make test-unit              # Run unit tests only (recommended for TDD)

# Full test suite
make test                   # All tests with race detection

# Integration tests
make test-integration       # Integration tests only (tagged)

# Coverage
make coverage              # Generate HTML coverage report
open coverage.html         # View coverage in browser

# Watch mode
make test-watch            # Auto-run tests on file changes (requires entr)
```

## 📁 Where to Put Tests

```
✅ Unit tests → Next to code
   internal/config/config_test.go

✅ Integration tests → tests/integration/
   tests/integration/api_test.go

✅ Test helpers → tests/helpers/
   tests/helpers/testutil/testutil.go

✅ Test data → tests/fixtures/
   tests/fixtures/apps/sample.json
```

## 📝 Test Template

### Unit Test (Co-located)

```go
// internal/mypackage/mypackage_test.go
package mypackage

import (
    "testing"
    "github.com/stretchr/testify/assert"
)

func TestMyFunction(t *testing.T) {
    result := MyFunction("input")
    assert.Equal(t, "expected", result)
}
```

### Integration Test (Centralized)

```go
// tests/integration/mytest_test.go
// +build integration

package integration

import (
    "testing"
    "github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
)

func TestIntegration(t *testing.T) {
    if testing.Short() {
        t.Skip("Skipping integration test")
    }
    
    mock := testutil.NewMockKernelClient(t)
    // Test multiple components together
}
```

## 🔧 Common Patterns

### Table-Driven Test

```go
func TestValidate(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        wantErr bool
    }{
        {"valid", "hello", false},
        {"empty", "", true},
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := Validate(tt.input)
            if tt.wantErr {
                assert.Error(t, err)
            } else {
                assert.NoError(t, err)
            }
        })
    }
}
```

### Using Mocks

```go
import "github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"

func TestWithMock(t *testing.T) {
    mock := testutil.NewMockKernelClient(t)
    
    // Mock returns predefined value
    pid := uint32(123)
    mock.On("CreateProcess", 
        mock.Anything, "test", uint32(5), "STANDARD",
    ).Return(&pid, nil)
    
    // Use mock
    result, err := mock.CreateProcess(ctx, "test", 5, "STANDARD")
    
    // Verify
    assert.NoError(t, err)
    mock.AssertExpectations(t)
}
```

### HTTP Handler Test

```go
func TestHandler(t *testing.T) {
    router := gin.New()
    router.GET("/test", MyHandler)
    
    req := httptest.NewRequest("GET", "/test", nil)
    w := httptest.NewRecorder()
    router.ServeHTTP(w, req)
    
    assert.Equal(t, 200, w.Code)
}
```

## 🎯 Coverage Goals

- Critical paths: **90%+**
- Business logic: **80%+**
- Handlers: **70%+**
- Overall: **75%+**

## 📊 Current Status

```
✅ config:      100%  
✅ middleware:  ~90%  
✅ service:     89.7%
✅ providers:   ~85%
⚠️  grpc:       ~70%
⚠️  http:       ~60%
```

## 🐛 Common Issues

### Slow tests?
```bash
go test -short ./...     # Skip slow tests
make test-unit           # Unit tests only
```

### Race conditions?
```bash
make test-race           # Always use race detector
```

### Flaky tests?
- Use deterministic data
- Avoid time-based assertions
- Mock external dependencies

## 📚 Full Documentation

- **Comprehensive Guide**: See [TESTING.md](../TESTING.md)
- **Detailed Info**: See [tests/README.md](./README.md)
- **Examples**: Look at existing `*_test.go` files

## 💡 Tips

1. **Run unit tests frequently** during development
2. **Use `-v` flag** for verbose output when debugging
3. **Keep tests fast** - mock slow operations
4. **Test error cases** - not just happy paths
5. **Use meaningful names** - describe what's being tested
6. **Clean up resources** with `defer` or `t.Cleanup()`
7. **Run with `-race`** to catch concurrency issues
8. **Write tests first** when fixing bugs (TDD)

## 🔗 Quick Links

- Run tests: `make test-unit`
- View coverage: `make coverage && open coverage.html`
- Read guide: `cat TESTING.md`
- See examples: `ls internal/*/\*_test.go`

---

**Need help?** Check [TESTING.md](../TESTING.md) or ask the team!

