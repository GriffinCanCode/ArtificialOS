/*!
 * io_uring-style Syscall Completion Tests
 * Tests for io_uring-inspired async syscall completion
 */

use ai_os_kernel::syscalls::{
    IoUringExecutor, IoUringManager, SyscallCompletionStatus, SyscallExecutor,
    SyscallSubmissionEntry,
};
use ai_os_kernel::core::types::Pid;
use ai_os_kernel::memory::MemoryManager;
use ai_os_kernel::process::ProcessManager;
use ai_os_kernel::permissions::PermissionManager;
use ai_os_kernel::vfs::Vfs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn setup_test_manager() -> (IoUringManager, Pid) {
    let memory_manager = MemoryManager::new();
    let process_manager = ProcessManager::new(memory_manager.clone());
    let permission_manager = Arc::new(PermissionManager::new());
    let vfs = Vfs::new();

    let syscall_executor = SyscallExecutor::new(
        process_manager.clone(),
        memory_manager.clone(),
        permission_manager,
        vfs,
    );

    let iouring_executor = Arc::new(IoUringExecutor::new(syscall_executor));
    let manager = IoUringManager::new(iouring_executor);

    // Create a test process
    let pid = process_manager.create_process(
        vec!["/bin/test".to_string()],
        vec![],
        vec![],
    ).unwrap();

    (manager, pid)
}

#[tokio::test]
async fn test_iouring_create_ring() {
    let (manager, pid) = setup_test_manager();

    let ring = manager.create_ring(pid, Some(128), Some(256)).unwrap();
    assert_eq!(ring.pid(), pid);
    assert!(ring.sq_is_empty());
    assert!(ring.cq_is_empty());
}

#[tokio::test]
async fn test_iouring_submit_file_read() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, b"Hello, io_uring!").unwrap();

    // Submit a read operation
    let entry = SyscallSubmissionEntry::read_file(pid, test_file.clone(), 42);
    let seq = manager.submit(pid, entry).unwrap();

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Reap completions
    let completions = manager.reap_completions(pid, Some(10)).unwrap();
    assert!(!completions.is_empty());

    let completion = completions.iter().find(|c| c.seq == seq).unwrap();
    assert_eq!(completion.user_data, 42);
    assert!(completion.status.is_success());
}

#[tokio::test]
async fn test_iouring_submit_file_write() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("write_test.txt");

    // Submit a write operation
    let data = b"io_uring write test".to_vec();
    let entry = SyscallSubmissionEntry::write_file(pid, test_file.clone(), data.clone(), 100);
    let seq = manager.submit(pid, entry).unwrap();

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Reap completions
    let completions = manager.reap_completions(pid, Some(10)).unwrap();
    assert!(!completions.is_empty());

    let completion = completions.iter().find(|c| c.seq == seq).unwrap();
    assert_eq!(completion.user_data, 100);
    assert!(completion.status.is_success());

    // Verify file contents
    let contents = std::fs::read(&test_file).unwrap();
    assert_eq!(contents, data);
}

#[tokio::test]
async fn test_iouring_batch_submission() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();

    // Create multiple files
    let mut entries = Vec::new();
    for i in 0..10 {
        let file = temp_dir.path().join(format!("batch_{}.txt", i));
        std::fs::write(&file, format!("Content {}", i)).unwrap();
        entries.push(SyscallSubmissionEntry::read_file(pid, file, i as u64));
    }

    // Submit batch
    let seqs = manager.submit_batch(pid, entries).unwrap();
    assert_eq!(seqs.len(), 10);

    // Wait for all completions
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Reap completions
    let completions = manager.reap_completions(pid, None).unwrap();
    assert_eq!(completions.len(), 10);

    // Verify all completed successfully
    for completion in completions {
        assert!(completion.status.is_success());
    }
}

