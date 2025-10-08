/*!
 * System Limits and Constants
 *
 * Centralized location for all system-wide limits, thresholds, and magic numbers.
 * Organized by domain for maintainability and discoverability.
 *
 * ## Design Philosophy
 * - All values include rationale comments explaining WHY they exist
 * - Values are grouped by domain (memory, IPC, process, etc.)
 * - Performance-critical constants are marked with [PERF]
 * - Security-critical constants are marked with [SECURITY]
 * - Linux-compatible values are marked with [LINUX-COMPAT]
 */

use std::time::Duration;

// =============================================================================
// MEMORY LIMITS
// =============================================================================

/// Total simulated memory pool (1GB)
/// Used as default capacity for memory manager
pub const DEFAULT_MEMORY_POOL: usize = 1024 * 1024 * 1024;

/// Small block threshold for segregated free list (4KB)
/// Power-of-2 buckets up to this size for O(1) allocation
/// [PERF] Aligned with common page size
pub const SMALL_BLOCK_MAX: usize = 4 * 1024;

/// Medium block threshold (64KB)
/// Separates small/medium/large allocation strategies
pub const MEDIUM_BLOCK_MAX: usize = 64 * 1024;

/// SIMD operation threshold (64 bytes)
/// Minimum size to use SIMD for memory operations (copy, compare)
/// [PERF] Below this, standard loops are faster due to SIMD overhead
pub const MEMORY_SIMD_THRESHOLD: usize = 64;

/// Garbage collection threshold (100MB)
/// Trigger GC when total allocated memory exceeds this
pub const DEFAULT_GC_THRESHOLD: usize = 100 * 1024 * 1024;

/// Coalescing interval (every 100 deallocations)
/// [PERF] Amortizes O(n log n) sorting cost across deallocations
pub const DEALLOC_COALESCE_INTERVAL: u64 = 100;

// =============================================================================
// PROCESS RESOURCE LIMITS
// =============================================================================

/// Standard process memory limit (1GB)
/// Default for normal-priority processes
pub const STANDARD_PROCESS_MEMORY: usize = 1024 * 1024 * 1024;

/// Restricted process memory limit (256MB)
/// For sandboxed/low-priority processes
pub const RESTRICTED_PROCESS_MEMORY: usize = 256 * 1024 * 1024;

/// High priority process memory (512MB)
pub const HIGH_PRIORITY_MEMORY: usize = 512 * 1024 * 1024;

/// Normal priority process memory (256MB)
pub const NORMAL_PRIORITY_MEMORY: usize = 256 * 1024 * 1024;

/// Low priority process memory (128MB)
pub const LOW_PRIORITY_MEMORY: usize = 128 * 1024 * 1024;

/// Background priority process memory (64MB)
pub const BACKGROUND_PRIORITY_MEMORY: usize = 64 * 1024 * 1024;

/// Idle priority process memory (32MB)
pub const IDLE_PRIORITY_MEMORY: usize = 32 * 1024 * 1024;

/// Standard file descriptor limit per process
pub const STANDARD_MAX_FILE_DESCRIPTORS: usize = 1024;

/// Restricted file descriptor limit
pub const RESTRICTED_MAX_FILE_DESCRIPTORS: usize = 256;

/// Max network connections per process
pub const MAX_NETWORK_CONNECTIONS: u32 = 100;

/// Max memory mappings per process
pub const MAX_MEMORY_MAPPINGS: usize = 100;

/// Max async tasks per process
pub const MAX_ASYNC_TASKS: usize = 100;

/// Max sockets per process
pub const MAX_SOCKETS: usize = 100;

/// High memory usage threshold (100MB)
/// Triggers resource monitoring alerts
pub const HIGH_MEMORY_THRESHOLD: usize = 100 * 1024 * 1024;

/// High file descriptor count threshold
pub const HIGH_FD_THRESHOLD: usize = 100;

// =============================================================================
// IPC LIMITS
// =============================================================================

/// Default pipe buffer capacity (64KB)
/// [LINUX-COMPAT] Matches Linux default pipe buffer
pub const DEFAULT_PIPE_CAPACITY: usize = 65536;

/// Maximum pipe capacity (1MB)
/// Prevents excessive memory use per pipe
pub const MAX_PIPE_CAPACITY: usize = 1024 * 1024;

/// Maximum pipes per process
pub const MAX_PIPES_PER_PROCESS: usize = 100;

