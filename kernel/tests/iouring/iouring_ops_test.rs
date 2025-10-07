/*!
 * io_uring Operations Unit Tests
 * Tests for close and fsync operations in io_uring executor
 */

use ai_os_kernel::security::SandboxManager;
use ai_os_kernel::syscalls::SyscallExecutor;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_fd_close() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Create a temporary file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "test content").unwrap();

    // Open the file
    let result = executor.open(1, &file_path, 0x0000, 0o644); // O_RDONLY
    assert!(result.is_success());

    // Extract FD from result
    let data = result.data().unwrap();
    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
    let fd = json["fd"].as_u64().unwrap() as u32;

    // Close the FD
    let close_result = executor.close_fd(1, fd);
    assert!(close_result.is_success());

    // Closing again should fail
    let close_again = executor.close_fd(1, fd);
    assert!(close_again.is_error());
}

#[test]
fn test_fd_close_invalid() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Try to close invalid FD
    let result = executor.close_fd(1, 9999);
    assert!(result.is_error());
}

#[test]
fn test_fd_fsync() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Create a temporary file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sync_test.txt");
    File::create(&file_path).unwrap();

    // Open the file for writing
    let result = executor.open(1, &file_path, 0x0002, 0o644); // O_WRONLY
    assert!(result.is_success());

    // Extract FD
    let data = result.data().unwrap();
    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
    let fd = json["fd"].as_u64().unwrap() as u32;

    // Fsync should succeed
    let fsync_result = executor.fsync_fd(1, fd);
    assert!(fsync_result.is_success());

    // Clean up
    executor.close_fd(1, fd);
}

#[test]
fn test_fd_fsync_invalid() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Try to fsync invalid FD
    let result = executor.fsync_fd(1, 9999);
    assert!(result.is_error());
}

#[test]
fn test_fd_fdatasync() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Create a temporary file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("datasync_test.txt");
    File::create(&file_path).unwrap();

    // Open the file for writing
    let result = executor.open(1, &file_path, 0x0002, 0o644); // O_WRONLY
    assert!(result.is_success());

    // Extract FD
    let data = result.data().unwrap();
    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
    let fd = json["fd"].as_u64().unwrap() as u32;

    // Fdatasync should succeed
    let fdatasync_result = executor.fdatasync_fd(1, fd);
    assert!(fdatasync_result.is_success());

    // Clean up
    executor.close_fd(1, fd);
}

#[test]
fn test_open_close_cycle() {
    let sandbox = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox);

    // Create a temporary file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("cycle_test.txt");
    File::create(&file_path).unwrap();

    // Open and close multiple times
    for _ in 0..5 {
        let result = executor.open(1, &file_path, 0x0000, 0o644);
        assert!(result.is_success());

        let data = result.data().unwrap();
        let json: serde_json::Value = serde_json::from_slice(data).unwrap();
        let fd = json["fd"].as_u64().unwrap() as u32;

        let close_result = executor.close_fd(1, fd);
        assert!(close_result.is_success());
    }
}
