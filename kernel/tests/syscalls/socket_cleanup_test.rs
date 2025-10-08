/*!
 * Socket Cleanup Tests
 * Comprehensive tests for socket tracking, cleanup, and FD recycling
 *
 * Note: These tests use public API only since SocketManager internals
 * are private. For more detailed internal testing, see network.rs module tests.
 */

use ai_os_kernel::syscalls::SocketManager;

#[test]
fn test_socket_manager_creation() {
    let manager = SocketManager::new();

    // Verify initial state
    let stats = manager.stats();
    assert_eq!(stats.total_tcp_listeners, 0);
    assert_eq!(stats.total_tcp_streams, 0);
    assert_eq!(stats.total_udp_sockets, 0);
    assert_eq!(stats.total_sockets(), 0);
}

#[test]
fn test_socket_count_tracking() {
    let manager = SocketManager::new();
    let pid = 100;

    // Initially no sockets
    assert_eq!(manager.get_socket_count(pid), 0);
    assert!(!manager.has_process_sockets(pid));
}

#[test]
fn test_cleanup_idempotent() {
    let manager = SocketManager::new();
    let pid = 600;

    // Cleanup on non-existent process should be safe
    let closed1 = manager.cleanup_process_sockets(pid);
    assert_eq!(closed1, 0, "Should find nothing");

    // Second cleanup should also be safe (idempotent)
    let closed2 = manager.cleanup_process_sockets(pid);
    assert_eq!(closed2, 0, "Second cleanup should also find nothing");
}

#[test]
fn test_socket_stats_interface() {
    let manager = SocketManager::new();
    let stats = manager.stats();

    // Stats struct should be usable
    let _total = stats.total_sockets();
    let _listeners = stats.total_tcp_listeners;
    let _streams = stats.total_tcp_streams;
    let _udp = stats.total_udp_sockets;
    let _recycled = stats.recycled_fds_available;
}

#[test]
fn test_manager_is_cloneable() {
    let manager1 = SocketManager::new();
    let manager2 = manager1.clone();

    // Both should have same initial state
    assert_eq!(
        manager1.get_socket_count(100),
        manager2.get_socket_count(100)
    );
}

#[test]
fn test_concurrent_cleanup_safety() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(SocketManager::new());
    let mut handles = vec![];

    // Spawn threads to cleanup different processes concurrently
    for pid in 100..110 {
        let mgr = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            // Cleanup should be safe even if process has no sockets
            let _closed = mgr.cleanup_process_sockets(pid);
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_has_process_sockets_consistency() {
    let manager = SocketManager::new();
    let pid = 200;

    // has_process_sockets should match get_socket_count
    assert_eq!(
        manager.has_process_sockets(pid),
        manager.get_socket_count(pid) > 0
    );
}
