/*!
 * Zero-Copy IPC Cleanup Test
 * Verifies that zero-copy rings are properly cleaned up on process termination
 */

use ai_os_kernel::ipc::core::manager::IPCManager;
use ai_os_kernel::memory::manager::MemoryManager;

#[test]
fn test_zerocopy_cleanup_via_ipc_manager() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager);

    let pid = 100;

    // Verify zerocopy is available
    let zerocopy = ipc_manager.zerocopy().expect("ZeroCopy should be enabled");

    // Create a zero-copy ring for the process
    let ring_result = zerocopy.create_ring(pid, 4096, 4096);
    assert!(ring_result.is_ok(), "Should create zero-copy ring");

    // Verify ring exists
    assert!(zerocopy.has_process_rings(pid), "Process should have rings");

    // Get initial stats
    let stats_before = zerocopy.stats();
    assert_eq!(stats_before.active_rings, 1, "Should have 1 active ring");
    assert_eq!(stats_before.active_buffer_pools, 1, "Should have 1 buffer pool");

    // Call clear_process_queue - this should clean up zero-copy rings
    let cleaned = ipc_manager.clear_process_queue(pid);
    assert!(cleaned > 0, "Should have cleaned up resources");

    // Verify zero-copy rings are cleaned
    assert!(!zerocopy.has_process_rings(pid), "Process rings should be cleaned up");

    let stats_after = zerocopy.stats();
    assert_eq!(stats_after.active_rings, 0, "Should have 0 active rings after cleanup");
    assert_eq!(stats_after.active_buffer_pools, 0, "Should have 0 buffer pools after cleanup");
}

#[test]
fn test_zerocopy_cleanup_multiple_processes() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager);

    let zerocopy = ipc_manager.zerocopy().expect("ZeroCopy should be enabled");

    // Create rings for multiple processes
    for pid in 100..=105 {
        zerocopy.create_ring(pid, 4096, 4096).expect("Should create ring");
    }

    let stats = zerocopy.stats();
    assert_eq!(stats.active_rings, 6, "Should have 6 active rings");

    // Clean up one process
    ipc_manager.clear_process_queue(100);

    let stats = zerocopy.stats();
    assert_eq!(stats.active_rings, 5, "Should have 5 active rings after cleanup");
    assert!(!zerocopy.has_process_rings(100), "PID 100 should be cleaned");
    assert!(zerocopy.has_process_rings(101), "PID 101 should still exist");

    // Clean up remaining processes
    for pid in 101..=105 {
        ipc_manager.clear_process_queue(pid);
    }

    let stats = zerocopy.stats();
    assert_eq!(stats.active_rings, 0, "All rings should be cleaned");
}

#[test]
fn test_zerocopy_cleanup_with_buffer_pools() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager);

    let zerocopy = ipc_manager.zerocopy().expect("ZeroCopy should be enabled");
    let pid = 200;

    // Create ring and buffer pool
    zerocopy.create_ring(pid, 4096, 4096).expect("Should create ring");

    let buffer_pool = zerocopy.get_buffer_pool(pid).expect("Should have buffer pool");

    // Acquire some buffers
    let _buf1 = buffer_pool.acquire(4096).expect("Should acquire buffer");
    let _buf2 = buffer_pool.acquire(8192).expect("Should acquire buffer");

    // Cleanup should handle both ring and buffer pool
    let cleaned = ipc_manager.clear_process_queue(pid);
    assert!(cleaned > 0, "Should clean up resources");

    // Verify cleanup
    assert!(!zerocopy.has_process_rings(pid), "Should clean up all zerocopy resources");
    assert!(zerocopy.get_ring(pid).is_none(), "Ring should be gone");
    assert!(zerocopy.get_buffer_pool(pid).is_none(), "Buffer pool should be gone");
}

#[test]
fn test_ipc_manager_without_zerocopy() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    // Verify zerocopy is not available
    assert!(ipc_manager.zerocopy().is_none(), "ZeroCopy should not be enabled");

    // clear_process_queue should still work without zerocopy
    let cleaned = ipc_manager.clear_process_queue(100);
    assert_eq!(cleaned, 0, "Should clean 0 resources (none exist)");
}

#[test]
fn test_zerocopy_cleanup_with_mixed_ipc_resources() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager);

    let pid = 300;

    // Create various IPC resources
    // 1. Message queue
    ipc_manager.send_message(pid, pid, b"test message".to_vec())
        .expect("Should send message");

    // 2. Pipe
    let pipe_id = ipc_manager.pipes().create(pid, pid + 1, Some(4096))
        .expect("Should create pipe");

    // 3. Shared memory
    let shm_id = ipc_manager.shm().create(pid, 8192)
        .expect("Should create shared memory");

    // 4. Zero-copy ring
    let zerocopy = ipc_manager.zerocopy().expect("Should have zerocopy");
    zerocopy.create_ring(pid, 4096, 4096).expect("Should create ring");

    // Verify all resources exist
    assert!(ipc_manager.has_messages(pid), "Should have messages");
    assert!(zerocopy.has_process_rings(pid), "Should have zerocopy ring");

    // Clean up all resources for the process
    let cleaned = ipc_manager.clear_process_queue(pid);
    assert!(cleaned >= 3, "Should clean up multiple resources (message, pipe, shm, zerocopy)");

    // Verify all are cleaned
    assert!(!ipc_manager.has_messages(pid), "Messages should be cleaned");
    assert!(!zerocopy.has_process_rings(pid), "Zerocopy ring should be cleaned");
}

#[test]
fn test_zerocopy_memory_cleanup() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager.clone());

    let zerocopy = ipc_manager.zerocopy().expect("Should have zerocopy");
    let pid = 400;

    // Get initial memory usage
    let (_, initial_used, _) = memory_manager.info();

    // Create ring (allocates memory)
    zerocopy.create_ring(pid, 16384, 16384).expect("Should create ring");

    let (_, after_create, _) = memory_manager.info();
    assert!(after_create > initial_used, "Memory usage should increase");

    // Cleanup
    ipc_manager.clear_process_queue(pid);

    // Memory should be freed
    let (_, after_cleanup, _) = memory_manager.info();
    assert!(after_cleanup < after_create, "Memory should be freed after cleanup");
}

#[test]
fn test_zerocopy_cleanup_idempotent() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::with_zerocopy(memory_manager);

    let zerocopy = ipc_manager.zerocopy().expect("Should have zerocopy");
    let pid = 500;

    // Create ring
    zerocopy.create_ring(pid, 4096, 4096).expect("Should create ring");

    // First cleanup
    let cleaned1 = ipc_manager.clear_process_queue(pid);
    assert!(cleaned1 > 0, "Should clean resources");

    // Second cleanup should be safe (idempotent)
    let cleaned2 = ipc_manager.clear_process_queue(pid);
    assert_eq!(cleaned2, 0, "Should clean 0 resources on second call");

    // Verify still no resources
    assert!(!zerocopy.has_process_rings(pid), "Should have no rings");
}