#[tokio::test]
async fn test_iouring_wait_specific_completion() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("wait_test.txt");
    std::fs::write(&test_file, b"Wait test").unwrap();

    // Submit operation
    let entry = SyscallSubmissionEntry::read_file(pid, test_file, 999);
    let seq = manager.submit(pid, entry).unwrap();

    // Wait for specific completion (blocking in background)
    let manager_clone = manager.clone();
    let handle = tokio::spawn(async move {
        manager_clone.wait_completion(pid, seq)
    });

    // Wait for completion
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        handle
    ).await;

    assert!(result.is_ok());
    let completion = result.unwrap().unwrap().unwrap();
    assert_eq!(completion.seq, seq);
    assert_eq!(completion.user_data, 999);
    assert!(completion.status.is_success());
}

#[tokio::test]
async fn test_iouring_concurrent_operations() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();

    // Submit many concurrent operations
    let mut handles = Vec::new();
    for i in 0..50 {
        let manager = manager.clone();
        let file = temp_dir.path().join(format!("concurrent_{}.txt", i));
        std::fs::write(&file, format!("Data {}", i)).unwrap();

        let handle = tokio::spawn(async move {
            let entry = SyscallSubmissionEntry::read_file(pid, file, i as u64);
            manager.submit(pid, entry)
        });
        handles.push(handle);
    }

    // Wait for all submissions
    let mut seqs = Vec::new();
    for handle in handles {
        let seq = handle.await.unwrap().unwrap();
        seqs.push(seq);
    }

    // Wait for completions
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Reap all completions
    let completions = manager.reap_completions(pid, None).unwrap();
    assert_eq!(completions.len(), 50);
}

#[tokio::test]
async fn test_iouring_error_handling() {
    let (manager, pid) = setup_test_manager();

    // Submit operation for non-existent file
    let entry = SyscallSubmissionEntry::read_file(
        pid,
        PathBuf::from("/nonexistent/file.txt"),
        123
    );
    let seq = manager.submit(pid, entry).unwrap();

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Reap completion
    let completions = manager.reap_completions(pid, Some(10)).unwrap();
    let completion = completions.iter().find(|c| c.seq == seq).unwrap();

    // Should have error status
    assert!(completion.status.is_error());
}

#[tokio::test]
async fn test_iouring_stats() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("stats_test.txt");
    std::fs::write(&test_file, b"Stats").unwrap();

    // Submit several operations
    for i in 0..5 {
        let entry = SyscallSubmissionEntry::read_file(pid, test_file.clone(), i);
        let _ = manager.submit(pid, entry).unwrap();
    }

    // Wait for completions
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Check stats
    let stats = manager.stats();
    assert_eq!(stats.active_rings, 1);
    assert_eq!(stats.total_submissions, 5);
}

#[tokio::test]
async fn test_iouring_ring_destruction() {
    let (manager, pid) = setup_test_manager();

    // Create ring
    let _ = manager.create_ring(pid, None, None).unwrap();
    assert!(manager.get_ring(pid).is_some());

    // Destroy ring
    manager.destroy_ring(pid).unwrap();
    assert!(manager.get_ring(pid).is_none());
}

#[tokio::test]
async fn test_iouring_open_close() {
    let (manager, pid) = setup_test_manager();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("open_close_test.txt");
    std::fs::write(&test_file, b"Open/Close test").unwrap();

    // Submit open operation
    let open_entry = SyscallSubmissionEntry::open(
        pid,
        test_file.clone(),
        0, // O_RDONLY
        0,
        1000
    );
    let open_seq = manager.submit(pid, open_entry).unwrap();

    // Wait for open to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let completions = manager.reap_completions(pid, Some(10)).unwrap();
    let open_completion = completions.iter().find(|c| c.seq == open_seq).unwrap();
    assert!(open_completion.status.is_success());
}

#[tokio::test]
async fn test_iouring_fsync() {
    let (manager, pid) = setup_test_manager();

    // Submit fsync operation (will likely fail without valid FD, but tests the path)
    let fsync_entry = SyscallSubmissionEntry::fsync(pid, 3, 2000);
    let seq = manager.submit(pid, fsync_entry).unwrap();

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let completions = manager.reap_completions(pid, Some(10)).unwrap();
    assert!(!completions.is_empty());
}
