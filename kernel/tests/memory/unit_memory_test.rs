/*!
 * Memory Manager Tests
 * Comprehensive tests for memory allocation, deallocation, and OOM handling
 */

use ai_os_kernel::memory::{Allocator, GarbageCollector, MemoryError, MemoryInfo, MemoryManager};
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_memory_manager_initialization() {
    let mem_mgr = MemoryManager::new();
    let (total, used, available) = mem_mgr.info();

    assert_eq!(total, 1024 * 1024 * 1024); // 1GB
    assert_eq!(used, 0);
    assert_eq!(available, total);
}

#[test]
fn test_basic_allocation() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;
    let size = 1024 * 1024; // 1MB

    let result = mem_mgr.allocate(size, pid);
    assert!(result.is_ok());

    let (_, used, _) = mem_mgr.info();
    assert_eq!(used, size);
}

#[test]
fn test_multiple_allocations() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Allocate multiple blocks
    let addr1 = mem_mgr.allocate(1024, pid).unwrap();
    let addr2 = mem_mgr.allocate(2048, pid).unwrap();
    let addr3 = mem_mgr.allocate(4096, pid).unwrap();

    // Addresses should be different
    assert_ne!(addr1, addr2);
    assert_ne!(addr2, addr3);
    assert_ne!(addr1, addr3);

    // Total memory used should be sum of all allocations
    let (_, used, _) = mem_mgr.info();
    assert_eq!(used, 1024 + 2048 + 4096);
}

#[test]
fn test_allocation_and_deallocation() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;
    let size = 1024 * 1024;

    let addr = mem_mgr.allocate(size, pid).unwrap();
    let (_, used_before, _) = mem_mgr.info();
    assert_eq!(used_before, size);

    mem_mgr.deallocate(addr).unwrap();
    let (_, used_after, _) = mem_mgr.info();
    assert_eq!(used_after, 0);
}

#[test]
fn test_out_of_memory() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Try to allocate more than total memory
    let result = mem_mgr.allocate(2 * 1024 * 1024 * 1024, pid); // 2GB
    assert!(result.is_err());

    match result {
        Err(MemoryError::OutOfMemory {
            requested,
            available,
            used,
            total,
        }) => {
            assert_eq!(requested, 2 * 1024 * 1024 * 1024);
            assert_eq!(available, total);
            assert_eq!(used, 0);
            assert_eq!(total, 1024 * 1024 * 1024);
        }
        _ => panic!("Expected OutOfMemory error"),
    }
}

#[test]
fn test_oom_after_partial_allocation() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Allocate 900MB
    mem_mgr.allocate(900 * 1024 * 1024, pid).unwrap();

    // Try to allocate another 200MB (should fail)
    let result = mem_mgr.allocate(200 * 1024 * 1024, pid);
    assert!(result.is_err());

    match result {
        Err(MemoryError::OutOfMemory { .. }) => {
            // Expected
        }
        _ => panic!("Expected OutOfMemory error"),
    }
}

#[test]
fn test_process_memory_cleanup() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Allocate multiple blocks for the process
    mem_mgr.allocate(10 * 1024 * 1024, pid).unwrap();
    mem_mgr.allocate(20 * 1024 * 1024, pid).unwrap();
    mem_mgr.allocate(30 * 1024 * 1024, pid).unwrap();

    let (_, used_before, _) = mem_mgr.info();
    assert_eq!(used_before, 60 * 1024 * 1024);

    // Free all memory for the process
    let freed = mem_mgr.free_process_memory(pid);
    assert_eq!(freed, 60 * 1024 * 1024);

    let (_, used_after, _) = mem_mgr.info();
    assert_eq!(used_after, 0);
}

#[test]
fn test_get_process_memory() {
    let mem_mgr = MemoryManager::new();
    let pid1 = 100;
    let pid2 = 200;

    mem_mgr.allocate(10 * 1024 * 1024, pid1).unwrap();
    mem_mgr.allocate(20 * 1024 * 1024, pid2).unwrap();
    mem_mgr.allocate(5 * 1024 * 1024, pid1).unwrap();

    assert_eq!(mem_mgr.process_memory(pid1), 15 * 1024 * 1024);
    assert_eq!(mem_mgr.process_memory(pid2), 20 * 1024 * 1024);
}

#[test]
fn test_invalid_deallocation() {
    let mem_mgr = MemoryManager::new();

    // Try to deallocate an invalid address
    let result = mem_mgr.deallocate(999999);
    assert!(result.is_err());

    match result {
        Err(MemoryError::InvalidAddress) => {
            // Expected
        }
        _ => panic!("Expected InvalidAddress error"),
    }
}

#[test]
fn test_double_deallocation() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    let addr = mem_mgr.allocate(1024, pid).unwrap();
    mem_mgr.deallocate(addr).unwrap();

    // Second deallocation should fail
    let result = mem_mgr.deallocate(addr);
    assert!(result.is_err());
}

#[test]
fn test_memory_stats() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    mem_mgr.allocate(100 * 1024 * 1024, pid).unwrap(); // 100MB

    let stats = mem_mgr.stats();
    assert_eq!(stats.total_memory, 1024 * 1024 * 1024);
    assert_eq!(stats.used_memory, 100 * 1024 * 1024);
    assert_eq!(stats.available_memory, 924 * 1024 * 1024);
    assert_eq!(stats.allocated_blocks, 1);
    assert!((stats.usage_percentage - 9.765625).abs() < 0.001);
}

#[test]
#[serial]
fn test_garbage_collection() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Allocate and deallocate many blocks
    for _ in 0..10 {
        let addr = mem_mgr.allocate(1024, pid).unwrap();
        mem_mgr.deallocate(addr).unwrap();
    }

    // Force GC
    let removed = mem_mgr.force_collect();
    assert_eq!(removed, 10);

    // After GC, allocated blocks should be 0
    let stats = mem_mgr.stats();
    assert_eq!(stats.allocated_blocks, 0);
    assert_eq!(stats.fragmented_blocks, 0);
}

#[test]
fn test_concurrent_allocations() {
    use std::sync::Arc;
    use std::thread;

    let mem_mgr = Arc::new(MemoryManager::new());
    let mut handles = vec![];

    // Spawn multiple threads allocating memory
    for i in 0..10 {
        let mem_mgr_clone = Arc::clone(&mem_mgr);
        let handle = thread::spawn(move || {
            let pid = 100 + i;
            mem_mgr_clone.allocate(1024 * 1024, pid).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let (_, used, _) = mem_mgr.info();
    assert_eq!(used, 10 * 1024 * 1024);
}

#[test]
fn test_memory_pressure_thresholds() {
    let mem_mgr = MemoryManager::new();
    let pid = 100;

    // Allocate 85% of memory (should trigger warning)
    let size = (1024 * 1024 * 1024 * 85) / 100;
    let result = mem_mgr.allocate(size, pid);
    assert!(result.is_ok());

    let stats = mem_mgr.stats();
    assert!(stats.usage_percentage > 80.0);
}
