/*!
 * Clipboard Manager Tests
 * Tests for clipboard operations, history, and subscriptions
 */

use ai_os_kernel::core::clipboard::{
    ClipboardData, ClipboardError, ClipboardFormat, ClipboardManager,
};

#[test]
fn test_clipboard_copy_paste() {
    let manager = ClipboardManager::new();
    let pid = 100;
    let data = ClipboardData::Text("Hello, World!".to_string());

    // Copy
    let entry_id = manager.copy(pid, data.clone()).unwrap();
    assert!(entry_id > 0);

    // Paste
    let pasted = manager.paste(pid).unwrap();
    assert_eq!(pasted.id, entry_id);
    assert_eq!(pasted.source_pid, pid);

    // Verify data
    match &pasted.data {
        ClipboardData::Text(text) => assert_eq!(text, "Hello, World!"),
        _ => panic!("Expected text data"),
    }
}

#[test]
fn test_clipboard_history() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy multiple entries
    manager
        .copy(pid, ClipboardData::Text("First".to_string()))
        .unwrap();
    manager
        .copy(pid, ClipboardData::Text("Second".to_string()))
        .unwrap();
    manager
        .copy(pid, ClipboardData::Text("Third".to_string()))
        .unwrap();

    // Check history (should have first two, current is third)
    let history = manager.history(pid, None);
    assert_eq!(history.len(), 2);

    // Check current is most recent
    let current = manager.paste(pid).unwrap();
    match &current.data {
        ClipboardData::Text(text) => assert_eq!(text, "Third"),
        _ => panic!("Expected text data"),
    }
}

#[test]
fn test_clipboard_history_limit() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy 10 entries
    for i in 0..10 {
        manager
            .copy(pid, ClipboardData::Text(format!("Entry {}", i)))
            .unwrap();
    }

    // Get limited history
    let history = manager.history(pid, Some(5));
    assert_eq!(history.len(), 5);
}

#[test]
fn test_global_clipboard() {
    let manager = ClipboardManager::new();
    let pid1 = 100;
    let _pid2 = 200;

    // Copy to global clipboard from pid1
    let data = ClipboardData::Text("Global message".to_string());
    let entry_id = manager.copy_global(pid1, data).unwrap();
    assert!(entry_id > 0);

    // Paste from global clipboard to pid2
    let pasted = manager.paste_global().unwrap();
    assert_eq!(pasted.id, entry_id);
    assert_eq!(pasted.source_pid, pid1);

    // Verify data
    match &pasted.data {
        ClipboardData::Text(text) => assert_eq!(text, "Global message"),
        _ => panic!("Expected text data"),
    }
}

#[test]
fn test_clipboard_process_isolation() {
    let manager = ClipboardManager::new();
    let pid1 = 100;
    let pid2 = 200;

    // Copy to pid1 clipboard
    manager
        .copy(pid1, ClipboardData::Text("PID1 data".to_string()))
        .unwrap();

    // Copy to pid2 clipboard
    manager
        .copy(pid2, ClipboardData::Text("PID2 data".to_string()))
        .unwrap();

    // Verify each process has its own clipboard
    let paste1 = manager.paste(pid1).unwrap();
    let paste2 = manager.paste(pid2).unwrap();

    match &paste1.data {
        ClipboardData::Text(text) => assert_eq!(text, "PID1 data"),
        _ => panic!("Expected text data"),
    }

    match &paste2.data {
        ClipboardData::Text(text) => assert_eq!(text, "PID2 data"),
        _ => panic!("Expected text data"),
    }
}

#[test]
fn test_clipboard_size_limit() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Create data larger than 10MB
    let large_data = ClipboardData::Bytes(vec![0u8; 11 * 1024 * 1024]);

    // Should fail
    let result = manager.copy(pid, large_data);
    assert!(matches!(result, Err(ClipboardError::TooLarge { .. })));
}

#[test]
fn test_clipboard_empty_data() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Try to copy empty text
    let result = manager.copy(pid, ClipboardData::Text(String::new()));
    assert!(matches!(result, Err(ClipboardError::InvalidData(_))));

    // Try to copy empty bytes
    let result = manager.copy(pid, ClipboardData::Bytes(Vec::new()));
    assert!(matches!(result, Err(ClipboardError::InvalidData(_))));
}

#[test]
fn test_clipboard_clear() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy some data
    manager
        .copy(pid, ClipboardData::Text("Test data".to_string()))
        .unwrap();

    // Verify clipboard has data
    assert!(manager.paste(pid).is_ok());

    // Clear clipboard
    manager.clear(pid);

    // Verify clipboard is empty
    let result = manager.paste(pid);
    assert!(matches!(result, Err(ClipboardError::Empty)));
}

