/*!
 * Syscall Tests
 * Tests for sandboxed system call execution
 */

use ai_os_kernel::security::{Capability, SandboxConfig, SandboxManager};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_env() -> (SyscallExecutor, SandboxManager, TempDir, u32) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    // Create standard sandbox for test process
    let mut config = SandboxConfig::standard(pid);
    // Canonicalize the path to handle macOS /private prefix
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    (executor, sandbox_manager, temp_dir, pid)
}

#[test]
fn test_file_exists_allowed() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"hello").unwrap();

    let result = executor.execute(pid, Syscall::FileExists { path: test_file });

    match result {
        SyscallResult::Success { data } => {
            assert_eq!(data.unwrap()[0], 1); // File exists
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_file_exists_nonexistent() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 201;

    // Create new sandbox with canonicalized path
    let mut config = SandboxConfig::standard(pid);
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    let test_file = temp_dir.path().join("nonexistent.txt");

    let result = executor.execute(pid, Syscall::FileExists { path: test_file });

    match result {
        SyscallResult::Success { data } => {
            assert_eq!(data.unwrap()[0], 0); // File doesn't exist
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_read_file_allowed() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let test_file = temp_dir.path().join("read_test.txt");
    let content = b"Hello, World!";
    fs::write(&test_file, content).unwrap();

    let result = executor.execute(pid, Syscall::ReadFile { path: test_file });

    match result {
        SyscallResult::Success { data } => {
            assert_eq!(data.unwrap(), content);
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_read_file_blocked_path() {
    let (executor, _, _, pid) = setup_test_env();

    // Try to read a blocked path
    let blocked_path = PathBuf::from("/etc/passwd");

    let result = executor.execute(pid, Syscall::ReadFile { path: blocked_path });

    match result {
        SyscallResult::PermissionDenied { .. } => {
            // Expected
        }
        _ => panic!("Expected permission denied, got: {:?}", result),
    }
}

#[test]
fn test_write_file_allowed() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let test_file = temp_dir.path().join("write_test.txt");
    let content = b"Test content".to_vec();

    let result = executor.execute(
        pid,
        Syscall::WriteFile {
            path: test_file.clone(),
            data: content.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            // Verify file was written
            let read_content = fs::read(&test_file).unwrap();
            assert_eq!(read_content, content);
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_write_file_no_capability() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 200;

    // Create sandbox without WriteFile capability
    let mut config = SandboxConfig::minimal(pid);
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_manager.create_sandbox(config);

    let test_file = temp_dir.path().join("write_test.txt");

    let result = executor.execute(
        pid,
        Syscall::WriteFile {
            path: test_file,
            data: b"test".to_vec(),
        },
    );

    match result {
        SyscallResult::PermissionDenied { .. } => {
            // Expected
        }
        _ => panic!("Expected permission denied, got: {:?}", result),
    }
}

#[test]
fn test_create_file() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 200;

    // Create sandbox with CreateFile capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::CreateFile);
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_manager.create_sandbox(config);

    let test_file = temp_dir.path().join("new_file.txt");

    let result = executor.execute(
        pid,
        Syscall::CreateFile {
            path: test_file.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            assert!(test_file.exists());
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_delete_file() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 200;

    let test_file = temp_dir.path().join("delete_test.txt");
    fs::write(&test_file, b"delete me").unwrap();

    // Create sandbox with DeleteFile capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::DeleteFile);
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(
        pid,
        Syscall::DeleteFile {
            path: test_file.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            assert!(!test_file.exists());
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_list_directory() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 200;

    // Create some test files
    fs::write(temp_dir.path().join("file1.txt"), b"test").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), b"test").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), b"test").unwrap();

    // Create sandbox with ListDirectory capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ListDirectory);
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(
        pid,
        Syscall::ListDirectory {
            path: temp_dir.path().to_path_buf(),
        },
    );

    match result {
        SyscallResult::Success { data } => {
            let files: Vec<String> = serde_json::from_slice(&data.unwrap()).unwrap();
            assert_eq!(files.len(), 3);
            assert!(files.contains(&"file1.txt".to_string()));
            assert!(files.contains(&"file2.txt".to_string()));
            assert!(files.contains(&"file3.txt".to_string()));
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_file_stat() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let test_file = temp_dir.path().join("stat_test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let result = executor.execute(pid, Syscall::FileStat { path: test_file });

    match result {
        SyscallResult::Success { data } => {
            let file_info: serde_json::Value = serde_json::from_slice(&data.unwrap()).unwrap();
            assert_eq!(file_info["name"], "stat_test.txt");
            assert_eq!(file_info["size"], 12);
            assert_eq!(file_info["is_dir"], false);
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_copy_file() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let source = temp_dir.path().join("source.txt");
    let destination = temp_dir.path().join("dest.txt");
    let content = b"copy me";
    fs::write(&source, content).unwrap();

    let result = executor.execute(
        pid,
        Syscall::CopyFile {
            source: source.clone(),
            destination: destination.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            assert!(destination.exists());
            let copied_content = fs::read(&destination).unwrap();
            assert_eq!(copied_content, content);
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_move_file() {
    let (executor, _, temp_dir, pid) = setup_test_env();

    let source = temp_dir.path().join("move_source.txt");
    let destination = temp_dir.path().join("move_dest.txt");
    let content = b"move me";
    fs::write(&source, content).unwrap();

    let result = executor.execute(
        pid,
        Syscall::MoveFile {
            source: source.clone(),
            destination: destination.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            assert!(!source.exists());
            assert!(destination.exists());
            let moved_content = fs::read(&destination).unwrap();
            assert_eq!(moved_content, content);
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_create_directory() {
    let (executor, sandbox_manager, temp_dir, _) = setup_test_env();
    let pid = 200;

    let new_dir = temp_dir.path().join("new_directory");

    // Create sandbox with CreateFile capability (used for directories too)
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::CreateFile);
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(
        pid,
        Syscall::CreateDirectory {
            path: new_dir.clone(),
        },
    );

    match result {
        SyscallResult::Success { .. } => {
            assert!(new_dir.exists());
            assert!(new_dir.is_dir());
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_get_system_info() {
    let (executor, sandbox_manager, _, _) = setup_test_env();
    let pid = 200;

    // Grant SystemInfo capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::SystemInfo);
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(pid, Syscall::GetSystemInfo);

    match result {
        SyscallResult::Success { data } => {
            let info: serde_json::Value = serde_json::from_slice(&data.unwrap()).unwrap();
            assert!(info["os"].is_string());
            assert!(info["arch"].is_string());
            assert!(info["family"].is_string());
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_get_current_time() {
    let (executor, sandbox_manager, _, _) = setup_test_env();
    let pid = 200;

    // Grant TimeAccess capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::TimeAccess);
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(pid, Syscall::GetCurrentTime);

    match result {
        SyscallResult::Success { data } => {
            let timestamp_bytes = data.unwrap();
            assert_eq!(timestamp_bytes.len(), 8); // u64 timestamp
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_spawn_process_blocked() {
    let (executor, _, _, pid) = setup_test_env();

    // Standard sandbox doesn't have SpawnProcess capability
    let result = executor.execute(
        pid,
        Syscall::SpawnProcess {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
        },
    );

    match result {
        SyscallResult::PermissionDenied { .. } => {
            // Expected
        }
        _ => panic!("Expected permission denied, got: {:?}", result),
    }
}

#[test]
fn test_spawn_process_with_capability() {
    let (executor, sandbox_manager, _, _) = setup_test_env();
    let pid = 200;

    // Grant SpawnProcess capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::SpawnProcess);
    sandbox_manager.create_sandbox(config);

    let result = executor.execute(
        pid,
        Syscall::SpawnProcess {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
        },
    );

    match result {
        SyscallResult::Success { data } => {
            let output: serde_json::Value = serde_json::from_slice(&data.unwrap()).unwrap();
            assert!(output["stdout"].as_str().unwrap().contains("hello"));
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_invalid_command_injection_prevention() {
    let (executor, sandbox_manager, _, _) = setup_test_env();
    let pid = 200;

    // Grant SpawnProcess capability
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::SpawnProcess);
    sandbox_manager.create_sandbox(config);

    // Try to inject shell commands
    let result = executor.execute(
        pid,
        Syscall::SpawnProcess {
            command: "echo; cat /etc/passwd".to_string(),
            args: vec![],
        },
    );

    match result {
        SyscallResult::Error { message } => {
            assert!(message.contains("shell metacharacters"));
        }
        _ => panic!("Expected error for shell injection, got: {:?}", result),
    }
}

#[test]
fn test_network_request_no_capability() {
    let (executor, _, _, pid) = setup_test_env();

    let result = executor.execute(
        pid,
        Syscall::NetworkRequest {
            url: "https://example.com".to_string(),
        },
    );

    match result {
        SyscallResult::PermissionDenied { .. } => {
            // Expected
        }
        _ => panic!("Expected permission denied, got: {:?}", result),
    }
}
