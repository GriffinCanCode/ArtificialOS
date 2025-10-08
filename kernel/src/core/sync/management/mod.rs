/*!
 * Synchronization Configuration & Management
 *
 * CPU-topology-aware configuration for concurrent data structures:
 * - Intelligent shard count calculation
 * - Hardware-aware optimization
 * - Workload profiling
 */

mod shard_manager;

// Re-export public API
pub use shard_manager::{ShardManager, WorkloadProfile};