#[test]
fn test_clipboard_get_entry() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy multiple entries
    let id1 = manager
        .copy(pid, ClipboardData::Text("Entry 1".to_string()))
        .unwrap();
    let id2 = manager
        .copy(pid, ClipboardData::Text("Entry 2".to_string()))
        .unwrap();

    // Get first entry
    let entry = manager.get_entry(pid, id1).unwrap();
    assert_eq!(entry.id, id1);

    // Get second entry
    let entry = manager.get_entry(pid, id2).unwrap();
    assert_eq!(entry.id, id2);

    // Try to get non-existent entry
    let result = manager.get_entry(pid, 99999);
    assert!(matches!(result, Err(ClipboardError::NotFound(_))));
}

#[test]
fn test_clipboard_subscriptions() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Subscribe to text format
    manager.subscribe(pid, vec![ClipboardFormat::Text]);

    // Copy text (should trigger subscription)
    manager
        .copy(pid, ClipboardData::Text("Subscribed text".to_string()))
        .unwrap();

    // Unsubscribe
    manager.unsubscribe(pid);

    // Copy more text (should not trigger subscription)
    manager
        .copy(pid, ClipboardData::Text("Not subscribed".to_string()))
        .unwrap();
}

#[test]
fn test_clipboard_html_format() {
    let manager = ClipboardManager::new();
    let pid = 100;

    let html = r#"<div><p>Hello <strong>World</strong></p></div>"#;
    let data = ClipboardData::Html(html.to_string());

    let entry_id = manager.copy(pid, data).unwrap();
    let pasted = manager.paste(pid).unwrap();

    assert_eq!(pasted.id, entry_id);
    match &pasted.data {
        ClipboardData::Html(content) => assert_eq!(content, html),
        _ => panic!("Expected HTML data"),
    }

    // Verify format
    assert_eq!(pasted.format(), ClipboardFormat::Html);
}

#[test]
fn test_clipboard_image_format() {
    let manager = ClipboardManager::new();
    let pid = 100;

    let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    let data = ClipboardData::Image {
        data: image_data.clone(),
        mime_type: "image/jpeg".to_string(),
    };

    let entry_id = manager.copy(pid, data).unwrap();
    let pasted = manager.paste(pid).unwrap();

    assert_eq!(pasted.id, entry_id);
    match &pasted.data {
        ClipboardData::Image { data, mime_type } => {
            assert_eq!(data, &image_data);
            assert_eq!(mime_type, "image/jpeg");
        }
        _ => panic!("Expected image data"),
    }
}

#[test]
fn test_clipboard_stats() {
    let manager = ClipboardManager::new();
    let pid1 = 100;
    let pid2 = 200;

    // Copy to different processes
    manager
        .copy(pid1, ClipboardData::Text("PID1 data".to_string()))
        .unwrap();
    manager
        .copy(pid2, ClipboardData::Text("PID2 data".to_string()))
        .unwrap();

    // Copy to global clipboard
    manager
        .copy_global(pid1, ClipboardData::Text("Global data".to_string()))
        .unwrap();

    // Get stats
    let stats = manager.stats();
    assert_eq!(stats.process_count, 2);
    assert!(stats.total_entries >= 3);
    assert_eq!(stats.global_entries, 1);
}

#[test]
fn test_clipboard_cleanup() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy some data
    manager
        .copy(pid, ClipboardData::Text("Test data".to_string()))
        .unwrap();

    // Subscribe
    manager.subscribe(pid, vec![]);

    // Verify data exists
    assert!(manager.paste(pid).is_ok());

    // Cleanup process
    manager.cleanup(pid);

    // Verify clipboard is cleared
    let result = manager.paste(pid);
    assert!(matches!(result, Err(ClipboardError::Empty)));

    // Verify stats show cleanup
    let stats = manager.stats();
    assert_eq!(stats.process_count, 0);
    assert_eq!(stats.subscriptions, 0);
}

#[test]
fn test_clipboard_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(ClipboardManager::new());
    let mut handles = vec![];

    // Spawn 10 threads, each copying to its own clipboard
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let pid = 100 + i;
            for j in 0..10 {
                manager_clone
                    .copy(pid, ClipboardData::Text(format!("PID {} - Entry {}", pid, j)))
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all processes have their own clipboard
    for i in 0..10 {
        let pid = 100 + i;
        let pasted = manager.paste(pid).unwrap();
        match &pasted.data {
            ClipboardData::Text(text) => {
                assert!(text.starts_with(&format!("PID {}", pid)));
            }
            _ => panic!("Expected text data"),
        }
    }
}

#[test]
fn test_clipboard_history_overflow() {
    let manager = ClipboardManager::new();
    let pid = 100;

    // Copy more than 100 entries
    for i in 0..150 {
        manager
            .copy(pid, ClipboardData::Text(format!("Entry {}", i)))
            .unwrap();
    }

    // History should be capped at 100 (99 history + 1 current)
    let history = manager.history(pid, None);
    assert!(history.len() <= 100);
}

