/*!
 * Core Module
 *
 * Fundamental kernel types, error handling, and performance primitives.
 *
 * # Module Organization
 *
 * - **errors**, **types**, **traits**: Core abstractions and type system
 * - **limits**: System-wide limits and constants
 * - **guard**: RAII resource guards with type-state pattern
 * - **sync**: Synchronization primitives (locks, wait queues, RCU)
 * - **memory**: Memory utilities (arena, CoW, pooling)
 * - **serialization**: Bincode and JSON with SIMD optimization
 * - **data_structures**: Specialized data structures (inline strings, epoch FD table)
 * - **optimization**: Low-level performance hints (prefetch, branch prediction)
 * - **simd**: SIMD-accelerated operations (memory, search, math, text)
 */

// Core abstractions
pub mod errors;
pub mod guard;
pub mod limits;
pub mod traits;
pub mod types;

// Modules
pub mod clipboard;
pub mod data_structures;
pub mod memory;
pub mod optimization;
pub mod serialization;
pub mod simd;
pub mod sync;

// Re-export core abstractions
pub use errors::*;
pub use limits::*;
pub use traits::*;
pub use types::*;

// Re-export guard types
pub use guard::{
    AsyncTaskGuard, CompositeGuard, FdGuard, Guard, GuardDrop, GuardError, GuardRef, GuardResult,
    IpcGuard, IpcResourceType, LockGuard, LockState, Locked, MemoryGuard, Observable,
    ObservableGuard, Operation, Recoverable, SyscallGuard, TimeoutPolicy, TransactionGuard,
    TypedGuard, TypedState, Unlocked,
};

// Re-export sync primitives
pub use sync::{
    AdaptiveLock, CondvarWait, FlatCombiningCounter, FutexWait, RcuCell, SeqlockStats,
    ShardManager, SpinWait, StrategyType, StripedMap, SyncConfig, WaitError, WaitQueue, WaitResult,
    WaitStrategy, WakeResult, WorkloadProfile,
};

// Re-export memory utilities
pub use memory::{
    with_arena, ArenaString, ArenaVec, CowMemory, CowMemoryManager, CowStats, PooledBuffer,
    SharedPool,
};

// Re-export serialization utilities
pub use serialization::{
    from_bincode, from_json, is_zero_u32, is_zero_u64, is_zero_usize, serialized_size,
    skip_serializing_none, system_time_micros, to_bincode, to_json, to_json_string,
};

// Re-export data structures
pub use data_structures::{EpochFdTable, InlineString};

// Re-export optimization utilities
pub use optimization::{
    find_hash_simd, likely, path_starts_with_any, prefetch_read, prefetch_write, unlikely,
    PrefetchExt,
};

// Re-export SIMD operations
pub use simd::{
    ascii_to_lower, ascii_to_upper, avg_u64, capabilities as simd_capabilities, contains_byte,
    count_byte, detect_simd_support, find_byte, init_simd, is_ascii, max_u64, min_u64, rfind_byte,
    simd_memcmp, simd_memcpy, simd_memmove, simd_memset, sum_u32, sum_u64, trim, trim_end,
    trim_start, SimdCapabilities,
};

// Re-export CPU hints wildcard (barrier, spin_loop, etc.)
pub use optimization::*;

// Re-export clipboard
pub use clipboard::{
    ClipboardData, ClipboardEntry, ClipboardError, ClipboardFormat, ClipboardManager,
    ClipboardResult, ClipboardStats, ClipboardSubscription,
};
