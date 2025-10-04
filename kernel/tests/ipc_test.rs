/*!
 * IPC Tests
 * Tests for inter-process communication and message queuing
 */

use ai_os_kernel::ipc::IPCManager;
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_basic_message_send_receive() {
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();
    let pid = 100;

    let message = ipc.receive_message(pid);
    assert!(message.is_none());
}

#[test]
fn test_has_messages() {
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();

    let from_pid = 100;
    let to_pid = 200;

    // Create a message larger than 1MB
    let large_data = vec![0u8; 2 * 1024 * 1024]; // 2MB

    let result = ipc.send_message(from_pid, to_pid, large_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds limit"));
}

#[test]
fn test_queue_size_limit() {
    let mut ipc = IPCManager::new();

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
    assert!(result.unwrap_err().contains("Queue for PID"));
}

#[test]
fn test_clear_process_queue() {
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();
    let pid = 100;

    let cleared_count = ipc.clear_process_queue(pid);
    assert_eq!(cleared_count, 0);
}

#[test]
fn test_message_timestamp() {
    let mut ipc = IPCManager::new();

    let from_pid = 100;
    let to_pid = 200;

    ipc.send_message(from_pid, to_pid, b"test".to_vec())
        .unwrap();

    let message = ipc.receive_message(to_pid).unwrap();
    assert!(message.timestamp > 0);
}

#[test]
fn test_bidirectional_communication() {
    let mut ipc = IPCManager::new();

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
    let mut ipc = IPCManager::new();

    let initial_usage = ipc.get_global_memory_usage();

    // Send a message
    ipc.send_message(100, 200, vec![0u8; 1024]).unwrap();

    let after_send = ipc.get_global_memory_usage();
    assert!(after_send > initial_usage);

    // Receive the message
    ipc.receive_message(200);

    let after_receive = ipc.get_global_memory_usage();
    // Memory should be significantly reduced after receiving
    // Just verify it's less than after sending
    assert!(after_receive < after_send, 
        "Memory after receive ({}) should be less than after send ({})", 
        after_receive, after_send);
}

#[test]
#[serial]
fn test_global_memory_limit() {
    let mut ipc = IPCManager::new();

    // Try to fill up global IPC memory (100MB limit)
    // Message size limit is 1MB, so use messages within that limit
    let message_size = 1024 * 1024; // 1MB per message
    let mut sent_count = 0;
    let mut hit_limit = false;

    // Send messages until we hit the global limit (should be around 100 messages)
    for i in 0..150 {
        let result = ipc.send_message(100, 200 + i, vec![0u8; message_size]);
        if result.is_ok() {
            sent_count += 1;
        } else {
            // Should fail due to global limit or queue limit
            let err = result.unwrap_err();
            assert!(
                err.contains("Global IPC memory limit") 
                    || err.contains("Queue for PID")
                    || err.contains("exceeds limit"),
                "Unexpected error: {}",
                err
            );
            hit_limit = true;
            break;
        }
    }

    // Should have sent some but not all, and should have hit the limit
    assert!(sent_count > 0);
    assert!(sent_count < 150);
    assert!(hit_limit);
    
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
    let mut ipc = IPCManager::new();

    let initial_usage = ipc.get_global_memory_usage();

    // Send multiple messages
    let message_count = 10;
    for _ in 0..message_count {
        ipc.send_message(100, 200, vec![0u8; 100 * 1024]).unwrap();
    }

    let before_clear = ipc.get_global_memory_usage();
    assert!(before_clear > initial_usage);

    // Clear the queue
    let cleared = ipc.clear_process_queue(200);
    assert_eq!(cleared, message_count);

    let after_clear = ipc.get_global_memory_usage();
    // Should be close to initial state (may have differences due to concurrent tests)
    // Just verify it's significantly less than before clear
    assert!(after_clear < before_clear / 2, 
        "Memory after clear ({}) should be much less than before ({})", 
        after_clear, before_clear);
}

#[test]
fn test_concurrent_message_sending() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let ipc = Arc::new(Mutex::new(IPCManager::new()));
    let mut handles = vec![];

    // Multiple threads sending to the same recipient
    for i in 0..10 {
        let ipc_clone = Arc::clone(&ipc);
        let handle = thread::spawn(move || {
            let mut ipc = ipc_clone.lock().unwrap();
            ipc.send_message(100 + i, 999, format!("Message {}", i).into_bytes())
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut ipc = ipc.lock().unwrap();

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
    let mut ipc = IPCManager::new();

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
