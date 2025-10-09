/*!
 * Virtual File System Module
 * Pluggable filesystem abstraction layer with observability
 */

pub mod init;
pub mod local;
pub mod memory;
pub mod mount;
pub mod observable;
pub mod observable_wrapper;
pub mod paths;
pub mod traits;
pub mod types;

// Re-exports
pub use init::{init_vfs, sync_native_apps};
pub use local::LocalFS;
pub use memory::MemFS;
pub use mount::{MountManager, MountPoint};
pub use observable::{EventBroadcaster, FileEvent, Observable};
pub use observable_wrapper::ObservableFS;
pub use paths::{app, mounts, storage, user};
pub use traits::{FileSystem, OpenFile};
pub use types::{Entry, FileType, Metadata, OpenFlags, OpenMode, Permissions, VfsError, VfsResult};
