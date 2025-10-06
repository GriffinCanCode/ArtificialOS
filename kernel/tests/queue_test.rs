/*!
 * Async Queue Tests
 * Tests for async message queues (FIFO, Priority, PubSub)
 */

use ai_os_kernel::ipc::{QueueManager, QueueType};
use ai_os_kernel::MemoryManager;
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_create_fifo_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(100))
        .unwrap();

    assert!(queue_id > 0);

    // Verify stats
    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.queue_type, QueueType::Fifo);
    assert_eq!(stats.length, 0);
    assert_eq!(stats.capacity, 100);
}

#[test]
fn test_create_priority_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Priority, Some(50))
        .unwrap();

    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.queue_type, QueueType::Priority);
    assert_eq!(stats.capacity, 50);
}

#[test]
fn test_create_pubsub_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::PubSub, None)
        .unwrap();

    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.queue_type, QueueType::PubSub);
    assert_eq!(stats.subscriber_count, 0);
}

#[test]
fn test_fifo_send_receive() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    let data = b"Hello, Queue!".to_vec();
    manager.send(queue_id, owner_pid, data.clone(), None)
        .unwrap();

    let received = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    assert_eq!(received.data, data);
    assert_eq!(received.from, owner_pid);
}

#[test]
fn test_fifo_ordering() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    // Send messages in order
    for i in 1..=5 {
        let data = format!("Message {}", i).into_bytes();
        manager.send(queue_id, owner_pid, data, None).unwrap();
    }

    // Receive in FIFO order
    for i in 1..=5 {
        let expected = format!("Message {}", i).into_bytes();
        let received = manager.receive(queue_id, owner_pid).unwrap().unwrap();
        assert_eq!(received.data, expected);
    }
}

#[test]
fn test_priority_queue_ordering() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Priority, Some(10))
        .unwrap();

    // Send messages with different priorities
    manager.send(queue_id, owner_pid, b"Low".to_vec(), Some(1)).unwrap();
    manager.send(queue_id, owner_pid, b"High".to_vec(), Some(10)).unwrap();
    manager.send(queue_id, owner_pid, b"Medium".to_vec(), Some(5)).unwrap();

    // Should receive in priority order (highest first)
    let msg1 = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    assert_eq!(msg1.data, b"High");
    assert_eq!(msg1.priority, 10);

    let msg2 = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    assert_eq!(msg2.data, b"Medium");
    assert_eq!(msg2.priority, 5);

    let msg3 = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    assert_eq!(msg3.data, b"Low");
    assert_eq!(msg3.priority, 1);
}

#[test]
fn test_pubsub_subscribe_unsubscribe() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let sub_pid1 = 200;
    let sub_pid2 = 300;

    let queue_id = manager.create(owner_pid, QueueType::PubSub, None)
        .unwrap();

    // Subscribe two processes
    manager.subscribe(queue_id, sub_pid1).unwrap();
    manager.subscribe(queue_id, sub_pid2).unwrap();

    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.subscriber_count, 2);

    // Unsubscribe one
    manager.unsubscribe(queue_id, sub_pid1).unwrap();

    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.subscriber_count, 1);
}

#[test]
fn test_pubsub_message_delivery() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let sub_pid1 = 200;
    let sub_pid2 = 300;

    let queue_id = manager.create(owner_pid, QueueType::PubSub, None)
        .unwrap();

    // Subscribe both processes
    manager.subscribe(queue_id, sub_pid1).unwrap();
    manager.subscribe(queue_id, sub_pid2).unwrap();

    // Publish a message
    let data = b"Broadcast message".to_vec();
    manager.send(queue_id, owner_pid, data.clone(), None)
        .unwrap();

    // Both subscribers should receive it
    let msg1 = manager.receive(queue_id, sub_pid1).unwrap().unwrap();
    assert_eq!(msg1.data, data);

    let msg2 = manager.receive(queue_id, sub_pid2).unwrap().unwrap();
    assert_eq!(msg2.data, data);
}

#[test]
fn test_queue_capacity_limit() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let capacity = 5;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(capacity))
        .unwrap();

    // Fill to capacity
    for i in 0..capacity {
        let data = format!("Message {}", i).into_bytes();
        manager.send(queue_id, owner_pid, data, None).unwrap();
    }

    // Next send should fail
    let result = manager.send(queue_id, owner_pid, b"Overflow".to_vec(), None);
    assert!(result.is_err());
}

