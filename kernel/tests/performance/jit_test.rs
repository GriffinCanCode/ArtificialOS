/*!
 * JIT Compilation Tests
 * Tests for hot path detection and JIT compilation
 */

use ai_os_kernel::security::SandboxManager;
use ai_os_kernel::syscalls::jit::{JitManager, SyscallPattern};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutorWithIpc};
use std::sync::Arc;

#[tokio::test]
async fn test_jit_hotpath_detection() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);
    let syscall = Syscall::GetProcessList;

    // Record the same syscall many times to trigger hot path detection
    for _ in 0..150 {
        jit.record_syscall(1, &syscall);
    }

    // Should be detected as hot
    assert!(jit.should_use_jit(1, &syscall));
}

#[tokio::test]
async fn test_jit_compilation_candidates() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);

    // Record various syscalls with high frequency
    for _ in 0..200 {
        jit.record_syscall(1, &Syscall::GetProcessList);
    }
    for _ in 0..150 {
        jit.record_syscall(1, &Syscall::GetProcessInfo { target_pid: 1 });
    }

    let candidates = jit.get_compilation_candidates();

    // Should identify hot patterns
    assert!(!candidates.is_empty());
    assert!(candidates.contains(&SyscallPattern::from_syscall(&Syscall::GetProcessList)));
}

#[tokio::test]
async fn test_jit_compilation() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);
    let pattern = SyscallPattern::from_syscall(&Syscall::GetProcessList);

    // Compile the pattern
    let result = jit.compile_hotpath(pattern.clone());
    assert!(result.is_ok());

    // Stats should reflect compilation
    let stats = jit.stats();
    assert_eq!(stats.compiled_paths, 1);
}

#[tokio::test]
async fn test_jit_cache_hit() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);
    let syscall = Syscall::GetProcessList;
    let pattern = SyscallPattern::from_syscall(&syscall);

    // Compile the pattern
    jit.compile_hotpath(pattern.clone()).unwrap();

    // Mark as hot
    for _ in 0..150 {
        jit.record_syscall(1, &syscall);
    }

    // Try to execute using JIT
    let result = jit.try_execute_jit(1, &syscall);
    assert!(result.is_some());

    // Stats should show hit
    let stats = jit.stats();
    assert!(stats.jit_hits > 0);
}

#[tokio::test]
async fn test_jit_cache_miss() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);
    let syscall = Syscall::GetProcessList;

    // Try to execute without compilation
    let result = jit.try_execute_jit(1, &syscall);
    assert!(result.is_none());

    // Stats should show miss
    let stats = jit.stats();
    assert!(stats.jit_misses > 0);
}

#[tokio::test]
async fn test_jit_multiple_patterns() {
    let executor = Arc::new(SyscallExecutorWithIpc::new(SandboxManager::new()));
    let jit = JitManager::new(executor);

    // Compile multiple patterns
    let patterns = vec![
        SyscallPattern::from_syscall(&Syscall::GetProcessList),
        SyscallPattern::from_syscall(&Syscall::GetProcessInfo { target_pid: 1 }),
    ];

    for pattern in &patterns {
        jit.compile_hotpath(pattern.clone()).unwrap();
    }

    let stats = jit.stats();
    assert_eq!(stats.compiled_paths, 2);
}

#[test]
fn test_syscall_pattern_extraction() {
    let syscall = Syscall::GetProcessList;
    let pattern = SyscallPattern::from_syscall(&syscall);

    match pattern {
        SyscallPattern::Simple(_) => {} // Expected
        _ => panic!("Expected Simple pattern"),
    }
}

#[test]
fn test_syscall_pattern_file_ops() {
    let syscall = Syscall::ReadFile {
        path: "/test".into(),
    };
    let pattern = SyscallPattern::from_syscall(&syscall);

    match pattern {
        SyscallPattern::FileOp(_) => {} // Expected
        _ => panic!("Expected FileOp pattern"),
    }
}

#[test]
fn test_syscall_pattern_ipc_ops() {
    let syscall = Syscall::CreatePipe {
        reader_pid: 1,
        writer_pid: 2,
        capacity: None,
    };
    let pattern = SyscallPattern::from_syscall(&syscall);

    match pattern {
        SyscallPattern::IpcOp(_) => {} // Expected
        _ => panic!("Expected IpcOp pattern"),
    }
}
