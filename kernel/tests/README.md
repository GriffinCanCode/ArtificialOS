# Kernel Test Suite

Comprehensive test suite for the AI-OS kernel.

## Test Organization

Tests are organized by domain in the `tests/` directory for easy navigation and management.

### Test Directories

#### Memory (`memory/`)
- **memory_test.rs**: Memory allocation, deallocation, OOM handling, garbage collection
- **unit_memory_test.rs**: Unit tests for memory operations
- **address_recycling_test.rs**: Address recycling and exhaustion prevention

#### Process (`process/`)
- **process_test.rs**: Process lifecycle, creation, termination, memory integration
- **executor_test.rs**: OS-level process spawning and management

#### IPC (`ipc/`)
- **ipc_test.rs**: Basic inter-process communication and message queuing
- **unit_ipc_test.rs**: Unit tests for IPC operations
- **pipe_test.rs**: Unix-style pipe IPC tests
- **queue_test.rs**: Message queue tests (FIFO, Priority, PubSub)
- **shm_test.rs**: Shared memory tests
- **ipc_id_recycling_test.rs**: IPC resource ID recycling

#### Syscalls (`syscalls/`)
- **syscall_test.rs**: System call execution, sandboxing, permission checks
- **unit_syscall_test.rs**: Unit tests for syscall operations
- **syscalls_integration_test.rs**: Syscall integration tests
- **async_syscall_test.rs**: Async syscall handler tests
- **async_task_test.rs**: Async task management tests
- **batch_test.rs**: Batch execution tests (parallel and sequential)
- **streaming_test.rs**: Streaming file operations

#### Security (`security/`)
- **sandbox_test.rs**: Sandbox isolation and capability tests
- **permissions_test.rs**: Permission management integration tests
- **namespace_test.rs**: Network namespace isolation tests
- **limits_test.rs**: Resource limit enforcement tests
- **ebpf_test.rs**: eBPF manager integration tests
- **ebpf_events_test.rs**: eBPF event system tests
- **ebpf_filters_test.rs**: eBPF filter management tests
- **ebpf_loader_test.rs**: eBPF program loader tests
- **ebpf_providers_test.rs**: Linux and macOS eBPF provider tests

#### Scheduler (`scheduler/`)
- **scheduler_test.rs**: Process scheduling policies (Round Robin, Priority, CFS)

#### Signals (`signals/`)
- **signals_test.rs**: Signal handling, RT signals, callbacks, priority queuing (87+ tests)

#### VFS (`vfs/`)
- **vfs_test.rs**: Virtual filesystem, mount management, permissions
- **memfs_test.rs**: In-memory filesystem unit tests

#### IO_uring (`iouring/`)
- **iouring_syscall_test.rs**: io_uring-style async syscall completion
- **iouring_ops_test.rs**: io_uring operations tests
- **zerocopy_test.rs**: Zero-copy buffer management

#### Performance (`performance/`)
- **simd_test.rs**: SIMD optimizations and operations
- **jit_test.rs**: JIT compilation and hot path detection
- **dashmap_stress_test.rs**: Concurrent stress tests for DashMap-based managers
- **ahash_test.rs**: Hash function tests

#### gRPC (`grpc/`)
- **grpc_advanced_test.rs**: gRPC streaming, async, and batch endpoints

#### Integration (`integration/`)
- **integration_tests.rs**: End-to-end integration tests
- **integration_process_test.rs**: Process integration with executor and limits

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