/// Global pipe memory limit (50MB)
/// Total memory for all pipes across all processes
pub const GLOBAL_PIPE_MEMORY_LIMIT: usize = 50 * 1024 * 1024;

/// Maximum message queue capacity (10,000 messages)
pub const MAX_QUEUE_CAPACITY: usize = 10_000;

/// Maximum message size (1MB)
/// Applies to both pipes and message queues
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Maximum queues per process
pub const MAX_QUEUES_PER_PROCESS: usize = 100;

/// Global queue memory limit (100MB)
pub const GLOBAL_QUEUE_MEMORY_LIMIT: usize = 100 * 1024 * 1024;

/// Maximum shared memory segment size (100MB)
pub const MAX_SEGMENT_SIZE: usize = 100 * 1024 * 1024;

/// Maximum shared memory segments per process
pub const MAX_SEGMENTS_PER_PROCESS: usize = 10;

/// Global shared memory limit (500MB)
pub const GLOBAL_SHM_MEMORY_LIMIT: usize = 500 * 1024 * 1024;

/// IPC manager queue size (1000 pending operations)
pub const IPC_MANAGER_QUEUE_SIZE: usize = 1000;

/// SIMD batch size for IPC operations (64 operations)
/// [PERF] Reduces atomic operations by 64x
pub const IPC_SIMD_BATCH_SIZE: usize = 64;

// =============================================================================
// PERFORMANCE TUNING
// =============================================================================

/// io_uring submission queue size
/// [PERF] Must be power of 2 for efficient ring buffer
pub const DEFAULT_SQ_SIZE: usize = 256;

/// io_uring completion queue size
/// [PERF] 2x SQ size to prevent overflow during burst
pub const DEFAULT_CQ_SIZE: usize = 512;

/// Process-specific io_uring SQ size (smaller for per-process)
pub const PROCESS_ZEROCOPY_SQ_SIZE: usize = 128;

/// Process-specific io_uring CQ size
pub const PROCESS_ZEROCOPY_CQ_SIZE: usize = 256;

/// io_uring batch size for syscall submission
/// [PERF] Amortizes syscall overhead
pub const IOURING_BATCH_SIZE: usize = 32;

/// Default streaming chunk size (64KB)
/// Balance between throughput and latency
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// JSON SIMD threshold (1KB)
/// [PERF] Use simd_json for payloads >1KB, std::json for smaller
pub const JSON_SIMD_THRESHOLD: usize = 1024;

/// Small buffer size for zero-copy buffer pool (4KB)
pub const SMALL_BUFFER_SIZE: usize = 4096;

/// Medium buffer size for zero-copy buffer pool (64KB)
pub const MEDIUM_BUFFER_SIZE: usize = 65536;

/// Lock-free ring buffer size (64KB)
/// [PERF] Must be power of 2 for fast modulo via bitmask
pub const LOCKFREE_RING_SIZE: usize = 65536;

/// Futex parking slot count (512 slots)
/// [PERF] Higher count reduces contention, must be power of 2
pub const FUTEX_PARKING_SLOTS: usize = 512;

/// Shard count bounds (min/max)
/// [PERF] Auto-calculated based on CPU count, clamped to this range
pub const MIN_SHARD_COUNT: usize = 8;
pub const MAX_SHARD_COUNT: usize = 512;

/// JIT hotpath detection threshold (100 calls)
/// Syscalls executed >100 times are considered "hot"
pub const JIT_HOT_THRESHOLD: u64 = 100;

/// JIT detection window (1000 syscalls)
/// Track hotness over this many recent syscalls
pub const JIT_DETECTION_WINDOW: u64 = 1000;

/// Maximum lock backoff duration (1ms)
/// [PERF] Prevents excessive spinning on contended locks
pub const MAX_LOCK_BACKOFF_MICROS: u64 = 1000;

// =============================================================================
// TIMEOUTS
// =============================================================================

/// Standard IPC operation timeout (10 seconds)
pub const STANDARD_IPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Restricted IPC operation timeout (2 seconds)
pub const RESTRICTED_IPC_TIMEOUT: Duration = Duration::from_secs(2);

/// Relaxed IPC operation timeout (60 seconds)
pub const RELAXED_IPC_TIMEOUT: Duration = Duration::from_secs(60);

/// Standard file I/O timeout (30 seconds)
pub const STANDARD_FILE_IO_TIMEOUT: Duration = Duration::from_secs(30);

/// Standard fsync timeout (60 seconds)
/// Longer because fsync can be slow on spinning disks
pub const STANDARD_FSYNC_TIMEOUT: Duration = Duration::from_secs(60);

