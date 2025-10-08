/*!
 * Memory Utilities
 *
 * High-performance memory management utilities:
 * - Arena allocation for bulk allocations
 * - Copy-on-write memory management
 * - Object pooling for zero-allocation patterns
 *
 * # Performance
 *
 * - Arena: O(1) bulk allocation, single deallocation
 * - CoW: Lazy copying reduces memory bandwidth
 * - Pool: Reuses allocations, reduces allocator pressure
 *
 * # Use Cases
 *
 * - **Arena**: Temporary bulk allocations (parsing, compilation)
 * - **CoW**: Shared read-only data with occasional writes
 * - **Pool**: Frequently allocated/deallocated objects (buffers, connections)
 */

mod arena;
mod cow_memory;
mod pool;

pub use arena::{with_arena, ArenaString, ArenaVec};
pub use cow_memory::{CowMemory, CowMemoryManager, CowStats};
pub use pool::{PooledBuffer, SharedPool};

