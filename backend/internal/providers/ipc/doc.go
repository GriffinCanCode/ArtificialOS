// Package ipc provides inter-process communication primitives.
//
// This provider exposes kernel IPC facilities (pipes and shared memory) to apps,
// enabling efficient data sharing and streaming between processes.
//
// Features:
//   - Unix-style pipes for streaming data
//   - Shared memory for zero-copy data transfer
//   - Process sandboxing and permission checks
//   - Resource limits and cleanup
//
// Pipes:
//   - Unidirectional byte streams
//   - Buffered (default 64KB capacity)
//   - Blocking semantics (would block on full/empty)
//   - Automatic cleanup on process termination
//
// Shared Memory:
//   - Zero-copy data sharing between processes
//   - Permission-based access (read-only or read-write)
//   - Up to 100MB per segment
//   - Owner-based lifecycle management
//
// Example Usage:
//
//	// Create a pipe between two apps
//	pipeID := ipc.create_pipe(reader_pid: 100, writer_pid: 200)
//
//	// Writer writes data
//	ipc.write_pipe(pipe_id: pipeID, data: "Hello from writer")
//
//	// Reader reads data
//	result := ipc.read_pipe(pipe_id: pipeID, size: 1024)
//
//	// Create shared memory
//	segID := ipc.create_shm(size: 4096)
//
//	// Another process attaches
//	ipc.attach_shm(segment_id: segID, read_only: false)
//
//	// Write to shared memory
//	ipc.write_shm(segment_id: segID, offset: 0, data: "shared data")
//
//	// Read from shared memory
//	result := ipc.read_shm(segment_id: segID, offset: 0, size: 100)
package ipc
