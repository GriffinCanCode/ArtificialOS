/*!
 * VFS subsystem tests entry point
 */

#[path = "vfs/vfs_test.rs"]
mod vfs_test;

#[path = "vfs/memfs_test.rs"]
mod memfs_test;

#[path = "vfs/vfs_fd_integration_test.rs"]
mod vfs_fd_integration_test;
