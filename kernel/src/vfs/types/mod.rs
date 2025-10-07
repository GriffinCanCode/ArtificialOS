/*!
 * VFS Types
 * Shared types for filesystem operations with modern serde patterns
 */

mod entry;
mod errors;
mod file_type;
mod metadata;
mod open_flags;
mod permissions;

pub use entry::Entry;
pub use errors::{VfsError, VfsResult};
pub use file_type::FileType;
pub use metadata::Metadata;
pub use open_flags::{OpenFlags, OpenMode};
pub use permissions::Permissions;
