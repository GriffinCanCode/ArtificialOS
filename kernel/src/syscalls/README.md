# Syscalls Module

System call implementation for AgentOS kernel.

## Overview

The syscalls module provides a comprehensive system call interface for processes running within the kernel. It includes sandboxing, capability checking, and resource management.

## Architecture

### Traits (`traits.rs`)

The module defines fine-grained trait interfaces for different categories of syscalls:

- **`FileSystemSyscalls`** - File and directory operations (read, write, create, delete, list, etc.)
- **`ProcessSyscalls`** - Process management (spawn, kill, list, wait, etc.)
- **`IpcSyscalls`** - Inter-process communication (pipes and shared memory)
- **`NetworkSyscalls`** - Network operations (sockets, send, receive, etc.)
- **`FileDescriptorSyscalls`** - Low-level file descriptor operations (open, close, dup, lseek, etc.)
- **`MemorySyscalls`** - Memory management (stats, garbage collection)
- **`SchedulerSyscalls`** - CPU scheduling operations
- **`SignalSyscalls`** - Process signaling
- **`SystemInfoSyscalls`** - System information and environment
- **`TimeSyscalls`** - Time and sleep operations
- **`SyscallExecutorTrait`** - Combined trait for complete syscall execution

### Types (`types.rs`)

Core data structures and types with modern serde patterns:

- **`SyscallError`** - Strongly-typed error enum using `thiserror`
  - Uses `#[serde(tag = "error_type", content = "details")]` for structured errors
  - Includes helper constructors for ergonomic error creation
  - Marked `#[non_exhaustive]` for forward compatibility
  
- **`SyscallResult`** - Result type for syscall operations
  - Uses `#[skip_serializing_none]` from `serde_with` for cleaner JSON
  - Tagged union format for type-safe deserialization
  - Maintains backward compatibility with existing code
  
- **`Syscall`** - Enum of all available system calls
  - Comprehensive documentation on all variants
  - Uses `#[serde_as]` for enhanced serde_with support
  - Optional fields use `#[serde(skip_serializing_if)]` to reduce payload size
  - Default values with `#[serde(default)]` for optional parameters
  - Marked `#[non_exhaustive]` for extensibility
  
- **`ProcessOutput`** - Process execution output structure
  - Skips empty stdout/stderr in serialization
  - Includes helper methods: `is_success()`, `is_empty()`
  
- **`SystemInfo`** - System information structure
  - Includes `current()` helper for runtime introspection

### Implementation

The module is organized into specialized files:

- **`executor.rs`** - Main `SyscallExecutor` struct that implements all traits
- **`fs.rs`** - Filesystem syscall implementations
- **`fd.rs`** - File descriptor syscall implementations
- **`process.rs`** - Process management syscalls
- **`ipc.rs`** - IPC syscalls (pipes and shared memory)
- **`network.rs`** - Network syscalls
- **`memory.rs`** - Memory management syscalls
- **`scheduler.rs`** - Scheduler syscalls
- **`signal.rs`** - Signal syscalls
- **`system.rs`** - System info syscalls
- **`time.rs`** - Time-related syscalls
- **`vfs_adapter.rs`** - VFS integration layer for filesystem operations

## Usage

### Basic Usage

```rust
use agentoskernel::syscalls::{SyscallExecutor, Syscall};
use agentoskernel::security::SandboxManager;

// Create executor with sandbox manager
let sandbox = SandboxManager::new();
let executor = SyscallExecutor::new(sandbox);

// Execute a syscall
let result = executor.execute(
    pid,
    Syscall::ReadFile {
        path: PathBuf::from("/path/to/file")
    }
);
```

### With Full Features

```rust
// Create executor with all features
let executor = SyscallExecutor::with_full_features(
    sandbox_manager,
    pipe_manager,
    shm_manager,
    process_manager,
    memory_manager,
)
.with_vfs(vfs_manager);
```

### Trait-Based Usage

```rust
use agentoskernel::syscalls::{FileSystemSyscalls, ProcessSyscalls};

fn my_function<E>(executor: &E, pid: Pid)
where
    E: FileSystemSyscalls + ProcessSyscalls
{
    // Use specific syscall categories
    executor.read_file(pid, &path);
    executor.spawn_process(pid, "command", &args);
}
```

## Security

All syscalls are protected by:

1. **Capability checks** - Processes must have appropriate capabilities
2. **Path access control** - Filesystem operations are sandboxed
3. **Resource limits** - Operations respect resource quotas
4. **Input validation** - All inputs are sanitized

## VFS Integration

Filesystem syscalls can route through the VFS layer when available, with automatic fallback to standard filesystem operations. This allows for:

- Multiple filesystem types (LocalFS, MemFS, etc.)
- Virtual filesystem overlays
- Custom filesystem implementations
- Transparent operation regardless of underlying storage

## Error Handling

The module uses two error types:

- **`SyscallError`** - Strongly-typed errors for internal use
- **`SyscallResult`** - Enum-based results for external API (backward compatible)

`SyscallError` automatically converts to `SyscallResult` when needed.

## Serde Best Practices

The syscalls module uses modern serde patterns for optimal serialization:

### Attributes Used

1. **`#[skip_serializing_none]`** - Omits `None` values from JSON output
   - Reduces payload size
   - Cleaner JSON representation
   - From `serde_with` crate

2. **`#[serde_as]`** - Enables advanced serde_with conversions
   - Prepares for future type conversions
   - Better PathBuf/Duration handling

3. **`#[serde(default)]`** - Provides default values on deserialization
   - Makes fields optional in JSON
   - Backward compatible with older clients

4. **`#[serde(skip_serializing_if = "Option::is_none")]`** - Conditional serialization
   - Used on optional syscall parameters
   - Reduces JSON payload size

5. **`#[non_exhaustive]`** - Forward compatibility
   - Allows adding variants without breaking changes
   - Recommended for public APIs

### Type Patterns

- **Tagged Unions**: Use `#[serde(tag = "type")]` for type discrimination
- **Flattening**: Avoid unless necessary (breaks type safety)
- **String Validation**: Use custom deserializers for constrained strings
- **Helper Methods**: Add ergonomic constructors and queries

### Example

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MyStruct {
    #[serde(default)]
    optional_field: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<Item>,
}
```

## Extension

To add new syscalls:

1. Add the syscall variant to `Syscall` enum in `types.rs`
2. Add the trait method to appropriate trait in `traits.rs`
3. Implement the handler method in the corresponding implementation file
4. Add the dispatch case in `executor.rs` `execute()` method

## Testing

Unit tests are located in `/tests/unit/` directory:
- `syscalls_test.rs` - Basic syscall tests
- `syscalls_fs_test.rs` - Filesystem syscall tests
- `syscalls_ipc_test.rs` - IPC syscall tests
- `syscalls_process_test.rs` - Process syscall tests
