/*!
 * Core Module
 * Fundamental kernel types and error handling
 */

pub mod bincode;
pub mod const_generics;
pub mod errors;
pub mod flat_combining;
pub mod guard;
pub mod hints;
pub mod inline_string;
pub mod json;
pub mod limits;
pub mod rcu;
pub mod serde;
pub mod seqlock_stats;
pub mod shard_manager;
pub mod sync;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use errors::*;
pub use flat_combining::FlatCombiningCounter;
pub use guard::{
    AsyncTaskGuard, CompositeGuard, FdGuard, Guard, GuardDrop, GuardError, GuardRef, GuardResult,
    IpcGuard, IpcResourceType, LockGuard, LockState, Locked, MemoryGuard, Observable,
    ObservableGuard, Operation, Recoverable, SyscallGuard, TransactionGuard, TypedGuard,
    TypedState, Unlocked,
};
pub use hints::*;
pub use inline_string::InlineString;
pub use rcu::RcuCell;
pub use seqlock_stats::SeqlockStats;
pub use shard_manager::{ShardManager, WorkloadProfile};
pub use traits::*;
pub use types::*;
