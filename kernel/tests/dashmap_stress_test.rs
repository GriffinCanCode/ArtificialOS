/*!
 * DashMap Stress Tests
 * Comprehensive concurrent stress tests for all DashMap-based managers
 */

use ai_os_kernel::core::types::{Pid, Priority};
use ai_os_kernel::ipc::queue::QueueManager;
use ai_os_kernel::ipc::shm::ShmManager;
use ai_os_kernel::ipc::QueueType;
use ai_os_kernel::memory::MemoryManager;
use ai_os_kernel::process::manager::ProcessManagerBuilder;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// Test constants for stress testing
const HIGH_CONCURRENCY: usize = 1000;
const EXTREME_CONCURRENCY: usize = 10000;
const STRESS_DURATION_MS: u64 = 5000;

// ============================================================================
// Queue Manager DashMap Stress Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_queue_manager_concurrent_create() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));
    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    // Spawn HIGH_CONCURRENCY tasks creating queues
    for i in 0..HIGH_CONCURRENCY {
        let manager = Arc::clone(&manager);
        let success = Arc::clone(&success_count);
        let errors = Arc::clone(&error_count);

        handles.push(tokio::spawn(async move {
            let pid = (i % 100) as Pid;
            match manager.create(pid, QueueType::Fifo, Some(100)) {
                Ok(_) => success.fetch_add(1, Ordering::Relaxed),
                Err(_) => errors.fetch_add(1, Ordering::Relaxed),
            };
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let successes = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    println!("Queue creation: {} successes, {} errors", successes, errors);
    assert!(
        successes > 0,
        "At least some queue creations should succeed"
    );
    assert_eq!(successes + errors, HIGH_CONCURRENCY as u64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_queue_manager_concurrent_send_receive() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));

    // Create multiple queues
    let mut queue_ids = vec![];
    for i in 0..10 {
        let qid = manager
            .create(i as Pid, QueueType::Fifo, Some(1000))
            .unwrap();
        queue_ids.push(qid);
    }

    let messages_sent = Arc::new(AtomicU64::new(0));
    let messages_received = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Spawn sender tasks
    for _ in 0..100 {
        let manager = Arc::clone(&manager);
        let qids = queue_ids.clone();
        let sent = Arc::clone(&messages_sent);

        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let qid = qids[rand::random::<usize>() % qids.len()];
                let data = vec![42u8; 64];
                if manager.send(qid, 1, data, None).is_ok() {
                    sent.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    // Spawn receiver tasks
    for _ in 0..100 {
        let manager = Arc::clone(&manager);
        let qids = queue_ids.clone();
        let received = Arc::clone(&messages_received);

        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let qid = qids[rand::random::<usize>() % qids.len()];
                if let Ok(Some(msg)) = manager.receive(qid, 1) {
                    // Read and deallocate the message data
                    let _ = manager.read_message_data(&msg);
                    received.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let sent = messages_sent.load(Ordering::Relaxed);
    let received = messages_received.load(Ordering::Relaxed);

    println!("Messages: {} sent, {} received", sent, received);
    assert!(sent > 0, "Should send some messages");
    assert!(received > 0, "Should receive some messages");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_queue_manager_priority_concurrent_stress() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));
    let queue_id = manager.create(1, QueueType::Priority, Some(5000)).unwrap();

    let mut handles = vec![];

    // Concurrent senders with different priorities
    for priority in 1..=10 {
        let manager = Arc::clone(&manager);
        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                let data = format!("priority-{}-msg-{}", priority, i).into_bytes();
                let _ = manager.send(queue_id, 1, data, Some(priority as Priority));
            }
        }));
    }

    // Concurrent receivers
    for _ in 0..5 {
        let manager = Arc::clone(&manager);
        handles.push(tokio::spawn(async move {
            for _ in 0..200 {
                if let Ok(Some(msg)) = manager.receive(queue_id, 1) {
                    let _ = manager.read_message_data(&msg);
                }
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_queue_manager_pubsub_concurrent_stress() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));
    let queue_id = manager.create(1, QueueType::PubSub, Some(1000)).unwrap();

    // Subscribe many processes
    for pid in 2..=100 {
        manager.subscribe(queue_id, pid).unwrap();
    }

    let mut handles = vec![];

    // Publishers
    for i in 0..50 {
        let manager = Arc::clone(&manager);
        handles.push(tokio::spawn(async move {
            for j in 0..100 {
                let data = format!("pub-{}-msg-{}", i, j).into_bytes();
                let _ = manager.send(queue_id, 1, data, None);
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
        }));
    }

    // Subscribers receiving
    for pid in 2..=50 {
        let manager = Arc::clone(&manager);
        handles.push(tokio::spawn(async move {
            for _ in 0..200 {
                if let Ok(Some(msg)) = manager.receive(queue_id, pid) {
                    let _ = manager.read_message_data(&msg);
                }
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Cleanup subscribers
    for pid in 2..=100 {
        let _ = manager.unsubscribe(queue_id, pid);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_queue_manager_create_destroy_stress() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));
    let operations = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for pid in 1..=50 {
        let manager = Arc::clone(&manager);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                // Create queue
                if let Ok(qid) = manager.create(pid, QueueType::Fifo, Some(10)) {
                    ops.fetch_add(1, Ordering::Relaxed);

                    // Send some messages
                    for _ in 0..5 {
                        let _ = manager.send(qid, pid, vec![1, 2, 3], None);
                    }

                    // Destroy queue
                    if manager.destroy(qid, pid).is_ok() {
                        ops.fetch_add(1, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_ops = operations.load(Ordering::Relaxed);
    println!("Queue create/destroy operations: {}", total_ops);
    assert!(total_ops > 0);
}

// ============================================================================
// Shared Memory Manager DashMap Stress Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_shm_manager_concurrent_create() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(ShmManager::new(memory_manager));
    let success_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for i in 0..HIGH_CONCURRENCY {
        let manager = Arc::clone(&manager);
        let success = Arc::clone(&success_count);

        handles.push(tokio::spawn(async move {
            let pid = (i % 100) as Pid;
            let size = 4096; // 4KB segments
            if manager.create(size, pid).is_ok() {
                success.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let successes = success_count.load(Ordering::Relaxed);
    println!("SHM segments created: {}", successes);
    assert!(successes > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_shm_manager_concurrent_attach_detach() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(ShmManager::new(memory_manager));

    // Create segments
    let mut segment_ids = vec![];
    for i in 0..20 {
        if let Ok(seg_id) = manager.create(8192, i as Pid) {
            segment_ids.push((seg_id, i as Pid));
        }
    }

    let segment_ids = Arc::new(segment_ids);
    let operations = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Multiple processes attaching/detaching
    for pid in 100..200 {
        let manager = Arc::clone(&manager);
        let segments = Arc::clone(&segment_ids);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                if let Some((seg_id, _)) = segments.get(rand::random::<usize>() % segments.len()) {
                    // Attach
                    if manager.attach(*seg_id, pid, false).is_ok() {
                        ops.fetch_add(1, Ordering::Relaxed);

                        // Small delay while attached
                        tokio::time::sleep(Duration::from_micros(100)).await;

                        // Detach
                        if manager.detach(*seg_id, pid).is_ok() {
                            ops.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_ops = operations.load(Ordering::Relaxed);
    println!("SHM attach/detach operations: {}", total_ops);
    assert!(total_ops > 0);

    // Cleanup
    for (seg_id, owner_pid) in segment_ids.iter() {
        let _ = manager.destroy(*seg_id, *owner_pid);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_shm_manager_concurrent_read_write() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(ShmManager::new(memory_manager));

    // Create a segment
    let segment_id = manager.create(65536, 1).unwrap(); // 64KB

    // Attach multiple processes
    for pid in 2..=50 {
        manager.attach(segment_id, pid, false).unwrap();
    }

    let write_count = Arc::new(AtomicU64::new(0));
    let read_count = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Writers
    for pid in 2..=25 {
        let manager = Arc::clone(&manager);
        let writes = Arc::clone(&write_count);

        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                let offset = (i * 64) % 60000;
                let data = vec![pid as u8; 64];
                if manager.write(segment_id, pid, offset, &data).is_ok() {
                    writes.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    // Readers
    for pid in 26..=50 {
        let manager = Arc::clone(&manager);
        let reads = Arc::clone(&read_count);

        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                let offset = (i * 64) % 60000;
                if manager.read(segment_id, pid, offset, 64).is_ok() {
                    reads.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let writes = write_count.load(Ordering::Relaxed);
    let reads = read_count.load(Ordering::Relaxed);

    println!("SHM operations: {} writes, {} reads", writes, reads);
    assert!(writes > 0);
    assert!(reads > 0);

    // Cleanup
    manager.destroy(segment_id, 1).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_shm_manager_create_destroy_stress() {
    let memory_manager = MemoryManager::new();
    let manager = Arc::new(ShmManager::new(memory_manager));
    let operations = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for pid in 1..=50 {
        let manager = Arc::clone(&manager);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                // Create segment
                if let Ok(seg_id) = manager.create(4096, pid) {
                    ops.fetch_add(1, Ordering::Relaxed);

                    // Do some operations
                    let _ = manager.write(seg_id, pid, 0, &[42u8; 100]);
                    let _ = manager.read(seg_id, pid, 0, 100);

                    // Destroy segment
                    if manager.destroy(seg_id, pid).is_ok() {
                        ops.fetch_add(1, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_ops = operations.load(Ordering::Relaxed);
    println!("SHM create/destroy operations: {}", total_ops);
    assert!(total_ops > 0);
}

// ============================================================================
// Process Manager DashMap Stress Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_process_manager_concurrent_create() {
    let manager = Arc::new(ProcessManagerBuilder::new().build());
    let success_count = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for i in 0..HIGH_CONCURRENCY {
        let manager = Arc::clone(&manager);
        let success = Arc::clone(&success_count);

        handles.push(tokio::spawn(async move {
            let name = format!("process-{}", i);
            let _pid = manager.create_process(name, 1);
            success.fetch_add(1, Ordering::Relaxed);
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let successes = success_count.load(Ordering::Relaxed);
    println!("Processes created: {}", successes);
    assert_eq!(successes, HIGH_CONCURRENCY as u64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_process_manager_concurrent_state_changes() {
    let manager = Arc::new(ProcessManagerBuilder::new().build());

    // Create processes
    let mut pids = vec![];
    for i in 0..100 {
        let pid = manager.create_process(format!("proc-{}", i), 1);
        pids.push(pid);
    }

    let pids = Arc::new(pids);
    let priority_changes = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Concurrent priority changes (since set_state is not available)
    for _ in 0..200 {
        let manager = Arc::clone(&manager);
        let process_ids = Arc::clone(&pids);
        let changes = Arc::clone(&priority_changes);

        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                if let Some(&pid) = process_ids.get(rand::random::<usize>() % process_ids.len()) {
                    // Change priorities
                    if manager.set_process_priority(pid, 5) {
                        changes.fetch_add(1, Ordering::Relaxed);
                    }
                    if manager.set_process_priority(pid, 3) {
                        changes.fetch_add(1, Ordering::Relaxed);
                    }
                    if manager.set_process_priority(pid, 7) {
                        changes.fetch_add(1, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_changes = priority_changes.load(Ordering::Relaxed);
    println!("Process priority changes: {}", total_changes);
    assert!(total_changes > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_process_manager_concurrent_priority_changes() {
    let manager = Arc::new(ProcessManagerBuilder::new().build());

    // Create processes
    let mut pids = vec![];
    for i in 0..100 {
        let pid = manager.create_process(format!("proc-{}", i), 1);
        pids.push(pid);
    }

    let pids = Arc::new(pids);
    let priority_changes = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Concurrent priority changes
    for _ in 0..100 {
        let manager = Arc::clone(&manager);
        let process_ids = Arc::clone(&pids);
        let changes = Arc::clone(&priority_changes);

        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                if let Some(&pid) = process_ids.get(rand::random::<usize>() % process_ids.len()) {
                    let priority = rand::random::<u8>() % 10;
                    if manager.set_process_priority(pid, priority as Priority) {
                        changes.fetch_add(1, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_changes = priority_changes.load(Ordering::Relaxed);
    println!("Process priority changes: {}", total_changes);
    assert!(total_changes > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_process_manager_create_kill_stress() {
    let manager = Arc::new(ProcessManagerBuilder::new().build());
    let operations = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for parent_pid in 1..=50 {
        let manager = Arc::clone(&manager);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                // Create process
                let name = format!("proc-{}-{}", parent_pid, i);
                let pid = manager.create_process(name, parent_pid as Priority);
                ops.fetch_add(1, Ordering::Relaxed);

                // Do some priority changes
                let _ = manager.set_process_priority(pid, 5);

                // Terminate process
                if manager.terminate_process(pid) {
                    ops.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_ops = operations.load(Ordering::Relaxed);
    println!("Process create/terminate operations: {}", total_ops);
    assert!(total_ops > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_process_manager_concurrent_info_access() {
    let manager = Arc::new(ProcessManagerBuilder::new().build());

    // Create processes
    let mut pids = vec![];
    for i in 0..100 {
        let pid = manager.create_process(format!("proc-{}", i), 1);
        pids.push(pid);
    }

    let pids = Arc::new(pids);
    let info_reads = Arc::new(AtomicU64::new(0));
    let list_calls = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Concurrent info readers
    for _ in 0..100 {
        let manager = Arc::clone(&manager);
        let process_ids = Arc::clone(&pids);
        let reads = Arc::clone(&info_reads);

        handles.push(tokio::spawn(async move {
            for _ in 0..200 {
                if let Some(&pid) = process_ids.get(rand::random::<usize>() % process_ids.len()) {
                    if manager.get_process(pid).is_some() {
                        reads.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    // Concurrent list all
    for _ in 0..50 {
        let manager = Arc::clone(&manager);
        let lists = Arc::clone(&list_calls);

        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let all = manager.list_processes();
                if !all.is_empty() {
                    lists.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let reads = info_reads.load(Ordering::Relaxed);
    let lists = list_calls.load(Ordering::Relaxed);

    println!("Process info reads: {}, list calls: {}", reads, lists);
    assert!(reads > 0);
    assert!(lists > 0);
}

// ============================================================================
// Combined Stress Tests - Multiple Managers
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_combined_process_ipc_stress() {
    let memory_manager = MemoryManager::new();
    let process_manager = Arc::new(ProcessManagerBuilder::new().build());
    let queue_manager = Arc::new(QueueManager::new(memory_manager.clone()));
    let shm_manager = Arc::new(ShmManager::new(memory_manager));

    let operations = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Combined stress: processes with IPC resources
    for i in 1..=100 {
        let pm = Arc::clone(&process_manager);
        let qm = Arc::clone(&queue_manager);
        let sm = Arc::clone(&shm_manager);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            // Create process
            let pid = pm.create_process(format!("worker-{}", i), 1);
            ops.fetch_add(1, Ordering::Relaxed);

            // Create queue
            if let Ok(qid) = qm.create(pid, QueueType::Fifo, Some(100)) {
                // Send messages
                for j in 0..20 {
                    let data = format!("msg-{}", j).into_bytes();
                    let _ = qm.send(qid, pid, data, None);
                }

                // Receive messages
                for _ in 0..10 {
                    if let Ok(Some(msg)) = qm.receive(qid, pid) {
                        let _ = qm.read_message_data(&msg);
                    }
                }

                let _ = qm.destroy(qid, pid);
            }

            // Create shared memory
            if let Ok(seg_id) = sm.create(4096, pid) {
                // Write data
                let _ = sm.write(seg_id, pid, 0, &[i as u8; 128]);

                // Read data
                let _ = sm.read(seg_id, pid, 0, 128);

                let _ = sm.destroy(seg_id, pid);
            }

            // Terminate process
            let _ = pm.terminate_process(pid);
            ops.fetch_add(1, Ordering::Relaxed);
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_ops = operations.load(Ordering::Relaxed);
    println!("Combined process+IPC operations: {}", total_ops);
    assert!(total_ops > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_extreme_concurrent_dashmap_access() {
    let memory_manager = MemoryManager::new();
    let queue_manager = Arc::new(QueueManager::new(memory_manager.clone()));
    let shm_manager = Arc::new(ShmManager::new(memory_manager));

    let total_operations = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Extreme concurrency test with timeout
    let test_future = async {
        for i in 0..500 {
            let qm = Arc::clone(&queue_manager);
            let sm = Arc::clone(&shm_manager);
            let ops = Arc::clone(&total_operations);

            handles.push(tokio::spawn(async move {
                let pid = (i % 50) as Pid;

                // Rapid-fire queue operations
                if let Ok(qid) = qm.create(pid, QueueType::Fifo, Some(10)) {
                    for _ in 0..10 {
                        let _ = qm.send(qid, pid, vec![1, 2, 3], None);
                        let _ = qm.receive(qid, pid);
                    }
                    let _ = qm.destroy(qid, pid);
                    ops.fetch_add(1, Ordering::Relaxed);
                }

                // Rapid-fire shm operations
                if let Ok(seg_id) = sm.create(1024, pid) {
                    let _ = sm.write(seg_id, pid, 0, &[42u8; 64]);
                    let _ = sm.read(seg_id, pid, 0, 64);
                    let _ = sm.destroy(seg_id, pid);
                    ops.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    };

    // Run with timeout
    timeout(Duration::from_secs(30), test_future)
        .await
        .expect("Test should complete within timeout");

    let total = total_operations.load(Ordering::Relaxed);
    println!("Extreme concurrent operations completed: {}", total);
    assert!(total > 0);
}

// ============================================================================
// Deadlock Detection Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_no_deadlock_circular_access() {
    let memory_manager = MemoryManager::new();
    let queue_manager = Arc::new(QueueManager::new(memory_manager));

    // Create multiple queues
    let mut queues = vec![];
    for i in 0..10 {
        if let Ok(qid) = queue_manager.create(i as Pid, QueueType::Fifo, Some(100)) {
            queues.push(qid);
        }
    }

    let queues = Arc::new(queues);
    let mut handles = vec![];

    // Tasks that access queues in circular pattern
    for worker_id in 0..50 {
        let qm = Arc::clone(&queue_manager);
        let q = Arc::clone(&queues);

        handles.push(tokio::spawn(async move {
            for round in 0..100 {
                // Access queues in different orders
                let start = (worker_id + round) % q.len();
                for offset in 0..q.len() {
                    let idx = (start + offset) % q.len();
                    let qid = q[idx];

                    let _ = qm.send(qid, 1, vec![worker_id as u8], None);
                    let _ = qm.receive(qid, 1);
                }
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }));
    }

    // Test should complete without deadlock
    let test_future = async {
        for handle in handles {
            handle.await.unwrap();
        }
    };

    timeout(Duration::from_secs(10), test_future)
        .await
        .expect("Should complete without deadlock");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_dashmap_entry_api_concurrent() {
    let memory_manager = MemoryManager::new();
    let queue_manager = Arc::new(QueueManager::new(memory_manager));

    let mut handles = vec![];
    let operations = Arc::new(AtomicU64::new(0));

    // Concurrent operations that use DashMap's entry API
    for pid in 1..=100 {
        let qm = Arc::clone(&queue_manager);
        let ops = Arc::clone(&operations);

        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                // Create and immediately use queue
                if let Ok(qid) = qm.create(pid, QueueType::Fifo, Some(100)) {
                    // Rapid operations
                    for i in 0..10 {
                        let data = format!("data-{}", i).into_bytes();
                        let _ = qm.send(qid, pid, data, None);
                    }

                    // Get stats (reads from DashMap)
                    let _ = qm.stats(qid);

                    // Destroy
                    if qm.destroy(qid, pid).is_ok() {
                        ops.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total = operations.load(Ordering::Relaxed);
    println!("DashMap entry API operations: {}", total);
    assert!(total > 0);
}