/// Standard network timeout (60 seconds)
pub const STANDARD_NETWORK_TIMEOUT: Duration = Duration::from_secs(60);

/// Process wait timeout (5 minutes)
pub const STANDARD_PROCESS_WAIT_TIMEOUT: Duration = Duration::from_secs(300);

/// io_uring completion timeout (30 seconds)
pub const DEFAULT_COMPLETION_TIMEOUT: Duration = Duration::from_secs(30);

/// Async task time-to-live (1 hour)
/// Tasks older than this are considered stale
pub const DEFAULT_TASK_TTL: Duration = Duration::from_secs(3600);

/// Async task cleanup interval (5 minutes)
/// How often to scan for stale tasks
pub const TASK_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// Maximum sleep duration for sys_sleep (1 minute)
/// [SECURITY] Prevents processes from sleeping indefinitely
pub const MAX_SLEEP_DURATION_MS: u64 = 60_000;

/// gRPC client timeout (30 seconds)
pub const GRPC_CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// gRPC keepalive interval (60 seconds)
pub const GRPC_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(60);

/// gRPC keepalive timeout (20 seconds)
pub const GRPC_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(20);

/// Shutdown grace period (10 seconds)
/// Maximum time to wait for graceful shutdown
pub const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(10);

/// VFS slow operation threshold (100ms)
/// Operations taking longer trigger performance logging
pub const VFS_SLOW_OPERATION_THRESHOLD_MS: u64 = 100;

// =============================================================================
// TIMEOUT EXECUTOR TUNING
// =============================================================================

/// Spin retries before yielding (17 iterations)
/// [PERF] ~20ns overhead per iteration
pub const TIMEOUT_SPIN_RETRIES: u32 = 17;

/// Yield retries before sleeping (100 iterations)
/// [PERF] ~100ns overhead per iteration
pub const TIMEOUT_YIELD_RETRIES: u32 = 100;

/// Time check interval during spinning (every 8 iterations)
/// [PERF] Balance between time accuracy and checking overhead
pub const TIMEOUT_TIME_CHECK_INTERVAL: u32 = 8;

/// Microsleep duration for long waits (10 microseconds)
pub const TIMEOUT_MICROSLEEP_DURATION: Duration = Duration::from_micros(10);

// =============================================================================
// MONITORING & OBSERVABILITY
// =============================================================================

/// Event stream ring buffer size (64KB)
/// [PERF] Must be power of 2 for lock-free ring buffer
pub const EVENT_RING_SIZE: usize = 65536;

/// Minimum samples for anomaly detection (100 samples)
/// Statistical anomaly detection needs sufficient baseline
pub const MIN_ANOMALY_SAMPLES: u64 = 100;

/// Adaptive sampling adjustment interval (10,000 events)
/// How often to recalculate sampling rates
pub const SAMPLING_ADJUSTMENT_INTERVAL: u64 = 10000;

/// Target monitoring overhead percentage (2%)
/// Adaptive sampler targets <2% CPU overhead
pub const TARGET_MONITORING_OVERHEAD_PCT: u8 = 2;

/// Initial sampling rate (100%)
/// Start at full sampling, adapt downward if overhead too high
pub const INITIAL_SAMPLING_RATE: u8 = 100;

/// Cleanup anomaly detection threshold (100ms)
/// Resource cleanup taking >100ms is anomalous
pub const CLEANUP_ANOMALY_THRESHOLD_MS: f64 = 100.0;

/// Expected resources freed during cleanup
/// Used for anomaly detection baseline
pub const EXPECTED_RESOURCES_FREED: f64 = 100.0;

// =============================================================================
// SECURITY & AUDIT
// =============================================================================

/// Maximum audit events stored globally (10,000 events)
/// [SECURITY] Prevents audit log from consuming excessive memory
pub const MAX_AUDIT_EVENTS: usize = 10_000;

/// Maximum audit events per process (100 events)
/// [SECURITY] Per-process limit for fine-grained tracking
pub const MAX_AUDIT_EVENTS_PER_PID: usize = 100;

/// Maximum eBPF event history (10,000 events)
/// [SECURITY] For eBPF-based security monitoring
pub const MAX_EBPF_EVENT_HISTORY: usize = 10_000;

/// Maximum pending signals per process (128 signals)
/// [LINUX-COMPAT] Matches typical Linux signal queue depth
pub const MAX_PENDING_SIGNALS: usize = 128;

