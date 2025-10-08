/*!
 * VFS and FD Manager Integration Tests
 * Verifies unified file system coordination
 */

use ai_os_kernel::core::types::Pid;
use ai_os_kernel::security::{SandboxConfig, SandboxManager, SandboxProvider};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutorWithIpc};
use ai_os_kernel::vfs::{FileSystem, MemFS, MountManager};
use std::path::PathBuf;
use std::sync::Arc;

/// Helper to create a sandbox with a configured PID
fn setup_sandbox(pid: Pid) -> SandboxManager {
    let sandbox = SandboxManager::new();
    let mut config = SandboxConfig::standard(pid);

    // Allow all file operations for tests - standard() already has ReadFile/WriteFile capabilities
    config.allowed_paths.push(PathBuf::from("/mem"));
    config.allowed_paths.push(PathBuf::from("/data"));
    config.allowed_paths.push(PathBuf::from("/data1"));
    config.allowed_paths.push(PathBuf::from("/data2"));

    sandbox.create_sandbox(config);
    sandbox
}

/// Test that VFS-opened files appear in FD table
#[test]
fn test_vfs_file_registered_in_fd_table() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    // Create VFS with MemFS mounted
    let vfs = MountManager::new();
    let mem_fs = Arc::new(MemFS::new());
    vfs.mount("/mem", mem_fs).unwrap();

    // Create test file via VFS
    vfs.write(&PathBuf::from("/mem/test.txt"), b"hello vfs")
        .unwrap();

    // Create executor with VFS
    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open file - should use VFS and register in FD table
    let result = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/mem/test.txt"),
            flags: 0x0001, // O_RDONLY
            mode: 0,
        },
    );

    assert!(result.is_success(), "Open should succeed: {:?}", result);
    let fd = serde_json::from_slice::<serde_json::Value>(result.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;

    println!("Opened FD: {}", fd);
    println!(
        "FD count for PID {}: {}",
        pid,
        executor.fd_manager().get_fd_count(pid)
    );

    // Verify FD is tracked
    assert_eq!(
        executor.fd_manager().get_fd_count(pid),
        1,
        "FD {} should be tracked for PID {}",
        fd,
        pid
    );

    // Close FD
    let close_result = executor.execute(pid, Syscall::Close { fd });
    assert!(close_result.is_success());
    assert_eq!(executor.fd_manager().get_fd_count(pid), 0);
}

/// Test dup/dup2 work with VFS handles
#[test]
fn test_dup_with_vfs_handles() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    // Create VFS with MemFS for fast testing
    let vfs = MountManager::new();
    let mem_fs = Arc::new(MemFS::new());
    vfs.mount("/mem", mem_fs).unwrap();

    // Create test file
    vfs.write(&PathBuf::from("/mem/test.txt"), b"test").unwrap();

    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open file
    let result = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/mem/test.txt"),
            flags: 0x0003, // O_RDWR
            mode: 0,
        },
    );

    assert!(result.is_success());
    let fd1 = serde_json::from_slice::<serde_json::Value>(result.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;

    // Duplicate FD
    let dup_result = executor.execute(pid, Syscall::Dup { fd: fd1 });

    assert!(dup_result.is_success());
    let fd2 = serde_json::from_slice::<serde_json::Value>(dup_result.data().unwrap()).unwrap()
        ["new_fd"]
        .as_u64()
        .unwrap() as u32;

    // Both FDs should be tracked
    assert_eq!(executor.fd_manager().get_fd_count(pid), 2);

    // lseek on fd1
    let lseek_result = executor.execute(
        pid,
        Syscall::Lseek {
            fd: fd1,
            offset: 2,
            whence: 0, // SEEK_SET
        },
    );
    assert!(lseek_result.is_success());

    // Close both
    executor.execute(pid, Syscall::Close { fd: fd1 });
    assert_eq!(executor.fd_manager().get_fd_count(pid), 1);

    executor.execute(pid, Syscall::Close { fd: fd2 });
    assert_eq!(executor.fd_manager().get_fd_count(pid), 0);
}

