/*!
 * VFS Tests
 * Comprehensive tests for virtual filesystem
 */

use ai_os_kernel::vfs::{FileSystem, LocalFS, MemFS, MountManager};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_memfs_basic_operations() {
    let fs = MemFS::new();

    // Write and read
    fs.write(Path::new("/test.txt"), b"hello world").unwrap();
    let data = fs.read(Path::new("/test.txt")).unwrap();
    assert_eq!(data, b"hello world");

    // Exists
    assert!(fs.exists(Path::new("/test.txt")));
    assert!(!fs.exists(Path::new("/missing.txt")));

    // Metadata
    let metadata = fs.metadata(Path::new("/test.txt")).unwrap();
    assert!(metadata.is_file());
    assert_eq!(metadata.size, 11);

    // Delete
    fs.delete(Path::new("/test.txt")).unwrap();
    assert!(!fs.exists(Path::new("/test.txt")));
}

#[test]
fn test_memfs_directories() {
    let fs = MemFS::new();

    // Create directory
    fs.create_dir(Path::new("/data")).unwrap();
    assert!(fs.exists(Path::new("/data")));

    // Create nested
    fs.create_dir(Path::new("/data/logs")).unwrap();
    assert!(fs.exists(Path::new("/data/logs")));

    // Write file in directory
    fs.write(Path::new("/data/test.txt"), b"content").unwrap();

    // List directory
    let entries = fs.list_dir(Path::new("/data")).unwrap();
    assert_eq!(entries.len(), 2); // test.txt and logs

    // Remove directory (should fail - not empty)
    assert!(fs.remove_dir(Path::new("/data")).is_err());

    // Remove directory recursively
    fs.remove_dir_all(Path::new("/data")).unwrap();
    assert!(!fs.exists(Path::new("/data")));
}

#[test]
fn test_memfs_capacity_limit() {
    let fs = MemFS::with_capacity(100);

    // Should succeed - within limit
    fs.write(Path::new("/small.txt"), b"hello").unwrap();
    assert_eq!(fs.read(Path::new("/small.txt")).unwrap(), b"hello");

    // Should fail - exceeds capacity
    let large_data = vec![0u8; 200];
    assert!(fs.write(Path::new("/large.txt"), &large_data).is_err());
}

#[test]
fn test_memfs_copy_and_rename() {
    let fs = MemFS::new();

    // Create file
    fs.write(Path::new("/source.txt"), b"data").unwrap();

    // Copy
    fs.copy(Path::new("/source.txt"), Path::new("/dest.txt")).unwrap();
    assert_eq!(fs.read(Path::new("/source.txt")).unwrap(), b"data");
    assert_eq!(fs.read(Path::new("/dest.txt")).unwrap(), b"data");

    // Rename
    fs.rename(Path::new("/dest.txt"), Path::new("/renamed.txt")).unwrap();
    assert!(!fs.exists(Path::new("/dest.txt")));
    assert!(fs.exists(Path::new("/renamed.txt")));
}

#[test]
fn test_localfs_basic_operations() {
    let temp = TempDir::new().unwrap();
    let fs = LocalFS::new(temp.path());

    // Write and read
    fs.write(Path::new("test.txt"), b"local data").unwrap();
    let data = fs.read(Path::new("test.txt")).unwrap();
    assert_eq!(data, b"local data");

    // Exists
    assert!(fs.exists(Path::new("test.txt")));
    assert!(!fs.exists(Path::new("missing.txt")));

    // Metadata
    let metadata = fs.metadata(Path::new("test.txt")).unwrap();
    assert!(metadata.is_file());

    // Delete
    fs.delete(Path::new("test.txt")).unwrap();
    assert!(!fs.exists(Path::new("test.txt")));
}

#[test]
fn test_localfs_readonly() {
    let temp = TempDir::new().unwrap();

    // Create file with writable fs
    let fs_write = LocalFS::new(temp.path());
    fs_write.write(Path::new("test.txt"), b"data").unwrap();

    // Try to write with readonly fs
    let fs_readonly = LocalFS::readonly(temp.path());
    assert!(fs_readonly.readonly());

    // Read should work
    let data = fs_readonly.read(Path::new("test.txt")).unwrap();
    assert_eq!(data, b"data");

    // Write should fail
    assert!(fs_readonly.write(Path::new("test2.txt"), b"more data").is_err());
}

