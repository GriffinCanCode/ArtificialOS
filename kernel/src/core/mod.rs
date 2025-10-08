/*!
 * Core Module
 * Fundamental kernel types and error handling
 */

pub mod bincode;
pub mod const_generics;
pub mod errors;
pub mod guard;
pub mod hints;
pub mod json;
pub mod serde;
pub mod shard_manager;
pub mod sync;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use errors::*;
pub use guard::{
    AsyncTaskGuard, CompositeGuard, FdGuard, Guard, GuardDrop, GuardError, GuardRef, GuardResult,
    IpcGuard, IpcResourceType, LockGuard, LockState, Locked, MemoryGuard, Observable,
    ObservableGuard, Operation, Recoverable, SyscallGuard, TransactionGuard, TypedGuard,
    TypedState, Unlocked,
};
pub use hints::*;
pub use shard_manager::{ShardManager, WorkloadProfile};
pub use traits::*;
pub use types::*;
