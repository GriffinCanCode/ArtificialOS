/*!
 * Search Syscall Tests
 * Tests for file and content search functionality
 */

use ai_os_kernel::syscalls::core::executor::SyscallExecutorWithIpc;
use ai_os_kernel::syscalls::types::{SearchResult, Syscall, SyscallResult};
use ai_os_kernel::vfs::{LocalFS, MountManager};
use ai_os_kernel::security::SandboxManager;
use ai_os_kernel::ipc::{PipeManager, ShmManager};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_executor() -> (SyscallExecutorWithIpc, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    // Create sandbox manager
    let sandbox_manager = SandboxManager::new();

    // Create test sandbox
    let pid = 1;
    sandbox_manager.create_sandbox(pid, None).unwrap();

    // Create IPC managers
    let pipe_manager = PipeManager::new();
    let shm_manager = ShmManager::new();

    // Create executor with VFS
    let executor = SyscallExecutorWithIpc::with_ipc_direct(
        sandbox_manager,
        pipe_manager,
        shm_manager,
    );

    // Setup VFS
    let vfs = MountManager::new();
    let local_fs = Arc::new(LocalFS::new(temp_dir.path()));
    vfs.mount("/", local_fs).unwrap();

    let executor = executor.with_vfs(Arc::new(vfs));

    (executor, temp_dir)
}

#[test]
fn test_search_files_basic() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create test files
    std::fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();
    std::fs::write(temp_dir.path().join("example.rs"), "rust code").unwrap();
    std::fs::write(temp_dir.path().join("readme.md"), "documentation").unwrap();

    // Search for "test"
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "test",
        10,
        false,
        false,
        0.3,
    );

    assert!(matches!(result, SyscallResult::Success { .. }));

    // Verify results
    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("test")));
    } else {
        panic!("Expected success result");
    }
}

#[test]
fn test_search_files_fuzzy() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create test files with similar names
    std::fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
    std::fs::write(temp_dir.path().join("configuration.toml"), "[]").unwrap();
    std::fs::write(temp_dir.path().join("settings.yaml"), "---").unwrap();

    // Search with fuzzy matching
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "config",
        10,
        false,
        false,
        0.5,
    );

    assert!(matches!(result, SyscallResult::Success { .. }));

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        // Should match both config.json and configuration.toml
        assert!(results.len() >= 2);
    }
}

#[test]
fn test_search_files_recursive() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create nested directory structure
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();
    std::fs::write(sub_dir.join("nested.txt"), "nested file").unwrap();
    std::fs::write(temp_dir.path().join("root.txt"), "root file").unwrap();

    // Search recursively
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "txt",
        10,
        true, // recursive
        false,
        0.3,
    );

    assert!(matches!(result, SyscallResult::Success { .. }));

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        // Should find both root.txt and nested.txt
        assert!(results.len() >= 2);
    }
}

#[test]
fn test_search_content_basic() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create files with content
    std::fs::write(temp_dir.path().join("file1.txt"), "hello world\ntest line").unwrap();
    std::fs::write(temp_dir.path().join("file2.txt"), "another file\nhello again").unwrap();

    // Search for "hello" in content
    let result = executor.search_content(
        pid,
        &PathBuf::from("/"),
        "hello",
        10,
        false,
        false,
        true,
    );

    assert!(matches!(result, SyscallResult::Success { .. }));

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        assert!(!results.is_empty());
        // Verify content is included
        assert!(results.iter().any(|r| r.content.is_some()));
    }
}

#[test]
fn test_search_files_empty_query() {
    let (executor, _temp_dir) = create_test_executor();
    let pid = 1;

    // Empty query should return error or empty results
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "",
        10,
        false,
        false,
        0.3,
    );

    // Should still succeed but return no results
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_search_files_limit() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create many files
    for i in 0..20 {
        std::fs::write(temp_dir.path().join(format!("test{}.txt", i)), "content").unwrap();
    }

    // Search with limit
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "test",
        5, // limit to 5 results
        false,
        false,
        0.3,
    );

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        assert!(results.len() <= 5);
    }
}

#[test]
fn test_search_files_permission_denied() {
    let (executor, _temp_dir) = create_test_executor();
    let non_existent_pid = 999;

    // Search with non-existent PID (should fail permission check)
    let result = executor.search_files(
        non_existent_pid,
        &PathBuf::from("/"),
        "test",
        10,
        false,
        false,
        0.3,
    );

    assert!(matches!(result, SyscallResult::PermissionDenied { .. }));
}

#[test]
fn test_search_content_case_sensitive() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    std::fs::write(temp_dir.path().join("test.txt"), "Hello WORLD\nhello world").unwrap();

    // Case sensitive search
    let result = executor.search_content(
        pid,
        &PathBuf::from("/"),
        "Hello",
        10,
        false,
        true, // case sensitive
        false,
    );

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        assert_eq!(results.len(), 1); // Should only match "Hello", not "hello"
    }
}

#[test]
fn test_levenshtein_distance() {
    let (executor, temp_dir) = create_test_executor();
    let pid = 1;

    // Create files with varying similarity
    std::fs::write(temp_dir.path().join("test.txt"), "").unwrap();
    std::fs::write(temp_dir.path().join("text.txt"), "").unwrap();
    std::fs::write(temp_dir.path().join("best.txt"), "").unwrap();

    // Search for "test" - should match all with different scores
    let result = executor.search_files(
        pid,
        &PathBuf::from("/"),
        "test",
        10,
        false,
        false,
        0.5,
    );

    if let SyscallResult::Success { data: Some(data) } = result {
        let results: Vec<SearchResult> = serde_json::from_slice(&data).unwrap();
        assert!(results.len() >= 3);

        // Verify scores are ordered (best matches first)
        for i in 1..results.len() {
            assert!(results[i - 1].score <= results[i].score);
        }
    }
}

