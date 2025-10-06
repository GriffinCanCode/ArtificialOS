# Syscall Usage Guide

Quick reference for using all 50 syscalls from the Go backend.

## Usage Pattern

All syscalls are invoked through `KernelClient.ExecuteSyscall()`:

```go
data, err := kernelClient.ExecuteSyscall(ctx, pid, "syscall_name", map[string]interface{}{
    "param1": value1,
    "param2": value2,
})
```

---

## 📁 File System Operations

### read_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "read_file", map[string]interface{}{
    "path": "/storage/myfile.txt",
})
```

### write_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "write_file", map[string]interface{}{
    "path": "/storage/myfile.txt",
    "data": []byte("Hello World"),
})
```

### create_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "create_file", map[string]interface{}{
    "path": "/storage/newfile.txt",
})
```

### delete_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "delete_file", map[string]interface{}{
    "path": "/storage/oldfile.txt",
})
```

### list_directory
```go
data, err := client.ExecuteSyscall(ctx, pid, "list_directory", map[string]interface{}{
    "path": "/storage",
})
```

### file_exists
```go
data, err := client.ExecuteSyscall(ctx, pid, "file_exists", map[string]interface{}{
    "path": "/storage/myfile.txt",
})
```

### file_stat
```go
data, err := client.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{
    "path": "/storage/myfile.txt",
})
```

### move_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "move_file", map[string]interface{}{
    "source":      "/storage/old.txt",
    "destination": "/storage/new.txt",
})
```

### copy_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "copy_file", map[string]interface{}{
    "source":      "/storage/original.txt",
    "destination": "/storage/copy.txt",
})
```

### create_directory
```go
data, err := client.ExecuteSyscall(ctx, pid, "create_directory", map[string]interface{}{
    "path": "/storage/newdir",
})
```

### remove_directory
```go
data, err := client.ExecuteSyscall(ctx, pid, "remove_directory", map[string]interface{}{
    "path": "/storage/olddir",
})
```

### get_working_directory
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_working_directory", map[string]interface{}{})
```

### set_working_directory
```go
data, err := client.ExecuteSyscall(ctx, pid, "set_working_directory", map[string]interface{}{
    "path": "/storage/workspace",
})
```

### truncate_file
```go
data, err := client.ExecuteSyscall(ctx, pid, "truncate_file", map[string]interface{}{
    "path": "/storage/myfile.txt",
    "size": uint64(1024), // truncate to 1KB
})
```

---

## ⚙️ Process Operations

### spawn_process
```go
data, err := client.ExecuteSyscall(ctx, pid, "spawn_process", map[string]interface{}{
    "command": "/bin/ls",
    "args":    []string{"-la", "/storage"},
})
```

### kill_process
```go
data, err := client.ExecuteSyscall(ctx, pid, "kill_process", map[string]interface{}{
    "target_pid": uint32(123),
})
```

### get_process_info
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_process_info", map[string]interface{}{
    "target_pid": uint32(123),
})
```

### get_process_list
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_process_list", map[string]interface{}{})
```

### set_process_priority
```go
data, err := client.ExecuteSyscall(ctx, pid, "set_process_priority", map[string]interface{}{
    "target_pid": uint32(123),
    "priority":   uint32(10),
})
```

### get_process_state
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_process_state", map[string]interface{}{
    "target_pid": uint32(123),
})
```

### get_process_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_process_stats", map[string]interface{}{
    "target_pid": uint32(123),
})
```

### wait_process
```go
// Wait indefinitely
data, err := client.ExecuteSyscall(ctx, pid, "wait_process", map[string]interface{}{
    "target_pid": uint32(123),
})

// Wait with timeout
data, err := client.ExecuteSyscall(ctx, pid, "wait_process", map[string]interface{}{
    "target_pid": uint32(123),
    "timeout_ms": uint64(5000), // 5 seconds
})
```

---

## 🖥️ System Information

### get_system_info
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_system_info", map[string]interface{}{})
```

### get_current_time
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_current_time", map[string]interface{}{})
```

### get_env_var
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_env_var", map[string]interface{}{
    "key": "HOME",
})
```

### set_env_var
```go
data, err := client.ExecuteSyscall(ctx, pid, "set_env_var", map[string]interface{}{
    "key":   "MY_VAR",
    "value": "my_value",
})
```

---

## ⏱️ Time Operations

### sleep
```go
data, err := client.ExecuteSyscall(ctx, pid, "sleep", map[string]interface{}{
    "duration_ms": uint64(1000), // 1 second
})
```

### get_uptime
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_uptime", map[string]interface{}{})
```

---

## 💾 Memory Management

### get_memory_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_memory_stats", map[string]interface{}{})
```

### get_process_memory_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_process_memory_stats", map[string]interface{}{
    "target_pid": uint32(123),
})
```

### trigger_gc
```go
// Trigger GC for all processes
data, err := client.ExecuteSyscall(ctx, pid, "trigger_gc", map[string]interface{}{})

// Trigger GC for specific process
data, err := client.ExecuteSyscall(ctx, pid, "trigger_gc", map[string]interface{}{
    "target_pid": uint32(123),
})
```

---

## 📡 Signal & Network

### send_signal
```go
data, err := client.ExecuteSyscall(ctx, pid, "send_signal", map[string]interface{}{
    "target_pid": uint32(123),
    "signal":     uint32(15), // SIGTERM
})
```

### network_request
```go
data, err := client.ExecuteSyscall(ctx, pid, "network_request", map[string]interface{}{
    "url": "https://api.example.com/data",
})
```

---

## 🔄 IPC - Pipes

### create_pipe
```go
// Create pipe with default capacity
data, err := client.ExecuteSyscall(ctx, pid, "create_pipe", map[string]interface{}{
    "reader_pid": uint32(100),
    "writer_pid": uint32(101),
})

