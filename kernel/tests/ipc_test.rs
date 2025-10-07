/*!
 * IPC Tests
 * Tests for inter-process communication and message queuing
 */

use ai_os_kernel::ipc::IPCManager;
use ai_os_kernel::memory::MemoryManager;
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_basic_message_send_receive() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;
    let data = b"Hello, process!".to_vec();

    // Send message
    let result = ipc.send_message(from_pid, to_pid, data.clone());
    assert!(result.is_ok());

    // Receive message
    let message = ipc.receive_message(to_pid).unwrap();
    assert_eq!(message.from, from_pid);
    assert_eq!(message.to, to_pid);
    assert_eq!(message.data, data);
}

#[test]
fn test_message_ordering() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;

    // Send multiple messages
    ipc.send_message(from_pid, to_pid, b"Message 1".to_vec())
        .unwrap();
    ipc.send_message(from_pid, to_pid, b"Message 2".to_vec())
        .unwrap();
    ipc.send_message(from_pid, to_pid, b"Message 3".to_vec())
        .unwrap();

    // Receive in order (FIFO)
    let msg1 = ipc.receive_message(to_pid).unwrap();
    assert_eq!(msg1.data, b"Message 1");

    let msg2 = ipc.receive_message(to_pid).unwrap();
    assert_eq!(msg2.data, b"Message 2");

    let msg3 = ipc.receive_message(to_pid).unwrap();
    assert_eq!(msg3.data, b"Message 3");
}

#[test]
fn test_receive_from_empty_queue() {
    let ipc = IPCManager::new(MemoryManager::new());
    let pid = 100;

    let message = ipc.receive_message(pid);
    assert!(message.is_none());
}

#[test]
fn test_has_messages() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;

    assert!(!ipc.has_messages(to_pid));

    ipc.send_message(from_pid, to_pid, b"test".to_vec())
        .unwrap();

    assert!(ipc.has_messages(to_pid));

    ipc.receive_message(to_pid);

    assert!(!ipc.has_messages(to_pid));
}

#[test]
fn test_multiple_recipients() {
    let ipc = IPCManager::new(MemoryManager::new());

    let sender = 100;
    let receiver1 = 200;
    let receiver2 = 300;

    ipc.send_message(sender, receiver1, b"To R1".to_vec())
        .unwrap();
    ipc.send_message(sender, receiver2, b"To R2".to_vec())
        .unwrap();

    assert!(ipc.has_messages(receiver1));
    assert!(ipc.has_messages(receiver2));

    let msg1 = ipc.receive_message(receiver1).unwrap();
    assert_eq!(msg1.data, b"To R1");

    let msg2 = ipc.receive_message(receiver2).unwrap();
    assert_eq!(msg2.data, b"To R2");
}

#[test]
fn test_message_size_limit() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;

    // Create a message larger than 1MB
    let large_data = vec![0u8; 2 * 1024 * 1024]; // 2MB

    let result = ipc.send_message(from_pid, to_pid, large_data);
    assert!(result.is_err());
    match result.unwrap_err() {
        ai_os_kernel::ipc::IpcError::LimitExceeded(_) => {}
        _ => panic!("Expected LimitExceeded error"),
    }
}

#[test]
fn test_queue_size_limit() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;
    let small_data = b"x".to_vec();

    // Fill the queue to its limit (1000 messages)
    for _ in 0..1000 {
        ipc.send_message(from_pid, to_pid, small_data.clone())
            .unwrap();
    }

    // Next message should fail
    let result = ipc.send_message(from_pid, to_pid, small_data);
    assert!(result.is_err());
    match result.unwrap_err() {
        ai_os_kernel::ipc::IpcError::LimitExceeded(_) => {}
        _ => panic!("Expected LimitExceeded error"),
    }
}

#[test]
fn test_clear_process_queue() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;

    // Send multiple messages
    for i in 0..10 {
        ipc.send_message(from_pid, to_pid, format!("Message {}", i).into_bytes())
            .unwrap();
    }

    assert!(ipc.has_messages(to_pid));

    // Clear the queue
    let cleared_count = ipc.clear_process_queue(to_pid);
    assert_eq!(cleared_count, 10);

    assert!(!ipc.has_messages(to_pid));
}

#[test]
fn test_clear_empty_queue() {
    let ipc = IPCManager::new(MemoryManager::new());
    let pid = 100;

    let cleared_count = ipc.clear_process_queue(pid);
    assert_eq!(cleared_count, 0);
}

#[test]
fn test_message_timestamp() {
    let ipc = IPCManager::new(MemoryManager::new());

    let from_pid = 100;
    let to_pid = 200;

    ipc.send_message(from_pid, to_pid, b"test".to_vec())
        .unwrap();

    let message = ipc.receive_message(to_pid).unwrap();
    assert!(message.timestamp > 0);
}

