/*!
 * MemFS Tests
 * Unit tests for in-memory filesystem
 */

use std::path::Path;

use ai_os_kernel::vfs::traits::FileSystem;
use ai_os_kernel::vfs::types::VfsError;
use ai_os_kernel::vfs::memory::MemFS;

#[test]
fn test_memfs_basic() {
    let fs = MemFS::new();

    // Write and read
    fs.write(Path::new("/test.txt"), b"hello").unwrap();
    let data = fs.read(Path::new("/test.txt")).unwrap();
    assert_eq!(data, b"hello");

    // Exists
    assert!(fs.exists(Path::new("/test.txt")));
    assert!(!fs.exists(Path::new("/missing.txt")));

    // Delete
    fs.delete(Path::new("/test.txt")).unwrap();
    assert!(!fs.exists(Path::new("/test.txt")));
}

#[test]
fn test_memfs_directories() {
    let fs = MemFS::new();

    // Create directory
    fs.create_dir(Path::new("/testdir")).unwrap();
    assert!(fs.exists(Path::new("/testdir")));

    // Create nested
    fs.create_dir(Path::new("/testdir/nested")).unwrap();
    assert!(fs.exists(Path::new("/testdir/nested")));

    // List
    fs.write(Path::new("/testdir/file.txt"), b"content")
        .unwrap();
    let entries = fs.list_dir(Path::new("/testdir")).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_capacity_limit() {
    let fs = MemFS::with_capacity(10);

    // Should succeed
    fs.write(Path::new("/small.txt"), b"hello").unwrap();

    // Should fail - exceeds capacity
    assert!(matches!(
        fs.write(Path::new("/large.txt"), b"hello world"),
        Err(VfsError::OutOfSpace)
    ));
}

#[test]
fn test_path_normalization() {
    let fs = MemFS::new();

    fs.write(Path::new("/test.txt"), b"hello").unwrap();

    // Different path representations should work
    assert!(fs.exists(Path::new("test.txt")));
    assert!(fs.exists(Path::new("/test.txt")));
    assert!(fs.exists(Path::new("//test.txt")));
}