// Create pipe with custom capacity
data, err := client.ExecuteSyscall(ctx, pid, "create_pipe", map[string]interface{}{
    "reader_pid": uint32(100),
    "writer_pid": uint32(101),
    "capacity":   uint32(8192), // 8KB buffer
})
```

### write_pipe
```go
data, err := client.ExecuteSyscall(ctx, pid, "write_pipe", map[string]interface{}{
    "pipe_id": uint32(42),
    "data":    []byte("Hello from writer"),
})
```

### read_pipe
```go
data, err := client.ExecuteSyscall(ctx, pid, "read_pipe", map[string]interface{}{
    "pipe_id": uint32(42),
    "size":    uint32(1024), // read up to 1KB
})
```

### close_pipe
```go
data, err := client.ExecuteSyscall(ctx, pid, "close_pipe", map[string]interface{}{
    "pipe_id": uint32(42),
})
```

### destroy_pipe
```go
data, err := client.ExecuteSyscall(ctx, pid, "destroy_pipe", map[string]interface{}{
    "pipe_id": uint32(42),
})
```

### pipe_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "pipe_stats", map[string]interface{}{
    "pipe_id": uint32(42),
})
```

---

## 🔄 IPC - Shared Memory

### create_shm
```go
data, err := client.ExecuteSyscall(ctx, pid, "create_shm", map[string]interface{}{
    "size": uint32(4096), // 4KB segment
})
```

### attach_shm
```go
// Attach read-write
data, err := client.ExecuteSyscall(ctx, pid, "attach_shm", map[string]interface{}{
    "segment_id": uint32(10),
    "read_only":  false,
})

// Attach read-only
data, err := client.ExecuteSyscall(ctx, pid, "attach_shm", map[string]interface{}{
    "segment_id": uint32(10),
    "read_only":  true,
})
```

### detach_shm
```go
data, err := client.ExecuteSyscall(ctx, pid, "detach_shm", map[string]interface{}{
    "segment_id": uint32(10),
})
```

### write_shm
```go
data, err := client.ExecuteSyscall(ctx, pid, "write_shm", map[string]interface{}{
    "segment_id": uint32(10),
    "offset":     uint32(0),
    "data":       []byte("Shared data"),
})
```

### read_shm
```go
data, err := client.ExecuteSyscall(ctx, pid, "read_shm", map[string]interface{}{
    "segment_id": uint32(10),
    "offset":     uint32(0),
    "size":       uint32(1024), // read 1KB
})
```

### destroy_shm
```go
data, err := client.ExecuteSyscall(ctx, pid, "destroy_shm", map[string]interface{}{
    "segment_id": uint32(10),
})
```

### shm_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "shm_stats", map[string]interface{}{
    "segment_id": uint32(10),
})
```

---

## 🔧 Scheduler Operations

### schedule_next
```go
data, err := client.ExecuteSyscall(ctx, pid, "schedule_next", map[string]interface{}{})
```

### yield_process
```go
data, err := client.ExecuteSyscall(ctx, pid, "yield_process", map[string]interface{}{})
```

### get_current_scheduled
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_current_scheduled", map[string]interface{}{})
```

### get_scheduler_stats
```go
data, err := client.ExecuteSyscall(ctx, pid, "get_scheduler_stats", map[string]interface{}{})
```

---

## Error Handling

All syscalls return `([]byte, error)`. Check for errors:

```go
data, err := client.ExecuteSyscall(ctx, pid, "read_file", params)
if err != nil {
    if strings.Contains(err.Error(), "permission denied") {
        // Handle permission error
    } else if strings.Contains(err.Error(), "not found") {
        // Handle not found error
    } else {
        // Handle other errors
    }
    return err
}

// Parse data based on syscall type
// ...
```

## Response Data

Response data is JSON-encoded. Deserialize based on syscall:

```go
// Example: Reading file
data, err := client.ExecuteSyscall(ctx, pid, "read_file", params)
if err != nil {
    return err
}

// data contains file contents as raw bytes
fileContent := string(data)

// Example: Getting process list
data, err := client.ExecuteSyscall(ctx, pid, "get_process_list", params)
if err != nil {
    return err
}

var processes []ProcessInfo
if err := json.Unmarshal(data, &processes); err != nil {
    return err
}
```

---

## Security Notes

⚠️ **All syscalls are subject to sandbox security checks:**

1. **Capability checks** - Process must have required capabilities
2. **Path restrictions** - Filesystem operations check allowed/blocked paths
3. **Resource limits** - Memory, CPU, file descriptors enforced
4. **Process isolation** - Processes can only affect processes they own (unless privileged)

See `SYSCALL_AUDIT.md` for complete security model documentation.

---

## Performance Tips

1. **Batch operations** - Use IPC for bulk data transfer instead of many small syscalls
2. **Cache results** - System info rarely changes, cache `get_system_info` responses
3. **Async execution** - Use goroutines for parallel syscalls when independent
4. **Shared memory** - For large data sharing between processes, prefer shared memory over pipes

---

## See Also

- [SYSCALL_AUDIT.md](./SYSCALL_AUDIT.md) - Complete syscall audit and architecture
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture overview
- [proto/kernel.proto](../proto/kernel.proto) - Protobuf definitions
