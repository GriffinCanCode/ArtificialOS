/*!
 * Shared Memory Tests
 * Tests for zero-copy shared memory IPC
 */

use ai_os_kernel::shm::{ShmError, ShmManager};
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_basic_shm_create() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();
    assert!(segment_id > 0);
}

#[test]
fn test_shm_write_read() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Write data
    let data = b"Hello shared memory!";
    sm.write(segment_id, owner_pid, 0, data).unwrap();

    // Read data
    let read_data = sm.read(segment_id, owner_pid, 0, data.len()).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn test_shm_attach_detach() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let other_pid = 200;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Other process attaches
    sm.attach(segment_id, other_pid, false).unwrap();

    // Owner writes
    sm.write(segment_id, owner_pid, 0, b"shared data").unwrap();

    // Other process reads
    let data = sm.read(segment_id, other_pid, 0, 11).unwrap();
    assert_eq!(data, b"shared data");

    // Detach
    sm.detach(segment_id, other_pid).unwrap();

    // After detach, access should fail
    let result = sm.read(segment_id, other_pid, 0, 10);
    assert!(matches!(result, Err(ShmError::PermissionDenied(_))));
}

#[test]
fn test_shm_read_only_permission() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let reader_pid = 200;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Attach as read-only
    sm.attach(segment_id, reader_pid, true).unwrap();

    // Owner writes
    sm.write(segment_id, owner_pid, 0, b"test").unwrap();

    // Reader can read
    let data = sm.read(segment_id, reader_pid, 0, 4).unwrap();
    assert_eq!(data, b"test");

    // Reader cannot write
    let result = sm.write(segment_id, reader_pid, 0, b"fail");
    assert!(matches!(result, Err(ShmError::PermissionDenied(_))));
}

#[test]
fn test_shm_read_write_permission() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let writer_pid = 200;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Attach as read-write
    sm.attach(segment_id, writer_pid, false).unwrap();

    // Writer can write
    sm.write(segment_id, writer_pid, 0, b"from writer").unwrap();

    // Owner can read
    let data = sm.read(segment_id, owner_pid, 0, 11).unwrap();
    assert_eq!(data, b"from writer");
}

#[test]
fn test_shm_offset_access() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Write at different offsets
    sm.write(segment_id, owner_pid, 0, b"START").unwrap();
    sm.write(segment_id, owner_pid, 100, b"MIDDLE").unwrap();
    sm.write(segment_id, owner_pid, 4000, b"END").unwrap();

    // Read back
    let start = sm.read(segment_id, owner_pid, 0, 5).unwrap();
    assert_eq!(start, b"START");

    let middle = sm.read(segment_id, owner_pid, 100, 6).unwrap();
    assert_eq!(middle, b"MIDDLE");

    let end = sm.read(segment_id, owner_pid, 4000, 3).unwrap();
    assert_eq!(end, b"END");
}

#[test]
fn test_shm_out_of_bounds() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 1000;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Write beyond bounds
    let result = sm.write(segment_id, owner_pid, 900, &vec![0u8; 200]);
    assert!(matches!(result, Err(ShmError::InvalidRange { .. })));

    // Read beyond bounds
    let result = sm.read(segment_id, owner_pid, 900, 200);
    assert!(matches!(result, Err(ShmError::InvalidRange { .. })));
}

#[test]
fn test_shm_zero_size() {
    let sm = ShmManager::new();

    let owner_pid = 100;

    let result = sm.create(0, owner_pid);
    assert!(matches!(result, Err(ShmError::InvalidSize(_))));
}

#[test]
fn test_shm_size_limit() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 101 * 1024 * 1024; // 101MB (over limit)

    let result = sm.create(size, owner_pid);
    assert!(matches!(result, Err(ShmError::SizeExceeded { .. })));
}

#[test]
fn test_shm_destroy() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let other_pid = 200;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();
    sm.attach(segment_id, other_pid, false).unwrap();

    // Only owner can destroy
    let result = sm.destroy(segment_id, other_pid);
    assert!(matches!(result, Err(ShmError::PermissionDenied(_))));

    // Owner destroys
    sm.destroy(segment_id, owner_pid).unwrap();

    // Operations on destroyed segment should fail
    let result = sm.write(segment_id, owner_pid, 0, b"test");
    assert!(matches!(result, Err(ShmError::NotFound(_))));
}

#[test]
fn test_shm_stats() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let pid2 = 200;
    let pid3 = 300;
    let size = 8192;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Attach processes
    sm.attach(segment_id, pid2, false).unwrap(); // read-write
    sm.attach(segment_id, pid3, true).unwrap();  // read-only

    // Get stats
    let stats = sm.stats(segment_id).unwrap();
    assert_eq!(stats.id, segment_id);
    assert_eq!(stats.size, size);
    assert_eq!(stats.owner_pid, owner_pid);
    assert_eq!(stats.attached_pids.len(), 3); // owner + 2 attached
    assert!(stats.attached_pids.contains(&owner_pid));
    assert!(stats.attached_pids.contains(&pid2));
    assert!(stats.attached_pids.contains(&pid3));
    assert_eq!(stats.read_only_pids.len(), 1);
    assert!(stats.read_only_pids.contains(&pid3));
}

#[test]
fn test_shm_process_cleanup() {
    let sm = ShmManager::new();

    let pid = 100;

    // Create multiple segments owned by this process
    sm.create(1000, pid).unwrap();
    sm.create(2000, pid).unwrap();
    sm.create(3000, pid).unwrap();

    // Also attach to another segment
    let other_seg = sm.create(1000, 200).unwrap();
    sm.attach(other_seg, pid, false).unwrap();

    // Cleanup
    let count = sm.cleanup_process(pid);
    assert_eq!(count, 3); // Should destroy 3 owned segments
}

