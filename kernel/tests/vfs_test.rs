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
    fs.copy(Path::new("/source.txt"), Path::new("/dest.txt"))
        .unwrap();
    assert_eq!(fs.read(Path::new("/source.txt")).unwrap(), b"data");
    assert_eq!(fs.read(Path::new("/dest.txt")).unwrap(), b"data");

    // Rename
    fs.rename(Path::new("/dest.txt"), Path::new("/renamed.txt"))
        .unwrap();
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
    assert!(fs_readonly
        .write(Path::new("test2.txt"), b"more data")
        .is_err());
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
    mgr.write(Path::new("/special/file.txt"), b"special")
        .unwrap();
    mgr.write(Path::new("/normal.txt"), b"normal").unwrap();

    // Verify correct filesystem resolution
    assert_eq!(
        mgr.read(Path::new("/special/file.txt")).unwrap(),
        b"special"
    );
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
    mgr.copy(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
        .unwrap();

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
    mgr.rename(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
        .unwrap();

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
    mgr.mount("/local", Arc::new(LocalFS::new(temp.path())))
        .unwrap();
    mgr.mount("/memory", Arc::new(MemFS::new())).unwrap();

    // Write to both
    mgr.write(Path::new("/local/file.txt"), b"local data")
        .unwrap();
    mgr.write(Path::new("/memory/file.txt"), b"memory data")
        .unwrap();

    // Verify isolation
    assert_eq!(
        mgr.read(Path::new("/local/file.txt")).unwrap(),
        b"local data"
    );
    assert_eq!(
        mgr.read(Path::new("/memory/file.txt")).unwrap(),
        b"memory data"
    );

    // Copy from memory to local
    mgr.copy(
        Path::new("/memory/file.txt"),
        Path::new("/local/copied.txt"),
    )
    .unwrap();

    // Verify copied file exists on disk
    assert_eq!(
        mgr.read(Path::new("/local/copied.txt")).unwrap(),
        b"memory data"
    );
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

// ============================================
// Security Tests
// ============================================

#[test]
fn test_localfs_path_normalization_prevents_escape() {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let fs = LocalFS::new(temp.path());

    // Create a file outside the root
    let outside_file = temp.path().parent().unwrap().join("outside.txt");
    fs::write(&outside_file, b"secret").unwrap();

    // Try to access file outside root using ..
    let result = fs.read(Path::new("../outside.txt"));
    // Should not be able to read file outside root
    assert!(result.is_err());

    // Try with absolute path traversal
    let result = fs.read(Path::new("/../outside.txt"));
    assert!(result.is_err());

    // Try with multiple ..
    let result = fs.read(Path::new("foo/../../outside.txt"));
    assert!(result.is_err());

    // Cleanup
    let _ = fs::remove_file(&outside_file);
}

#[test]
fn test_localfs_path_normalization_dot_components() {
    let temp = TempDir::new().unwrap();
    let fs = LocalFS::new(temp.path());

    // Create a test file
    fs.write(Path::new("test.txt"), b"data").unwrap();

    // Access with . (current directory) - should work
    let data = fs.read(Path::new("./test.txt")).unwrap();
    assert_eq!(data, b"data");

    // Access with multiple ./ - should work
    let data = fs.read(Path::new("././test.txt")).unwrap();
    assert_eq!(data, b"data");

    // Create nested directory
    fs.create_dir(Path::new("dir")).unwrap();
    fs.write(Path::new("dir/file.txt"), b"nested").unwrap();

    // Access with .. from nested path - should work within root
    fs.write(Path::new("dir/../test2.txt"), b"sibling").unwrap();
    assert!(fs.exists(Path::new("test2.txt")));
}

#[test]
fn test_mount_manager_readonly_enforcement() {
    use ai_os_kernel::vfs::VfsError;

    let mgr = MountManager::new();
    let fs = Arc::new(MemFS::new());

    // Mount filesystem as readonly
    mgr.mount_with_options("/readonly", fs, true).unwrap();

    // Read operations should work
    // First mount a writable fs to create a file
    let writable_fs = Arc::new(MemFS::new());
    writable_fs.write(Path::new("/test.txt"), b"data").unwrap();
    mgr.unmount("/readonly").unwrap();
    mgr.mount_with_options("/readonly", writable_fs, true)
        .unwrap();

    let data = mgr.read(Path::new("/readonly/test.txt")).unwrap();
    assert_eq!(data, b"data");

    // Write operations should fail
    let result = mgr.write(Path::new("/readonly/new.txt"), b"fail");
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Create should fail
    let result = mgr.create(Path::new("/readonly/new.txt"));
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Append should fail
    let result = mgr.append(Path::new("/readonly/test.txt"), b"more");
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Delete should fail
    let result = mgr.delete(Path::new("/readonly/test.txt"));
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Create directory should fail
    let result = mgr.create_dir(Path::new("/readonly/newdir"));
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Truncate should fail
    let result = mgr.truncate(Path::new("/readonly/test.txt"), 0);
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Set permissions should fail
    use ai_os_kernel::vfs::Permissions;
    let result = mgr.set_permissions(Path::new("/readonly/test.txt"), Permissions::readwrite());
    assert!(matches!(result, Err(VfsError::ReadOnly)));
}

#[test]
fn test_mount_manager_readonly_cross_filesystem_copy() {
    use ai_os_kernel::vfs::VfsError;

    let mgr = MountManager::new();

    // Mount writable source and readonly destination
    let src_fs = Arc::new(MemFS::new());
    let dst_fs = Arc::new(MemFS::new());

    mgr.mount("/src", src_fs).unwrap();
    mgr.mount_with_options("/dst", dst_fs, true).unwrap();

    // Write to source
    mgr.write(Path::new("/src/file.txt"), b"data").unwrap();

    // Copy to readonly destination should fail
    let result = mgr.copy(Path::new("/src/file.txt"), Path::new("/dst/file.txt"));
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Rename to readonly destination should fail
    let result = mgr.rename(Path::new("/src/file.txt"), Path::new("/dst/file.txt"));
    assert!(matches!(result, Err(VfsError::ReadOnly)));
}

#[test]
fn test_mount_manager_readonly_with_open_flags() {
    use ai_os_kernel::vfs::{OpenFlags, OpenMode, VfsError};

    let mgr = MountManager::new();
    let fs = Arc::new(MemFS::new());

    // Create a file first
    fs.write(Path::new("/test.txt"), b"data").unwrap();

    mgr.mount_with_options("/readonly", fs, true).unwrap();

    // Opening for read should work
    let result = mgr.open(
        Path::new("/readonly/test.txt"),
        OpenFlags::read_only(),
        OpenMode::default(),
    );
    assert!(result.is_ok());

    // Opening for write should fail
    let result = mgr.open(
        Path::new("/readonly/test.txt"),
        OpenFlags::write_only(),
        OpenMode::default(),
    );
    assert!(matches!(result, Err(VfsError::ReadOnly)));

    // Opening for read-write should fail
    let result = mgr.open(
        Path::new("/readonly/test.txt"),
        OpenFlags::read_write(),
        OpenMode::default(),
    );
    assert!(matches!(result, Err(VfsError::ReadOnly)));
}

#[test]
fn test_memfs_capacity_concurrent_writes() {
    use std::thread;

    let fs = Arc::new(MemFS::with_capacity(1000));
    let mut handles = vec![];

    // Spawn multiple threads trying to write simultaneously
    for i in 0..10 {
        let fs_clone = Arc::clone(&fs);
        let handle = thread::spawn(move || {
            let data = vec![i as u8; 50]; // Each thread writes 50 bytes
            let path = format!("/file{}.txt", i);
            fs_clone.write(Path::new(&path), &data)
        });
        handles.push(handle);
    }

    // Collect results
    let mut successes = 0;
    for handle in handles {
        match handle.join().unwrap() {
            Ok(_) => successes += 1,
            Err(_) => {} // Some may fail due to capacity limits
        }
    }

    // At least some should succeed (50 bytes * 10 = 500 bytes, under 1000 limit)
    assert!(successes > 0);

    // Total written should not exceed capacity
    // Calculate actual size used
    let mut total_size = 0;
    for i in 0..10 {
        let path = format!("/file{}.txt", i);
        if let Ok(metadata) = fs.metadata(Path::new(&path)) {
            total_size += metadata.size;
        }
    }
    assert!(total_size <= 1000, "Total size {} exceeds capacity 1000", total_size);
}

#[test]
fn test_memfs_capacity_append_enforcement() {
    use ai_os_kernel::vfs::VfsError;

    let fs = MemFS::with_capacity(100);

    // Create a file near capacity
    fs.write(Path::new("/file.txt"), &vec![0u8; 90]).unwrap();

    // Append within capacity should work
    fs.append(Path::new("/file.txt"), &vec![1u8; 5]).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 95);

    // Append beyond capacity should fail
    let result = fs.append(Path::new("/file.txt"), &vec![2u8; 10]);
    assert!(matches!(result, Err(VfsError::OutOfSpace)));

    // Size should not have changed
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 95);
}