/// Maximum signal handlers per process (32 handlers)
/// One handler per standard signal type
pub const MAX_SIGNAL_HANDLERS: usize = 32;

// =============================================================================
// CHANNEL CAPACITIES
// =============================================================================

/// gRPC streaming channel capacity (100 messages)
/// Balance between buffering and backpressure
pub const GRPC_STREAM_CHANNEL_CAPACITY: usize = 100;

// =============================================================================
// FILESYSTEM LIMITS
// =============================================================================

/// /tmp filesystem capacity (100MB)
pub const TMP_FILESYSTEM_CAPACITY: usize = 100 * 1024 * 1024;

/// /cache filesystem capacity (50MB)
pub const CACHE_FILESYSTEM_CAPACITY: usize = 50 * 1024 * 1024;

// =============================================================================
// CPU SHARES (Priority System)
// =============================================================================

/// High priority CPU shares (1024)
pub const HIGH_PRIORITY_CPU_SHARES: u32 = 1024;

/// Normal priority CPU shares (512)
pub const NORMAL_PRIORITY_CPU_SHARES: u32 = 512;

/// Low priority CPU shares (256)
pub const LOW_PRIORITY_CPU_SHARES: u32 = 256;

/// Background priority CPU shares (128)
pub const BACKGROUND_PRIORITY_CPU_SHARES: u32 = 128;

/// Standard priority CPU shares (100)
pub const STANDARD_CPU_SHARES: u32 = 100;

/// Background priority max PIDs (100)
pub const BACKGROUND_PRIORITY_MAX_PIDS: u32 = 100;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Convert bytes to megabytes for human-readable output
#[inline]
pub const fn bytes_to_mb(bytes: usize) -> usize {
    bytes / (1024 * 1024)
}

/// Convert megabytes to bytes
#[inline]
pub const fn mb_to_bytes(mb: usize) -> usize {
    mb * 1024 * 1024
}

/// Check if size should use SIMD for memory operations
#[inline]
pub const fn should_use_memory_simd(size: usize) -> bool {
    size >= MEMORY_SIMD_THRESHOLD
}

/// Check if JSON payload should use SIMD parsing
#[inline]
pub const fn should_use_json_simd(size: usize) -> bool {
    size > JSON_SIMD_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_hierarchy() {
        // Ensure memory limits are ordered correctly
        assert!(SMALL_BLOCK_MAX < MEDIUM_BLOCK_MAX);
        assert!(MEDIUM_BLOCK_MAX < DEFAULT_MEMORY_POOL);
    }

    #[test]
    fn test_ipc_limits_consistent() {
        // Ensure per-process limits don't exceed global limits
        assert!(DEFAULT_PIPE_CAPACITY < GLOBAL_PIPE_MEMORY_LIMIT);
        assert!(MAX_MESSAGE_SIZE <= GLOBAL_QUEUE_MEMORY_LIMIT);
    }

    #[test]
    fn test_iouring_sizing() {
        // CQ should be larger than SQ to prevent overflow
        assert!(DEFAULT_CQ_SIZE >= DEFAULT_SQ_SIZE);
        assert!(PROCESS_ZEROCOPY_CQ_SIZE >= PROCESS_ZEROCOPY_SQ_SIZE);
    }

    #[test]
    fn test_timeout_hierarchy() {
        // Restricted timeouts should be shorter than standard
        assert!(RESTRICTED_IPC_TIMEOUT < STANDARD_IPC_TIMEOUT);
        assert!(STANDARD_IPC_TIMEOUT < RELAXED_IPC_TIMEOUT);
    }

    #[test]
    fn test_helper_functions() {
        assert_eq!(bytes_to_mb(1024 * 1024), 1);
        assert_eq!(mb_to_bytes(1), 1024 * 1024);

        assert!(should_use_memory_simd(128));
        assert!(!should_use_memory_simd(32));

        assert!(should_use_json_simd(2048));
        assert!(!should_use_json_simd(512));
    }

    #[test]
    fn test_power_of_two_requirements() {
        // These must be powers of 2 for efficient algorithms
        assert!(LOCKFREE_RING_SIZE.is_power_of_two());
        assert!(FUTEX_PARKING_SLOTS.is_power_of_two());
        assert!(EVENT_RING_SIZE.is_power_of_two());
        assert!(DEFAULT_SQ_SIZE.is_power_of_two());
        assert!(DEFAULT_CQ_SIZE.is_power_of_two());
    }
}