#[test]
fn test_shm_per_process_limit() {
    let sm = ShmManager::new();

    let owner_pid = 100;

    // Create segments up to the limit (10 per process)
    for _ in 0..10 {
        sm.create(1000, owner_pid).unwrap();
    }

    // Next creation should fail
    let result = sm.create(1000, owner_pid);
    assert!(matches!(result, Err(ShmError::ProcessLimitExceeded(_, _))));
}

#[test]
#[serial]
fn test_shm_global_memory_limit() {
    let sm = ShmManager::new();

    let mut created = 0;

    // Try to create segments that exceed global limit (500MB)
    let size = 10 * 1024 * 1024; // 10MB each
    let max_segments = 500 / 10; // 50 segments

    for i in 0..max_segments + 5 {
        match sm.create(size, 100 + i) {
            Ok(_) => created += 1,
            Err(ShmError::GlobalMemoryExceeded(_, _)) => break,
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    // Should have hit the global limit
    assert!(created >= max_segments - 1 && created <= max_segments + 1);

    // Clean up
    for i in 0..created {
        sm.cleanup_process(100 + i);
    }
}

#[test]
#[serial]
fn test_shm_memory_tracking() {
    let sm = ShmManager::new();

    let initial = sm.get_global_memory_usage();

    let size = 50000;
    let segment_id = sm.create(size, 100).unwrap();

    let after_create = sm.get_global_memory_usage();
    assert_eq!(after_create, initial + size);

    sm.destroy(segment_id, 100).unwrap();

    let after_destroy = sm.get_global_memory_usage();
    assert_eq!(after_destroy, initial);
}

#[test]
fn test_shm_not_found() {
    let sm = ShmManager::new();

    let result = sm.write(999, 100, 0, b"test");
    assert!(matches!(result, Err(ShmError::NotFound(999))));

    let result = sm.read(999, 100, 0, 10);
    assert!(matches!(result, Err(ShmError::NotFound(999))));

    let result = sm.stats(999);
    assert!(matches!(result, Err(ShmError::NotFound(999))));
}

#[test]
fn test_shm_unattached_access() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let other_pid = 200;

    let segment_id = sm.create(4096, owner_pid).unwrap();

    // Other process tries to access without attaching
    let result = sm.read(segment_id, other_pid, 0, 10);
    assert!(matches!(result, Err(ShmError::PermissionDenied(_))));

    let result = sm.write(segment_id, other_pid, 0, b"test");
    assert!(matches!(result, Err(ShmError::PermissionDenied(_))));
}

#[test]
fn test_shm_multiple_readers() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 4096;

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Attach multiple readers
    for pid in 200..210 {
        sm.attach(segment_id, pid, true).unwrap();
    }

    // Owner writes
    sm.write(segment_id, owner_pid, 0, b"broadcast").unwrap();

    // All readers can read
    for pid in 200..210 {
        let data = sm.read(segment_id, pid, 0, 9).unwrap();
        assert_eq!(data, b"broadcast");
    }
}

#[test]
fn test_shm_large_data() {
    let sm = ShmManager::new();

    let owner_pid = 100;
    let size = 1024 * 1024; // 1MB

    let segment_id = sm.create(size, owner_pid).unwrap();

    // Write large data
    let large_data = vec![0xAB; size];
    sm.write(segment_id, owner_pid, 0, &large_data).unwrap();

    // Read it back
    let read_data = sm.read(segment_id, owner_pid, 0, size).unwrap();
    assert_eq!(read_data.len(), size);
    assert_eq!(read_data, large_data);
}

#[test]
fn test_shm_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let sm = Arc::new(ShmManager::new());
    let size = 100000;
    let segment_id = sm.create(size, 100).unwrap();

    // Attach multiple processes
    for pid in 200..210 {
        sm.attach(segment_id, pid, false).unwrap();
    }

    let mut handles = vec![];

    // Each thread writes to its own offset
    for i in 0..10 {
        let sm_clone = Arc::clone(&sm);
        let handle = thread::spawn(move || {
            let pid = 200 + i;
            let offset = (i * 1000) as usize;
            let data = format!("Thread {} data", i);
            sm_clone.write(segment_id, pid, offset, data.as_bytes()).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all writes
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all writes
    for i in 0..10 {
        let offset = (i * 1000) as usize;
        let expected = format!("Thread {} data", i);
        let data = sm.read(segment_id, 100, offset, expected.len()).unwrap();
        assert_eq!(String::from_utf8(data).unwrap(), expected);
    }
}

#[test]
fn test_shm_zero_copy() {
    let sm = ShmManager::new();

    let pid1 = 100;
    let pid2 = 200;
    let size = 10000;

    let segment_id = sm.create(size, pid1).unwrap();
    sm.attach(segment_id, pid2, false).unwrap();

    // Write pattern
    let data = vec![0x42u8; 5000];
    sm.write(segment_id, pid1, 0, &data).unwrap();

    // Another process reads the same data
    let read_data = sm.read(segment_id, pid2, 0, 5000).unwrap();

    // Verify it's the same data (zero-copy means data is shared)
    assert_eq!(read_data, data);

    // Modify in place
    let new_data = vec![0x99u8; 1000];
    sm.write(segment_id, pid2, 2000, &new_data).unwrap();

    // Original owner sees the changes
    let modified = sm.read(segment_id, pid1, 2000, 1000).unwrap();
    assert_eq!(modified, new_data);
}
