# Memory-Mapped Files (mmap) Support

## Overview

AgentOS kernel now supports memory-mapped files (mmap), providing file-backed shared memory functionality similar to POSIX mmap(). This enables efficient file I/O and inter-process communication through shared memory regions.

## Architecture

### Components

1. **MmapManager (`kernel/src/ipc/mmap.rs`)**
   - Central manager for memory-mapped file regions
   - Integrates with VFS for file access
   - Supports both shared and private (copy-on-write) mappings
   - Automatic synchronization on unmap

2. **Syscalls (`kernel/src/syscalls/mmap.rs`)**
   - Syscall handlers for mmap operations
   - Permission checking and validation
   - JSON serialization of results

3. **IPC Module Integration**
   - Mmap is part of the IPC subsystem alongside pipes, queues, and shared memory
   - Consistent API and patterns

## Features

### Mapping Types

**Shared Mappings (`MapFlags::Shared`)**
- Changes visible to all processes mapping the same file
- Modifications sync back to file on msync() or munmap()
- Useful for IPC through files

**Private Mappings (`MapFlags::Private`)**
- Copy-on-write semantics
- Changes private to the mapping process
- Modifications do NOT sync to file
- Useful for loading executables

### Protection Flags

**Read (`ProtFlags::PROT_READ`)**
- Allow reading from mapped region

**Write (`ProtFlags::PROT_WRITE`)**
- Allow writing to mapped region

**Execute (`ProtFlags::PROT_EXEC`)**
- Allow executing code from mapped region
- Useful for loading shared libraries

**Combined Protection**
```rust
ProtFlags::read_write()  // Read + Write
ProtFlags::all()         // Read + Write + Execute
```

## API

### Creating a Mapping

```rust
let mmap_manager = MmapManager::with_vfs(vfs);

let mmap_id = mmap_manager.mmap(
    pid,
    "/data/myfile.dat".to_string(),
    0,              // offset in file
    4096,           // length to map
    ProtFlags::read_write(),
    MapFlags::Shared,
)?;
```

### Reading from Mapping

```rust
let data = mmap_manager.read(
    pid,
    mmap_id,
    0,      // offset in mapping
    1024,   // length to read
)?;
```

### Writing to Mapping

```rust
let data = b"Hello, mmap!";
mmap_manager.write(
    pid,
    mmap_id,
    0,      // offset in mapping
    data,
)?;
```

### Synchronizing to File

```rust
// Force write-back to file
mmap_manager.msync(pid, mmap_id)?;
```

### Unmapping

```rust
// Automatically syncs shared writable mappings
mmap_manager.munmap(pid, mmap_id)?;
```

## Syscalls

### Mmap - Create Mapping

**Request:**
```json
{
  "Mmap": {
    "path": "/data/myfile.dat",
    "offset": 0,
    "length": 4096,
    "prot": 3,    // READ (1) | WRITE (2) = 3
    "shared": true
  }
}
```

**Response:**
```json
{
  "status": "success",
  "data": [0, 0, 0, 1]  // mmap_id serialized
}
```

**Protection Flags (bitwise OR):**
- `0x01` (1) = PROT_READ
- `0x02` (2) = PROT_WRITE
- `0x04` (4) = PROT_EXEC

### MmapRead - Read from Mapping

**Request:**
```json
{
  "MmapRead": {
    "mmap_id": 1,
    "offset": 0,
    "length": 1024
  }
}
```

**Response:**
```json
{
  "status": "success",
  "data": [72, 101, 108, 108, 111, ...]  // file contents
}
```

### MmapWrite - Write to Mapping

**Request:**
```json
{
  "MmapWrite": {
    "mmap_id": 1,
    "offset": 0,
    "data": [72, 101, 108, 108, 111]
  }
}
```

**Response:**
```json
{
  "status": "success"
}
```

### Msync - Synchronize to File

**Request:**
```json
{
  "Msync": {
    "mmap_id": 1
  }
}
```

**Response:**
```json
{
  "status": "success"
}
```

### Munmap - Unmap Region

**Request:**
```json
{
  "Munmap": {
    "mmap_id": 1
  }
}
```

**Response:**
```json
{
  "status": "success"
}
```

### MmapStats - Get Mapping Info

**Request:**
```json
{
  "MmapStats": {
    "mmap_id": 1
  }
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 1,
    "path": "/data/myfile.dat",
    "offset": 0,
    "length": 4096,
    "prot": {
      "read": true,
      "write": true,
      "exec": false
    },
    "flags": "Shared",
    "owner_pid": 42
  }
}
```

## Use Cases

### 1. Large File Processing