#[test]
fn test_mount_manager_basic() {
    let mgr = MountManager::new();

    let mem_fs = Arc::new(MemFS::new());
    mgr.mount("/tmp", mem_fs).unwrap();

    // Write through manager
    mgr.write(Path::new("/tmp/file.txt"), b"content").unwrap();

    // Read through manager
    let data = mgr.read(Path::new("/tmp/file.txt")).unwrap();
    assert_eq!(data, b"content");

    // Check exists
    assert!(mgr.exists(Path::new("/tmp/file.txt")));

    // Unmount
    mgr.unmount("/tmp").unwrap();
    assert!(!mgr.is_mounted("/tmp"));
}

#[test]
fn test_mount_manager_multiple_mounts() {
    let mgr = MountManager::new();

    let fs1 = Arc::new(MemFS::new());
    let fs2 = Arc::new(MemFS::new());

    mgr.mount("/data", fs1).unwrap();
    mgr.mount("/tmp", fs2).unwrap();

    // Write to different filesystems
    mgr.write(Path::new("/data/file1.txt"), b"data1").unwrap();
    mgr.write(Path::new("/tmp/file2.txt"), b"data2").unwrap();

    // Verify isolation
    assert_eq!(mgr.read(Path::new("/data/file1.txt")).unwrap(), b"data1");
    assert_eq!(mgr.read(Path::new("/tmp/file2.txt")).unwrap(), b"data2");
    assert!(!mgr.exists(Path::new("/data/file2.txt")));
    assert!(!mgr.exists(Path::new("/tmp/file1.txt")));
}

#[test]
fn test_mount_manager_nested_mounts() {
    let mgr = MountManager::new();

    let root_fs = Arc::new(MemFS::new());
    let special_fs = Arc::new(MemFS::new());

    mgr.mount("/", root_fs).unwrap();
    mgr.mount("/special", special_fs).unwrap();

    // Write to nested mount
    mgr.write(Path::new("/special/file.txt"), b"special").unwrap();
    mgr.write(Path::new("/normal.txt"), b"normal").unwrap();

    // Verify correct filesystem resolution
    assert_eq!(mgr.read(Path::new("/special/file.txt")).unwrap(), b"special");
    assert_eq!(mgr.read(Path::new("/normal.txt")).unwrap(), b"normal");

    // Special file should not exist in root fs
    let weird_path = Path::new("/normal.txt").join("special");
    assert!(!mgr.exists(&weird_path));
}

#[test]
fn test_mount_manager_cross_filesystem_copy() {
    let mgr = MountManager::new();

    let fs1 = Arc::new(MemFS::new());
    let fs2 = Arc::new(MemFS::new());

    mgr.mount("/src", fs1).unwrap();
    mgr.mount("/dst", fs2).unwrap();

    // Write to source
    mgr.write(Path::new("/src/file.txt"), b"content").unwrap();

    // Copy across filesystems
    mgr.copy(Path::new("/src/file.txt"), Path::new("/dst/file.txt")).unwrap();

    // Verify both exist
    assert_eq!(mgr.read(Path::new("/src/file.txt")).unwrap(), b"content");
    assert_eq!(mgr.read(Path::new("/dst/file.txt")).unwrap(), b"content");
}

#[test]
fn test_mount_manager_cross_filesystem_rename() {
    let mgr = MountManager::new();

    let fs1 = Arc::new(MemFS::new());
    let fs2 = Arc::new(MemFS::new());

    mgr.mount("/src", fs1).unwrap();
    mgr.mount("/dst", fs2).unwrap();

    // Write to source
    mgr.write(Path::new("/src/file.txt"), b"content").unwrap();

    // Rename across filesystems (should copy + delete)
    mgr.rename(Path::new("/src/file.txt"), Path::new("/dst/file.txt")).unwrap();

    // Source should not exist
    assert!(!mgr.exists(Path::new("/src/file.txt")));
    // Destination should exist
    assert_eq!(mgr.read(Path::new("/dst/file.txt")).unwrap(), b"content");
}