/// Test multiple filesystem backends
#[test]
fn test_multiple_fs_backends() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    // Create VFS with multiple MemFS instances
    let vfs = MountManager::new();
    let mem_fs1 = Arc::new(MemFS::new());
    let mem_fs2 = Arc::new(MemFS::new());

    vfs.mount("/data1", mem_fs1).unwrap();
    vfs.mount("/data2", mem_fs2).unwrap();

    // Create files in both filesystems
    vfs.write(&PathBuf::from("/data1/file1.txt"), b"data1")
        .unwrap();
    vfs.write(&PathBuf::from("/data2/file2.txt"), b"data2")
        .unwrap();

    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open file from first mount
    let result1 = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/data1/file1.txt"),
            flags: 0x0001, // O_RDONLY
            mode: 0,
        },
    );
    assert!(result1.is_success());

    // Open file from second mount
    let result2 = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/data2/file2.txt"),
            flags: 0x0001, // O_RDONLY
            mode: 0,
        },
    );
    assert!(result2.is_success());

    // Both should be in FD table
    assert_eq!(executor.fd_manager().get_fd_count(pid), 2);

    // Extract FDs
    let fd1 = serde_json::from_slice::<serde_json::Value>(result1.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;
    let fd2 = serde_json::from_slice::<serde_json::Value>(result2.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;

    // Clean up
    executor.execute(pid, Syscall::Close { fd: fd1 });
    executor.execute(pid, Syscall::Close { fd: fd2 });
    assert_eq!(executor.fd_manager().get_fd_count(pid), 0);
}

/// Test process FD cleanup includes VFS files
#[test]
fn test_process_cleanup_includes_vfs_files() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    // Create VFS with MemFS
    let vfs = MountManager::new();
    let mem_fs = Arc::new(MemFS::new());
    vfs.mount("/mem", mem_fs).unwrap();

    // Create test files
    for i in 0..3 {
        let path = format!("/mem/file{}.txt", i);
        vfs.write(&PathBuf::from(&path), b"data").unwrap();
    }

    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open multiple files
    for i in 0..3 {
        let path = format!("/mem/file{}.txt", i);

        let result = executor.execute(
            pid,
            Syscall::Open {
                path: PathBuf::from(path),
                flags: 0x0001,
                mode: 0,
            },
        );
        assert!(result.is_success(), "File {} should open", i);
    }

    assert_eq!(executor.fd_manager().get_fd_count(pid), 3);

    // Cleanup process FDs
    let cleaned = executor.fd_manager().cleanup_process_fds(pid);
    assert_eq!(cleaned, 3);
    assert_eq!(executor.fd_manager().get_fd_count(pid), 0);
}

/// Test lseek works with VFS handles
#[test]
fn test_lseek_with_vfs() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    let vfs = MountManager::new();
    let mem_fs = Arc::new(MemFS::new());
    vfs.mount("/data", mem_fs).unwrap();

    vfs.write(&PathBuf::from("/data/test.txt"), b"0123456789")
        .unwrap();

    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open file
    let result = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/data/test.txt"),
            flags: 0x0001, // O_RDONLY
            mode: 0,
        },
    );

    assert!(result.is_success());
    let fd = serde_json::from_slice::<serde_json::Value>(result.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;

    // Test lseek to position 5
    let lseek_result = executor.execute(
        pid,
        Syscall::Lseek {
            fd,
            offset: 5,
            whence: 0, // SEEK_SET
        },
    );
    assert!(lseek_result.is_success());

    let offset = serde_json::from_slice::<serde_json::Value>(lseek_result.data().unwrap()).unwrap()
        ["offset"]
        .as_u64()
        .unwrap();
    assert_eq!(offset, 5);

    // Close
    executor.execute(pid, Syscall::Close { fd });
}

/// Test fcntl works with VFS handles
#[test]
fn test_fcntl_with_vfs() {
    let pid: Pid = 1;
    let sandbox = setup_sandbox(pid);

    let vfs = MountManager::new();
    let mem_fs = Arc::new(MemFS::new());
    vfs.mount("/data", mem_fs).unwrap();

    vfs.write(&PathBuf::from("/data/test.txt"), b"test")
        .unwrap();

    let executor = SyscallExecutorWithIpc::new(sandbox).with_vfs(vfs);

    // Open file
    let result = executor.execute(
        pid,
        Syscall::Open {
            path: PathBuf::from("/data/test.txt"),
            flags: 0x0001,
            mode: 0,
        },
    );

    let fd = serde_json::from_slice::<serde_json::Value>(result.data().unwrap()).unwrap()["fd"]
        .as_u64()
        .unwrap() as u32;

    // Test fcntl
    let fcntl_result = executor.execute(
        pid,
        Syscall::Fcntl {
            fd,
            cmd: 1, // F_GETFD
            arg: 0,
        },
    );
    assert!(fcntl_result.is_success());

    // Close
    executor.execute(pid, Syscall::Close { fd });
}
