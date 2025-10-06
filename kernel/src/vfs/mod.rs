/*!
 * Virtual File System Module
 * Pluggable filesystem abstraction layer
 */

pub mod local;
pub mod memory;
pub mod mount;
pub mod traits;
pub mod types;

// Re-exports
pub use local::LocalFS;
pub use memory::MemFS;
pub use mount::{MountManager, MountPoint};
pub use traits::{FileSystem, OpenFile};
pub use types::{Entry, FileType, Metadata, OpenFlags, OpenMode, Permissions, VfsError, VfsResult};
