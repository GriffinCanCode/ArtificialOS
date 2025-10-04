/*!
 * Process Manager Tests
 * Tests for process creation, lifecycle, and memory integration
 */

use ai_os_kernel::memory::MemoryManager;
use ai_os_kernel::process::{ProcessManager, ProcessState};
use pretty_assertions::assert_eq;

#[test]
fn test_process_creation() {
    let pm = ProcessManager::new();
    let pid = pm.create_process("test-app".to_string(), 5);

    assert_eq!(pid, 1);

    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.name, "test-app");
    assert_eq!(process.priority, 5);
    assert!(matches!(process.state, ProcessState::Ready));
}

#[test]
fn test_multiple_process_creation() {
    let pm = ProcessManager::new();

    let pid1 = pm.create_process("app1".to_string(), 5);
    let pid2 = pm.create_process("app2".to_string(), 3);
    let pid3 = pm.create_process("app3".to_string(), 8);

    assert_eq!(pid1, 1);
    assert_eq!(pid2, 2);
    assert_eq!(pid3, 3);

    let processes = pm.list_processes();
    assert_eq!(processes.len(), 3);
}

#[test]
fn test_process_termination() {
    let pm = ProcessManager::new();
    let pid = pm.create_process("test-app".to_string(), 5);

    assert!(pm.get_process(pid).is_some());

    let terminated = pm.terminate_process(pid);
    assert!(terminated);

    // Process should be removed after termination
    assert!(pm.get_process(pid).is_none());
}

#[test]
fn test_terminate_nonexistent_process() {
    let pm = ProcessManager::new();
    let terminated = pm.terminate_process(999);
    assert!(!terminated);
}

#[test]
fn test_list_processes() {
    let pm = ProcessManager::new();

    pm.create_process("app1".to_string(), 5);
    pm.create_process("app2".to_string(), 3);
    pm.create_process("app3".to_string(), 8);

    let processes = pm.list_processes();
    assert_eq!(processes.len(), 3);

    let names: Vec<String> = processes.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"app1".to_string()));
    assert!(names.contains(&"app2".to_string()));
    assert!(names.contains(&"app3".to_string()));
}

#[test]
fn test_process_manager_with_memory() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::with_memory_manager(mem_mgr.clone());

    let pid = pm.create_process("test-app".to_string(), 5);

    // Allocate memory for the process
    mem_mgr.allocate(10 * 1024 * 1024, pid).unwrap();

    let mem_used = mem_mgr.get_process_memory(pid);
    assert_eq!(mem_used, 10 * 1024 * 1024);

    // Terminate process - should clean up memory
    pm.terminate_process(pid);

    // Memory should be freed
    let mem_after = mem_mgr.get_process_memory(pid);
    assert_eq!(mem_after, 0);
}

#[test]
fn test_process_memory_cleanup_on_termination() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::with_memory_manager(mem_mgr.clone());

    let pid1 = pm.create_process("app1".to_string(), 5);
    let pid2 = pm.create_process("app2".to_string(), 5);

    // Allocate memory for both processes
    mem_mgr.allocate(20 * 1024 * 1024, pid1).unwrap();
    mem_mgr.allocate(30 * 1024 * 1024, pid2).unwrap();

    let (_, used_before, _) = mem_mgr.get_memory_info();
    assert_eq!(used_before, 50 * 1024 * 1024);

    // Terminate one process
    pm.terminate_process(pid1);

    // Only pid2's memory should remain
    let (_, used_after, _) = mem_mgr.get_memory_info();
    assert_eq!(used_after, 30 * 1024 * 1024);
}

#[test]
fn test_concurrent_process_creation() {
    use std::sync::Arc;
    use std::thread;

    let pm = Arc::new(ProcessManager::new());
    let mut handles = vec![];

    for i in 0..10 {
        let pm_clone = Arc::clone(&pm);
        let handle = thread::spawn(move || {
            pm_clone.create_process(format!("app-{}", i), 5);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let processes = pm.list_processes();
    assert_eq!(processes.len(), 10);
}

#[test]
fn test_get_process_details() {
    let pm = ProcessManager::new();
    let pid = pm.create_process("detailed-app".to_string(), 7);

    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.pid, pid);
    assert_eq!(process.name, "detailed-app");
    assert_eq!(process.priority, 7);
}

#[test]
fn test_process_manager_clone() {
    let pm1 = ProcessManager::new();
    pm1.create_process("app1".to_string(), 5);

    let pm2 = pm1.clone();
    let processes = pm2.list_processes();
    assert_eq!(processes.len(), 1);

    // Both should share the same underlying data
    pm2.create_process("app2".to_string(), 3);
    let processes_pm1 = pm1.list_processes();
    assert_eq!(processes_pm1.len(), 2);
}
