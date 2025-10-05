/*!
 * Pipe Tests
 * Tests for Unix-style pipe IPC
 */

use ai_os_kernel::pipe::{PipeError, PipeManager};
use pretty_assertions::assert_eq;
use serial_test::serial;

#[test]
fn test_basic_pipe_create() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();
    assert!(pipe_id > 0);
}

#[test]
fn test_pipe_write_read() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    // Write data
    let data = b"Hello through pipe!";
    let written = pm.write(pipe_id, writer_pid, data).unwrap();
    assert_eq!(written, data.len());

    // Read data
    let read_data = pm.read(pipe_id, reader_pid, data.len()).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn test_pipe_streaming() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    // Write multiple chunks
    pm.write(pipe_id, writer_pid, b"chunk1").unwrap();
    pm.write(pipe_id, writer_pid, b"chunk2").unwrap();
    pm.write(pipe_id, writer_pid, b"chunk3").unwrap();

    // Read back
    let chunk1 = pm.read(pipe_id, reader_pid, 6).unwrap();
    assert_eq!(chunk1, b"chunk1");

    let chunk2 = pm.read(pipe_id, reader_pid, 6).unwrap();
    assert_eq!(chunk2, b"chunk2");

    let chunk3 = pm.read(pipe_id, reader_pid, 6).unwrap();
    assert_eq!(chunk3, b"chunk3");
}

#[test]
fn test_pipe_permissions() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;
    let other_pid = 300;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    // Writer can't read
    let result = pm.read(pipe_id, writer_pid, 100);
    assert!(matches!(result, Err(PipeError::PermissionDenied(_))));

    // Reader can't write
    let result = pm.write(pipe_id, reader_pid, b"test");
    assert!(matches!(result, Err(PipeError::PermissionDenied(_))));

    // Other process can't access
    let result = pm.write(pipe_id, other_pid, b"test");
    assert!(matches!(result, Err(PipeError::PermissionDenied(_))));

    let result = pm.read(pipe_id, other_pid, 100);
    assert!(matches!(result, Err(PipeError::PermissionDenied(_))));
}

#[test]
fn test_pipe_capacity() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let capacity = 1024; // 1KB
    let pipe_id = pm.create(reader_pid, writer_pid, Some(capacity)).unwrap();

    // Fill the pipe
    let data = vec![0u8; capacity];
    let written = pm.write(pipe_id, writer_pid, &data).unwrap();
    assert_eq!(written, capacity);

    // Next write should fail (would block)
    let result = pm.write(pipe_id, writer_pid, b"x");
    assert!(matches!(result, Err(PipeError::WouldBlock(_))));

    // Read some data
    let _ = pm.read(pipe_id, reader_pid, 100).unwrap();

    // Now we can write again
    let written = pm.write(pipe_id, writer_pid, b"success").unwrap();
    assert_eq!(written, 7);
}

#[test]
fn test_pipe_read_empty() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    // Reading from empty pipe should block
    let result = pm.read(pipe_id, reader_pid, 100);
    assert!(matches!(result, Err(PipeError::WouldBlock(_))));
}

#[test]
fn test_pipe_close() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    // Write some data
    pm.write(pipe_id, writer_pid, b"data").unwrap();

    // Close the pipe
    pm.close(pipe_id, writer_pid).unwrap();

    // Writing after close should fail
    let result = pm.write(pipe_id, writer_pid, b"test");
    assert!(matches!(result, Err(PipeError::Closed)));

    // Reading existing data should still work
    let data = pm.read(pipe_id, reader_pid, 4).unwrap();
    assert_eq!(data, b"data");

    // Reading after buffer empty should return EOF (empty vec)
    let result = pm.read(pipe_id, reader_pid, 100);
    match result {
        Ok(data) if data.is_empty() => {} // EOF
        Err(PipeError::Closed) => {}      // Also acceptable
        _ => panic!("Expected EOF or Closed error"),
    }
}

#[test]
fn test_pipe_destroy() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, None).unwrap();

    pm.destroy(pipe_id).unwrap();

    // Operations on destroyed pipe should fail
    let result = pm.write(pipe_id, writer_pid, b"test");
    assert!(matches!(result, Err(PipeError::NotFound(_))));

    let result = pm.read(pipe_id, reader_pid, 100);
    assert!(matches!(result, Err(PipeError::NotFound(_))));
}

#[test]
fn test_pipe_stats() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let pipe_id = pm.create(reader_pid, writer_pid, Some(4096)).unwrap();

    // Write some data
    pm.write(pipe_id, writer_pid, b"test data").unwrap();

    // Get stats
    let stats = pm.stats(pipe_id).unwrap();
    assert_eq!(stats.id, pipe_id);
    assert_eq!(stats.reader_pid, reader_pid);
    assert_eq!(stats.writer_pid, writer_pid);
    assert_eq!(stats.capacity, 4096);
    assert_eq!(stats.buffered, 9);
    assert!(!stats.closed);
}

#[test]
fn test_pipe_process_cleanup() {
    let pm = PipeManager::new();

    let pid = 100;

    // Create multiple pipes involving this process
    pm.create(pid, 200, None).unwrap();
    pm.create(pid, 300, None).unwrap();
    pm.create(400, pid, None).unwrap();

    // Cleanup
    let count = pm.cleanup_process(pid);
    assert_eq!(count, 3);
}