#[test]
fn test_receive_empty_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    // Receive from empty queue should return None
    let result = manager.receive(queue_id, owner_pid).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_close_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    // Send a message
    manager.send(queue_id, owner_pid, b"Test".to_vec(), None)
        .unwrap();

    // Close the queue
    manager.close(queue_id, owner_pid).unwrap();

    // Check closed status
    let stats = manager.stats(queue_id).unwrap();
    assert!(stats.closed);
}

#[test]
fn test_destroy_queue() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    // Destroy the queue
    manager.destroy(queue_id, owner_pid).unwrap();

    // Stats should fail on destroyed queue
    let result = manager.stats(queue_id);
    assert!(result.is_err());
}

#[test]
fn test_queue_stats() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    // Check initial stats
    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.length, 0);
    assert_eq!(stats.owner_pid, owner_pid);

    // Send messages
    for _ in 0..3 {
        manager.send(queue_id, owner_pid, b"Test".to_vec(), None)
            .unwrap();
    }

    // Check updated stats
    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.length, 3);
}

#[test]
fn test_concurrent_queue_operations() {
    use std::sync::Arc;
    use std::thread;

    let memory_manager = MemoryManager::new();
    let manager = Arc::new(QueueManager::new(memory_manager));

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(100))
        .unwrap();

    // Spawn multiple threads sending messages
    let mut handles = vec![];
    for i in 0..10 {
        let mgr = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let data = format!("Message {}", i).into_bytes();
            mgr.send(queue_id, owner_pid, data, None).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all sends to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Should have 10 messages
    let stats = manager.stats(queue_id).unwrap();
    assert_eq!(stats.length, 10);

    // Receive all messages
    for _ in 0..10 {
        manager.receive(queue_id, owner_pid).unwrap().unwrap();
    }
}

#[test]
fn test_multiple_queues() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let pid1 = 100;
    let pid2 = 200;

    let queue1 = manager.create(pid1, QueueType::Fifo, Some(10))
        .unwrap();
    let queue2 = manager.create(pid2, QueueType::Priority, Some(10))
        .unwrap();

    // Send to both queues
    manager.send(queue1, pid1, b"To Q1".to_vec(), None).unwrap();
    manager.send(queue2, pid2, b"To Q2".to_vec(), Some(5)).unwrap();

    // Receive from both queues
    let msg1 = manager.receive(queue1, pid1).unwrap().unwrap();
    assert_eq!(msg1.data, b"To Q1");

    let msg2 = manager.receive(queue2, pid2).unwrap().unwrap();
    assert_eq!(msg2.data, b"To Q2");
}

#[test]
fn test_message_timestamp() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Fifo, Some(10))
        .unwrap();

    manager.send(queue_id, owner_pid, b"Test".to_vec(), None)
        .unwrap();

    let msg = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    // Just verify timestamp exists (SystemTime)
    assert!(msg.timestamp.elapsed().is_ok());
}

#[test]
fn test_pubsub_no_subscribers() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::PubSub, None)
        .unwrap();

    // Should be able to publish even with no subscribers
    let result = manager.send(queue_id, owner_pid, b"Test".to_vec(), None);
    assert!(result.is_ok());
}

#[test]
fn test_priority_default_value() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let owner_pid = 100;
    let queue_id = manager.create(owner_pid, QueueType::Priority, Some(10))
        .unwrap();

    // Send without priority (should use default = 0)
    manager.send(queue_id, owner_pid, b"Default priority".to_vec(), None)
        .unwrap();

    let msg = manager.receive(queue_id, owner_pid).unwrap().unwrap();
    // Default priority should be 0
    assert_eq!(msg.priority, 0);
}

#[test]
fn test_queue_isolation() {
    let memory_manager = MemoryManager::new();
    let manager = QueueManager::new(memory_manager);

    let pid1 = 100;
    let pid2 = 200;

    let queue1 = manager.create(pid1, QueueType::Fifo, Some(10))
        .unwrap();
    let queue2 = manager.create(pid2, QueueType::Fifo, Some(10))
        .unwrap();

    // Send to both queues
    manager.send(queue1, pid1, b"Q1 Message".to_vec(), None).unwrap();
    manager.send(queue2, pid2, b"Q2 Message".to_vec(), None).unwrap();

    // Messages should not cross queues
    let msg1 = manager.receive(queue1, pid1).unwrap().unwrap();
    assert_eq!(msg1.data, b"Q1 Message");

    let msg2 = manager.receive(queue2, pid2).unwrap().unwrap();
    assert_eq!(msg2.data, b"Q2 Message");

    // No more messages in either queue
    assert!(manager.receive(queue1, pid1).unwrap().is_none());
    assert!(manager.receive(queue2, pid2).unwrap().is_none());
}
