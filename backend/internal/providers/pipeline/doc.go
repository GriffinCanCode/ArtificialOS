// Package pipeline provides a multi-process ETL data pipeline demonstration.
//
// This provider implements a complete data pipeline with 4 stages that demonstrates
// all IPC mechanisms in the kernel: pipes, shared memory, and pub-sub queues.
//
// Pipeline Architecture:
//
//	Stage 1: Data Ingestion (reader process)
//	   ↓ PIPE (streaming, 64KB buffer)
//	Stage 2: Data Transformation (processor process)
//	   ↓ SHARED MEMORY (zero-copy, 1MB segment)
//	Stage 3: Data Aggregation (aggregator process)
//	   ↓ PUBSUB QUEUE (async broadcast)
//	Stage 4: Output Writers (3 subscriber processes)
//
// IPC Mechanisms Demonstrated:
//
//  1. PIPES (Stage 1 → 2):
//     - Unidirectional streaming
//     - Buffered flow control
//     - Blocking semantics
//
//  2. SHARED MEMORY (Stage 2 → 3):
//     - Zero-copy data transfer
//     - Large data blocks
//     - Memory-efficient
//
//  3. PUBSUB QUEUES (Stage 3 → Writers):
//     - Broadcast to multiple subscribers
//     - Async message delivery
//     - Fan-out pattern
//
// Features:
//   - Real-time metrics (throughput, latency, memory usage)
//   - Process lifecycle management
//   - Automatic cleanup on stop
//   - Comprehensive logging
//
// Example Usage:
//
//	// Start the pipeline
//	pipeline.start()
//	// → Creates 7 processes (4 stages + 3 writers)
//	// → Creates pipe, shared memory, and queue
//	// → Starts data flow simulation
//
//	// Monitor metrics
//	// → Throughput: 50 records/sec
//	// → Stage latencies: 5ms, 12ms, 8ms
//	// → Memory usage: Pipe 64KB, SHM 1MB, Queue 128KB
//
//	// Stop the pipeline
//	pipeline.stop()
//	// → Terminates all processes
//	// → Destroys all IPC resources
//	// → Cleans up memory
//
// Tools:
//   - pipeline.init: Initialize pipeline state
//   - pipeline.start: Start pipeline with IPC setup
//   - pipeline.stop: Stop pipeline and cleanup
//   - pipeline.clear_logs: Clear log history
//
// Why This Matters:
//
// This is a real-world use case that demonstrates:
//   - Multi-process coordination
//   - Different IPC mechanisms for different needs
//   - Performance benefits (shared memory vs copying)
//   - Pub-sub pattern for fan-out architectures
//
// Unlike single-process apps, this pipeline shows the value of:
//   - Pipes for streaming (Stage 1 → 2)
//   - Shared memory for bulk transfer (Stage 2 → 3)
//   - Queues for async broadcast (Stage 3 → Writers)
package pipeline
