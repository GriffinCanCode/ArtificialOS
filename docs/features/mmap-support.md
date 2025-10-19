# Memory-Mapped Files (mmap) Support

## Overview

The kernel implements memory-mapped files (mmap) for file-backed memory access. Processes can map files into their address space for efficient I/O and inter-process communication through shared memory regions.

## Architecture

### Components

1. **MmapManager (`kernel/src/ipc/utils/mmap.rs`)**
   - Central manager for memory-mapped file regions
   - Integrates with VFS for file access
   - Supports shared and private (copy-on-write) mappings
   - Automatic cleanup on process termination
   - Copy-on-write semantics for private mappings

2. **MmapEntry**
   - Represents a single memory mapping
   - Stores file path, offset, length, protection flags
   - Data backed by Arc<Mutex<CowMemory>> for shared ownership
   - Tracks owner process and mapping type

3. **Syscall Handlers**
   - Mmap: Create a new mapping
   - MmapRead: Read from mapping
   - MmapWrite: Write to mapping
   - Msync: Synchronize to file
   - Munmap: Unmap region
   - MmapStats: Get mapping information

## Features

### Mapping Types

**Shared Mappings (`MapFlags::Shared`)**
- Changes visible to all processes mapping the same file
- Modifications sync back to file on msync() or munmap()
- Useful for IPC through files
- Based on shared Arc backing

**Private Mappings (`MapFlags::Private`)**
- Copy-on-write semantics
- Changes private to the mapping process
- Modifications do NOT sync to file
- Useful for loading file data with CoW

### Protection Flags

**Read (`ProtFlags::PROT_READ`)**
- Allow reading from mapped region

**Write (`ProtFlags::PROT_WRITE`)**
- Allow writing to mapped region

**Execute (`ProtFlags::PROT_EXEC`)**
- Allow executing code from mapped region

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
// Force write-back to file (for shared writable mappings)
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

### Large File Processing

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

### Inter-Process Communication

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

### Configuration File Updates

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

For private mappings, the first write operation triggers a copy via CowMemory:

```rust
// CowMemory handles CoW semantics internally
let mut cow_guard = entry.data.lock();
cow_guard.write(|buf| {
    buf[offset..offset + data.len()].copy_from_slice(data);
});
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

## Current Limitations

1. **Entire File Loaded**: The complete mapped region is loaded into memory (not demand-paged)
2. **Shared CoW Model**: Shared mappings use Arc-based sharing with CoW semantics
3. **No Anonymous Mappings**: Only file-backed mappings are supported
4. **No Page-Level Granularity**: Operations work on byte ranges, not individual pages
5. **No mprotect**: Cannot change protection flags after mapping creation
6. **In-Memory Only**: No persistence beyond manual msync()

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
- Arc-based sharing model

## Security Considerations

1. **Path Traversal**: File paths validated by VFS layer
2. **Permission Checks**: Sandbox capabilities enforced at syscall
3. **Isolation**: Private mappings truly private (separate Arc/CoW)
4. **Automatic Cleanup**: No orphaned mappings after process termination

## Performance

**Advantages:**
- Fast random access to file data
- Reduces syscalls for repeated I/O
- Efficient IPC through shared mappings
- CoW reduces memory overhead for private mappings

**Trade-offs:**
- Initial map loads entire region
- Memory overhead for large mappings
- Copy-on-write has overhead on first write

## References

- POSIX mmap(): IEEE Std 1003.1-2017
- Linux mmap(2): man page
- Copy-on-Write design pattern