```rust
// Map large file into memory for random access
let mmap_id = mmap_manager.mmap(
    pid,
    "/data/large_dataset.bin".to_string(),
    0,
    file_size,
    ProtFlags::PROT_READ,
    MapFlags::Private,
)?;

// Read specific sections efficiently
let chunk = mmap_manager.read(pid, mmap_id, offset, chunk_size)?;
```

### 2. Inter-Process Communication

```rust
// Process A creates shared mapping
let mmap_id_a = mmap_manager.mmap(
    pid_a,
    "/tmp/shared.dat".to_string(),
    0,
    4096,
    ProtFlags::read_write(),
    MapFlags::Shared,
)?;

// Process B maps same file
let mmap_id_b = mmap_manager.mmap(
    pid_b,
    "/tmp/shared.dat".to_string(),
    0,
    4096,
    ProtFlags::read_write(),
    MapFlags::Shared,
)?;

// Process A writes
mmap_manager.write(pid_a, mmap_id_a, 0, b"Hello")?;
mmap_manager.msync(pid_a, mmap_id_a)?;

// Process B reads
let data = mmap_manager.read(pid_b, mmap_id_b, 0, 5)?;
// data == b"Hello"
```

### 3. Configuration File Updates

```rust
// Map config file
let mmap_id = mmap_manager.mmap(
    pid,
    "/config/app.conf".to_string(),
    0,
    config_size,
    ProtFlags::read_write(),
    MapFlags::Shared,
)?;

// Modify in memory
mmap_manager.write(pid, mmap_id, offset, new_config)?;

// Sync to disk
mmap_manager.msync(pid, mmap_id)?;
```

## Implementation Details

### Copy-on-Write (Private Mappings)

For private mappings, the first write operation triggers a copy:

```rust
if entry.flags == MapFlags::Private {
    // Clone the data (copy-on-write)
    let mut new_data = (*entry.data).clone();
    
    // Write to the copy
    new_data[offset..offset + data.len()].copy_from_slice(data);
    entry.data = Arc::new(new_data);
}
```

### Automatic Cleanup

When a process terminates, all its mappings are automatically cleaned up:

```rust
let cleaned = mmap_manager.cleanup_process(pid);
```

Shared writable mappings are synced before cleanup.

## Permissions

**Required Capabilities:**

- **Mmap**: `FileRead` (always), `FileWrite` (if prot includes write)
- **MmapRead**: None (validated at map time)
- **MmapWrite**: None (validated at map time)
- **Msync**: None
- **Munmap**: None
- **MmapStats**: `SystemInfo`

## Limitations & Future Work

### Current Limitations

1. **No True Shared Write**: Shared mappings are read-only after initial mapping due to Arc immutability
2. **No MAP_ANONYMOUS**: Only file-backed mappings supported
3. **No Page-Level Granularity**: Operations work on byte ranges, not pages
4. **No Memory Protection**: Cannot enforce protection flags at OS level
5. **In-Memory Copy**: Entire mapped region loaded into memory (not demand-paged)

### Future Improvements

1. **Demand Paging**: Only load pages as accessed
2. **True Shared Memory**: Use unsafe + locking for mutable shared mappings
3. **Anonymous Mappings**: MAP_ANONYMOUS for malloc-style allocations
4. **Memory Protection**: Integrate with OS memory protection (mprotect)
5. **Page Faults**: Handle page faults for lazy loading
6. **File Locking**: Coordinate with file locking subsystem
7. **Huge Pages**: Support transparent huge pages for large mappings

## Testing

```bash
# Run mmap unit tests
cargo test --lib mmap

# Integration tests
cargo test mmap_integration
```

## Comparison with POSIX mmap

### Similarities
- Protection flags (PROT_READ, PROT_WRITE, PROT_EXEC)
- Shared vs private mappings
- msync() for synchronization
- munmap() for cleanup

### Differences
- Simplified API (no MAP_FIXED, MAP_POPULATE, etc.)
- ID-based instead of pointer-based
- Automatic cleanup on process termination
- In-memory representation (not demand-paged)

## Security Considerations

1. **Path Traversal**: File paths validated by VFS layer
2. **Permission Checks**: Sandbox capabilities enforced
3. **Isolation**: Private mappings truly private (separate Arc)
4. **Automatic Cleanup**: No orphaned mappings

## Performance

**Advantages:**
- Fast random access to file data
- Reduces syscalls for repeated I/O
- Efficient IPC through shared mappings

**Trade-offs:**
- Initial map loads entire region
- Memory overhead for large mappings
- Copy-on-write has overhead on first write

## References

- POSIX mmap(): IEEE Std 1003.1-2017
- Linux mmap(2): man page
- Copy-on-Write: Operating Systems design pattern