#[test]
fn test_pipe_per_process_limit() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    // Create pipes up to the limit (100 per process)
    for i in 0..100 {
        pm.create(reader_pid, 200 + i, None).unwrap();
    }

    // Next creation should fail
    let result = pm.create(reader_pid, writer_pid, None);
    assert!(matches!(result, Err(PipeError::ProcessLimitExceeded(_, _))));
}

#[test]
#[serial]
fn test_pipe_global_memory_limit() {
    let pm = PipeManager::new();

    let mut created = 0;

    // Try to create pipes that exceed global limit (50MB)
    // Each pipe has 64KB capacity
    let capacity: usize = 65536;
    let max_pipes = (50 * 1024 * 1024) / capacity; // ~800 pipes

    // Distribute across multiple processes to avoid per-process limit
    for i in 0..max_pipes + 10 {
        let reader_pid = 100 + (i / 50) as u32; // Change process every 50 pipes
        let writer_pid = 1000 + (i / 50) as u32;

        match pm.create(reader_pid, writer_pid, Some(capacity)) {
            Ok(_) => created += 1,
            Err(PipeError::GlobalMemoryExceeded(_, _)) => break,
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    // Should have hit the global limit
    assert!(created >= max_pipes - 1 && created <= max_pipes + 1);

    // Clean up - cleanup all processes used
    for i in 0..((created / 50) + 2) {
        pm.cleanup_process(100 + i as u32);
        pm.cleanup_process(1000 + i as u32);
    }
}

#[test]
fn test_pipe_partial_write() {
    let pm = PipeManager::new();

    let reader_pid = 100;
    let writer_pid = 200;

    let capacity = 100;
    let pipe_id = pm.create(reader_pid, writer_pid, Some(capacity)).unwrap();

    // Fill most of the pipe
    let data1 = vec![0u8; 90];
    pm.write(pipe_id, writer_pid, &data1).unwrap();

    // Try to write more than available
    let data2 = vec![1u8; 50];
    let written = pm.write(pipe_id, writer_pid, &data2).unwrap();

    // Should only write what fits
    assert_eq!(written, 10);

    // Total buffered should be capacity
    let stats = pm.stats(pipe_id).unwrap();
    assert_eq!(stats.buffered, 100);
}

#[test]
fn test_pipe_bidirectional_setup() {
    let pm = PipeManager::new();

    let pid1 = 100;
    let pid2 = 200;

    // Create two pipes for bidirectional communication
    let pipe1 = pm.create(pid1, pid2, None).unwrap(); // pid1 reads, pid2 writes
    let pipe2 = pm.create(pid2, pid1, None).unwrap(); // pid2 reads, pid1 writes

    // pid1 sends to pid2
    pm.write(pipe2, pid1, b"Hello from 1").unwrap();
    let msg = pm.read(pipe2, pid2, 100).unwrap();
    assert_eq!(msg, b"Hello from 1");

    // pid2 sends to pid1
    pm.write(pipe1, pid2, b"Hello from 2").unwrap();
    let msg = pm.read(pipe1, pid1, 100).unwrap();
    assert_eq!(msg, b"Hello from 2");
}

#[test]
#[serial]
fn test_pipe_memory_tracking() {
    let pm = PipeManager::new();

    let initial = pm.get_global_memory_usage();

    let capacity = 10000;
    let pipe_id = pm.create(100, 200, Some(capacity)).unwrap();

    let after_create = pm.get_global_memory_usage();
    assert_eq!(after_create, initial + capacity);

    pm.destroy(pipe_id).unwrap();

    let after_destroy = pm.get_global_memory_usage();
    assert_eq!(after_destroy, initial);
}

#[test]
fn test_pipe_not_found() {
    let pm = PipeManager::new();

    let result = pm.write(999, 100, b"test");
    assert!(matches!(result, Err(PipeError::NotFound(999))));

    let result = pm.read(999, 100, 100);
    assert!(matches!(result, Err(PipeError::NotFound(999))));

    let result = pm.stats(999);
    assert!(matches!(result, Err(PipeError::NotFound(999))));
}

#[test]
fn test_pipe_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;

    let pm = Arc::new(PipeManager::new());
    let pipe_id = pm.create(100, 200, Some(10000)).unwrap();

    let pm_writer = Arc::clone(&pm);
    let pm_reader = Arc::clone(&pm);

    // Writer thread
    let writer = thread::spawn(move || {
        for i in 0..10 {
            let data = format!("Message {}", i);
            // Keep trying until write succeeds (in case pipe is full)
            while pm_writer.write(pipe_id, 200, data.as_bytes()).is_err() {
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });

    // Reader thread
    let reader = thread::spawn(move || {
        let mut messages = Vec::new();
        for _ in 0..10 {
            // Keep trying until read succeeds
            loop {
                match pm_reader.read(pipe_id, 100, 100) {
                    Ok(data) => {
                        messages.push(String::from_utf8(data).unwrap());
                        break;
                    }
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        }
        messages
    });

    writer.join().unwrap();
    let messages = reader.join().unwrap();

    // Should have received all 10 messages
    assert_eq!(messages.len(), 10);
    for i in 0..10 {
        assert_eq!(messages[i], format!("Message {}", i));
    }
}
