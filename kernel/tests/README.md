# Kernel Test Suite

Comprehensive test suite for the AI-OS kernel.

## Test Organization

All tests are centrally located in the `tests/` directory for easy access and management.

### Test Files
- **memory_test.rs**: Memory allocation, deallocation, OOM handling, garbage collection (16 tests)
- **process_test.rs**: Process lifecycle, creation, termination, memory integration (10 tests)
- **syscall_test.rs**: System call execution, sandboxing, permission checks (19 tests)
- **ipc_test.rs**: Inter-process communication, message queuing, memory limits (14 tests)
- **signals_test.rs**: Signal handling, RT signals, callbacks, priority queuing (87+ tests)
- **integration_tests.rs**: End-to-end integration tests (12 tests)

### Module Tests
Some modules include embedded unit tests:
- **sandbox.rs**: Capability management, path access control (3 tests)

### Total Test Coverage
- **150+ tests** covering all major kernel subsystems

## Running Tests

### All Tests
```bash
make test
```

### Unit Tests Only
```bash
make test-unit
```

### Integration Tests Only
```bash
make test-integration
```

### Verbose Output
```bash
make test-verbose
```

### Quick Tests (minimal output)
```bash
make test-quick
```

### Specific Test Files
```bash
make test-memory    # Run memory_test.rs
make test-process   # Run process_test.rs
make test-syscall   # Run syscall_test.rs
make test-ipc       # Run ipc_test.rs
make test-signals   # Run signals_test.rs
```

### Direct Cargo Commands
```bash
# Run all tests
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test test_basic_allocation -- --nocapture

# Run tests in a specific file
cargo test --test integration_tests
```

## Test Coverage

Generate coverage report (requires cargo-tarpaulin):

```bash
make coverage
```

View coverage report:
```bash
open coverage/index.html
```

## Testing Libraries

The test suite uses:

- **pretty_assertions**: Better assertion output with colored diffs
- **proptest**: Property-based testing for edge cases
- **mockall**: Mocking framework for complex dependencies
- **tempfile**: Temporary file/directory creation for filesystem tests
- **serial_test**: Serial test execution for tests that can't run in parallel
- **tokio-test**: Async testing utilities

## Test Guidelines

### Writing New Tests

1. **Use descriptive names**: `test_allocation_fails_when_out_of_memory`
2. **Test one thing**: Each test should verify a single behavior
3. **Use setup helpers**: Extract common setup into helper functions
4. **Clean up resources**: Use tempfile::TempDir for filesystem tests
5. **Use assertions with context**: pretty_assertions provides better error messages

### Test Structure

```rust
#[test]
fn test_something_specific() {
    // Arrange: Setup test environment
    let manager = MemoryManager::new();
    let pid = 100;
    
    // Act: Perform the operation
    let result = manager.allocate(1024, pid);
    
    // Assert: Verify the result
    assert!(result.is_ok());
    assert_eq!(manager.get_process_memory(pid), 1024);
}
```

### Concurrent Tests

For tests that modify global state, use `#[serial]`:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_with_global_state() {
    // This test won't run in parallel with other serial tests
}
```

### Async Tests

For async operations:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = some_async_function().await;
    assert!(result.is_ok());
}
```

### Property-Based Tests

For testing with random inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_allocation_size(size in 1024usize..10_000_000usize) {
        let manager = MemoryManager::new();
        let result = manager.allocate(size, 100);
        // Verify properties that should hold for any size
    }
}
```

## Test Coverage Goals

Target coverage by module:
- Memory management: 90%+
- Process management: 85%+
- Syscall execution: 85%+
- Sandbox enforcement: 90%+
- IPC: 80%+
- gRPC server: 70%+ (harder to test)

## CI/CD Integration

Tests run automatically on:
- Every commit (unit tests)
- Pull requests (all tests + coverage)
- Main branch (all tests + coverage + benchmarks)

## Performance Tests

For performance-critical code, add benchmarks:

```bash
make bench
```

## Debugging Tests

Run a single test with logging:

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

## Common Issues

### Test Failures Due to Timing
- Use proper synchronization primitives
- Don't rely on sleep() for timing

### Flaky Tests
- Identify race conditions
- Use deterministic test data
- Mark with `#[serial]` if needed

### Resource Cleanup
- Always use `TempDir` for filesystem tests
- Clean up processes in tests
- Don't leave zombie processes

## Adding New Tests

1. Create test file or add to existing module test
2. Write descriptive test function
3. Run locally to verify: `make test`
4. Check coverage: `make coverage`
5. Commit and push

## Test Reporting

View test results in CI:
- GitHub Actions: Check the "Tests" workflow
- Coverage: View on code coverage service

## Questions?

See the main README or kernel documentation for more information.