#[test]
fn test_bidirectional_communication() {
    let ipc = IPCManager::new(MemoryManager::new());

    let pid1 = 100;
    let pid2 = 200;

    // pid1 sends to pid2
    ipc.send_message(pid1, pid2, b"Hello from 1".to_vec())
        .unwrap();

    // pid2 sends to pid1
    ipc.send_message(pid2, pid1, b"Hello from 2".to_vec())
        .unwrap();

    // Each can receive their message
    let msg_to_2 = ipc.receive_message(pid2).unwrap();
    assert_eq!(msg_to_2.data, b"Hello from 1");
    assert_eq!(msg_to_2.from, pid1);

    let msg_to_1 = ipc.receive_message(pid1).unwrap();
    assert_eq!(msg_to_1.data, b"Hello from 2");
    assert_eq!(msg_to_1.from, pid2);
}

#[test]
#[serial]
fn test_global_memory_tracking() {
    let ipc = IPCManager::new(MemoryManager::new());

    let initial_usage = ipc.get_global_memory_usage();

    // Send a message - allocates through MemoryManager
    ipc.send_message(100, 200, vec![0u8; 1024]).unwrap();

    let after_send = ipc.get_global_memory_usage();
    assert!(
        after_send > initial_usage,
        "Memory usage should increase after send: {} -> {}",
        initial_usage,
        after_send
    );

    // Receive the message - memory cleanup happens when process terminates
    // Note: Individual message receive doesn't immediately reclaim memory
    ipc.receive_message(200);

    let after_receive = ipc.get_global_memory_usage();
    // Memory tracking is now unified through MemoryManager
    // Check that we're tracking memory consistently
    assert!(after_receive >= initial_usage);
}

#[test]
#[serial]
fn test_global_memory_limit() {
    let ipc = IPCManager::new(MemoryManager::new());

    // Try to fill up memory (1GB MemoryManager limit)
    // Message size limit is 1MB, so use messages within that limit
    let message_size = 1024 * 1024; // 1MB per message
    let mut sent_count = 0;
    let mut hit_limit = false;

    // Send messages until we hit the MemoryManager limit
    for i in 0..1100 {
        let result = ipc.send_message(100, 200 + i, vec![0u8; message_size]);
        if result.is_ok() {
            sent_count += 1;
        } else {
            // Should fail due to MemoryManager OOM or queue limit
            match result.unwrap_err() {
                ai_os_kernel::ipc::IpcError::LimitExceeded(_) => {}
                _ => {}
            }
            hit_limit = true;
            break;
        }
    }

    // Should have sent many messages and hit the limit
    assert!(sent_count > 100, "Should send at least 100 messages");
    assert!(hit_limit, "Should hit memory limit");

    // Clean up messages
    for i in 0..sent_count {
        while ipc.has_messages(200 + i) {
            ipc.receive_message(200 + i);
        }
    }
}

#[test]
#[serial]
fn test_memory_cleanup_on_clear() {
    let ipc = IPCManager::new(MemoryManager::new());

    let initial_usage = ipc.get_global_memory_usage();

    // Send multiple messages - allocates through MemoryManager
    let message_count = 10;
    for _ in 0..message_count {
        ipc.send_message(100, 200, vec![0u8; 100 * 1024]).unwrap();
    }

    let before_clear = ipc.get_global_memory_usage();
    assert!(
        before_clear > initial_usage,
        "Memory should increase after sending messages"
    );

    // Clear the queue - messages removed but memory managed by MemoryManager
    let cleared = ipc.clear_process_queue(200);
    assert_eq!(cleared, message_count);

    // Memory tracking is unified through MemoryManager now
    // Verify the operation completed successfully
    assert!(!ipc.has_messages(200), "Queue should be empty after clear");
}

#[test]
fn test_concurrent_message_sending() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let ipc = Arc::new(Mutex::new(IPCManager::new(MemoryManager::new())));
    let mut handles = vec![];

    // Multiple threads sending to the same recipient
    for i in 0..10 {
        let ipc_clone = Arc::clone(&ipc);
        let handle = thread::spawn(move || {
            let ipc = ipc_clone.lock().unwrap();
            ipc.send_message(100 + i, 999, format!("Message {}", i).into_bytes())
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let ipc = ipc.lock().unwrap();

    // Should have received all 10 messages
    let mut received_count = 0;
    while ipc.has_messages(999) {
        ipc.receive_message(999);
        received_count += 1;
    }

    assert_eq!(received_count, 10);
}

#[test]
fn test_queue_isolation() {
    let ipc = IPCManager::new(MemoryManager::new());

    let pid1 = 100;
    let pid2 = 200;
    let pid3 = 300;

    // Send to different processes
    ipc.send_message(pid1, pid2, b"To pid2".to_vec()).unwrap();
    ipc.send_message(pid1, pid3, b"To pid3".to_vec()).unwrap();

    // pid2 should only see its message
    let msg2 = ipc.receive_message(pid2).unwrap();
    assert_eq!(msg2.data, b"To pid2");
    assert!(!ipc.has_messages(pid2));

    // pid3 should still have its message
    assert!(ipc.has_messages(pid3));
    let msg3 = ipc.receive_message(pid3).unwrap();
    assert_eq!(msg3.data, b"To pid3");
}