#[test]
fn test_memfs_capacity_truncate_enforcement() {
    use ai_os_kernel::vfs::VfsError;

    let fs = MemFS::with_capacity(100);

    // Create a small file
    fs.write(Path::new("/file.txt"), &vec![0u8; 20]).unwrap();

    // Truncate to larger size within capacity should work
    fs.truncate(Path::new("/file.txt"), 50).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 50);

    // Truncate to size exceeding capacity should fail
    let result = fs.truncate(Path::new("/file.txt"), 150);
    assert!(matches!(result, Err(VfsError::OutOfSpace)));

    // Size should not have changed
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 50);

    // Truncate to smaller size should work
    fs.truncate(Path::new("/file.txt"), 30).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 30);
}

#[test]
fn test_memfs_capacity_overwrite_existing_file() {
    let fs = MemFS::with_capacity(100);

    // Create initial file
    fs.write(Path::new("/file.txt"), &vec![0u8; 50]).unwrap();

    // Overwrite with smaller file should work
    fs.write(Path::new("/file.txt"), &vec![1u8; 30]).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 30);

    // Overwrite with larger file within capacity should work
    fs.write(Path::new("/file.txt"), &vec![2u8; 90]).unwrap();
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 90);

    // Overwrite with file exceeding capacity should fail
    let result = fs.write(Path::new("/file.txt"), &vec![3u8; 150]);
    assert!(result.is_err());

    // Original file should still exist with old size
    assert_eq!(fs.metadata(Path::new("/file.txt")).unwrap().size, 90);
}

#[test]
fn test_mount_from_config() {
    use ai_os_kernel::vfs::MountPoint;

    let mgr = MountManager::new();
    let fs = Arc::new(MemFS::new());

    // Create readonly mount config
    let config = MountPoint::readonly("/readonly", "test");
    mgr.mount_from_config(&config, fs).unwrap();

    // Verify readonly enforcement
    let result = mgr.write(Path::new("/readonly/test.txt"), b"data");
    assert!(result.is_err());
}