#[test]
fn test_mount_manager_list_mounts() {
    let mgr = MountManager::new();

    mgr.mount("/data", Arc::new(MemFS::new())).unwrap();
    mgr.mount("/tmp", Arc::new(MemFS::new())).unwrap();

    let mounts = mgr.list_mounts();
    assert_eq!(mounts.len(), 2);

    let paths: Vec<_> = mounts.iter().map(|(p, _)| p.to_str().unwrap()).collect();
    assert!(paths.contains(&"/data"));
    assert!(paths.contains(&"/tmp"));
}

#[test]
fn test_memfs_truncate() {
    let fs = MemFS::new();

    // Create file
    fs.write(Path::new("/file.txt"), b"hello world").unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 11);

    // Truncate to smaller size
    fs.truncate(Path::new("/file.txt"), 5).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 5);
    assert_eq!(fs.read(Path::new("/file.txt")).unwrap(), b"hello");

    // Truncate to larger size (zero-filled)
    fs.truncate(Path::new("/file.txt"), 10).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 10);
}

#[test]
fn test_memfs_permissions() {
    use ai_os_kernel::vfs::Permissions;

    let fs = MemFS::new();

    // Create file
    fs.write(Path::new("/file.txt"), b"data").unwrap();

    // Set readonly
    let mut perms = Permissions::readwrite();
    perms.set_readonly(true);
    fs.set_permissions(Path::new("/file.txt"), perms).unwrap();

    // Verify permissions
    let metadata = fs.metadata(Path::new("/file.txt")).unwrap();
    assert!(metadata.permissions.is_readonly());
}

#[test]
fn test_localfs_directories() {
    let temp = TempDir::new().unwrap();
    let fs = LocalFS::new(temp.path());

    // Create directory
    fs.create_dir(Path::new("testdir")).unwrap();
    assert!(fs.exists(Path::new("testdir")));

    // List empty directory
    let entries = fs.list_dir(Path::new("testdir")).unwrap();
    assert_eq!(entries.len(), 0);

    // Create file in directory
    fs.write(Path::new("testdir/file.txt"), b"content").unwrap();

    // List directory
    let entries = fs.list_dir(Path::new("testdir")).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "file.txt");

    // Remove directory
    fs.remove_dir_all(Path::new("testdir")).unwrap();
    assert!(!fs.exists(Path::new("testdir")));
}

#[test]
fn test_integration_local_and_memory() {
    let temp = TempDir::new().unwrap();
    let mgr = MountManager::new();

    // Mount local and memory filesystems
    mgr.mount("/local", Arc::new(LocalFS::new(temp.path()))).unwrap();
    mgr.mount("/memory", Arc::new(MemFS::new())).unwrap();

    // Write to both
    mgr.write(Path::new("/local/file.txt"), b"local data").unwrap();
    mgr.write(Path::new("/memory/file.txt"), b"memory data").unwrap();

    // Verify isolation
    assert_eq!(mgr.read(Path::new("/local/file.txt")).unwrap(), b"local data");
    assert_eq!(mgr.read(Path::new("/memory/file.txt")).unwrap(), b"memory data");

    // Copy from memory to local
    mgr.copy(Path::new("/memory/file.txt"), Path::new("/local/copied.txt")).unwrap();

    // Verify copied file exists on disk
    assert_eq!(mgr.read(Path::new("/local/copied.txt")).unwrap(), b"memory data");
}

#[test]
fn test_memfs_append() {
    let fs = MemFS::new();

    // Create file
    fs.write(Path::new("/file.txt"), b"hello").unwrap();

    // Append
    fs.append(Path::new("/file.txt"), b" world").unwrap();

    // Verify
    assert_eq!(fs.read(Path::new("/file.txt")).unwrap(), b"hello world");
}

#[test]
fn test_mount_duplicate_error() {
    let mgr = MountManager::new();

    mgr.mount("/data", Arc::new(MemFS::new())).unwrap();

    // Try to mount again at same path
    let result = mgr.mount("/data", Arc::new(MemFS::new()));
    assert!(result.is_err());
}

#[test]
fn test_unmount_nonexistent_error() {
    let mgr = MountManager::new();

    let result = mgr.unmount("/nonexistent");
    assert!(result.is_err());
}
