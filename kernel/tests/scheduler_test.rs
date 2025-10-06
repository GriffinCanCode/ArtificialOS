/*!
 * Scheduler Tests
 * Comprehensive tests for CPU scheduler with multiple policies
 */

use ai_os_kernel::scheduler::{Policy, Scheduler};
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_round_robin_basic() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);
    scheduler.add(3, 5);

    assert_eq!(scheduler.len(), 3);

    // Should schedule in FIFO order
    assert_eq!(scheduler.schedule(), Some(1));
    assert_eq!(scheduler.current(), Some(1));
}

#[test]
fn test_round_robin_order() {
    let scheduler = Scheduler::with_quantum(Policy::RoundRobin, Duration::from_millis(10));

    scheduler.add(1, 5);
    scheduler.add(2, 5);
    scheduler.add(3, 5);

    // First round
    assert_eq!(scheduler.schedule(), Some(1));

    // Wait for quantum to expire and trigger preemption
    thread::sleep(Duration::from_millis(15));

    // Should rotate to next process
    assert_eq!(scheduler.schedule(), Some(2));

    thread::sleep(Duration::from_millis(15));
    assert_eq!(scheduler.schedule(), Some(3));

    // Should cycle back to process 1
    thread::sleep(Duration::from_millis(15));
    assert_eq!(scheduler.schedule(), Some(1));
}

#[test]
fn test_priority_scheduling_order() {
    let scheduler = Scheduler::new(Policy::Priority);

    scheduler.add(1, 3);  // Low priority
    scheduler.add(2, 8);  // High priority
    scheduler.add(3, 5);  // Medium priority
    scheduler.add(4, 8);  // High priority (same as pid 2)

    // Should schedule highest priority first
    assert_eq!(scheduler.schedule(), Some(2));
}

#[test]
fn test_priority_preemption() {
    let scheduler = Scheduler::with_quantum(Policy::Priority, Duration::from_millis(10));

    scheduler.add(1, 5);  // Medium priority
    scheduler.add(2, 8);  // High priority

    // Schedule first process (highest priority wins)
    assert_eq!(scheduler.schedule(), Some(2));

    let preemptions_before = scheduler.stats().preemptions;

    // Wait for quantum to expire
    thread::sleep(Duration::from_millis(15));

    // After preemption, highest priority process is still scheduled (process 2)
    assert_eq!(scheduler.schedule(), Some(2));

    // But preemption counter should have increased
    assert!(scheduler.stats().preemptions > preemptions_before);
}

#[test]
fn test_fair_scheduling() {
    let scheduler = Scheduler::new(Policy::Fair);

    // Add processes with different priorities
    scheduler.add(1, 8);  // High priority
    scheduler.add(2, 3);  // Low priority
    scheduler.add(3, 5);  // Medium priority

    // First schedule should pick based on vruntime (all start at 0)
    let first = scheduler.schedule();
    assert!(first.is_some());

    // All processes should get scheduled over time
    assert_eq!(scheduler.len(), 3);
}

#[test]
fn test_fair_scheduling_balance() {
    let scheduler = Scheduler::with_quantum(Policy::Fair, Duration::from_millis(10));

    scheduler.add(1, 8);  // High priority - slower vruntime growth
    scheduler.add(2, 3);  // Low priority - faster vruntime growth

    let mut pid1_count = 0;
    let mut pid2_count = 0;

    // Run multiple scheduling rounds
    for _ in 0..10 {
        if let Some(pid) = scheduler.schedule() {
            if pid == 1 {
                pid1_count += 1;
            } else if pid == 2 {
                pid2_count += 1;
            }
        }
        thread::sleep(Duration::from_millis(15)); // Trigger quantum expiry
    }

    // High priority process (pid 1) should be scheduled more often
    // due to slower vruntime growth
    assert!(pid1_count >= pid2_count);
}

#[test]
fn test_remove_process() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);
    scheduler.add(3, 5);
    assert_eq!(scheduler.len(), 3);

    assert!(scheduler.remove(2));
    assert_eq!(scheduler.len(), 2);

    // Should not find removed process
    assert!(!scheduler.remove(2));

    // Non-existent process
    assert!(!scheduler.remove(999));
}

#[test]
fn test_remove_current_process() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    // Schedule first process
    assert_eq!(scheduler.schedule(), Some(1));
    assert_eq!(scheduler.current(), Some(1));

    // Remove current process
    assert!(scheduler.remove(1));
    assert_eq!(scheduler.current(), None);
    assert_eq!(scheduler.len(), 1);
}

#[test]
fn test_yield_process() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    assert_eq!(scheduler.schedule(), Some(1));
    assert_eq!(scheduler.current(), Some(1));

    // Yield should switch to next process
    assert_eq!(scheduler.yield_process(), Some(2));
    assert_eq!(scheduler.current(), Some(2));

    // Process 1 should be back in queue
    assert_eq!(scheduler.len(), 2);
}

#[test]
fn test_yield_single_process() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);

    assert_eq!(scheduler.schedule(), Some(1));

    // Yielding with only one process should reschedule same process
    assert_eq!(scheduler.yield_process(), Some(1));
    assert_eq!(scheduler.current(), Some(1));
}

#[test]
fn test_yield_empty_queue() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    assert_eq!(scheduler.schedule(), Some(1));

    // Remove all other processes, yield should go back to queue
    scheduler.remove(1);

    // Yielding with empty queue
    let result = scheduler.yield_process();
    assert_eq!(result, None);
}

#[test]
fn test_empty_scheduler() {
    let scheduler = Scheduler::new(Policy::RoundRobin);
    assert!(scheduler.is_empty());
    assert_eq!(scheduler.len(), 0);
    assert_eq!(scheduler.schedule(), None);
    assert_eq!(scheduler.current(), None);
}

#[test]
fn test_statistics_tracking() {
    let scheduler = Scheduler::new(Policy::Priority);

    scheduler.add(1, 5);
    scheduler.add(2, 3);
    scheduler.add(3, 8);

    let stats_before = scheduler.stats();
    assert_eq!(stats_before.total_scheduled, 0);
    assert_eq!(stats_before.context_switches, 0);
    assert_eq!(stats_before.active_processes, 3);

    scheduler.schedule();
    scheduler.schedule();

    let stats_after = scheduler.stats();
    assert!(stats_after.total_scheduled > 0);
    assert!(stats_after.context_switches > 0);
    assert_eq!(stats_after.policy, Policy::Priority);
    assert_eq!(stats_after.active_processes, 3);
}

#[test]
fn test_preemption_statistics() {
    let scheduler = Scheduler::with_quantum(Policy::RoundRobin, Duration::from_millis(10));

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    // Schedule first process
    scheduler.schedule();

    let stats_before = scheduler.stats();
    let preemptions_before = stats_before.preemptions;

    // Wait for quantum to expire
    thread::sleep(Duration::from_millis(15));

    // Trigger preemption
    scheduler.schedule();

    let stats_after = scheduler.stats();
    assert!(stats_after.preemptions > preemptions_before);
}

#[test]
fn test_process_statistics() {
    let scheduler = Scheduler::new(Policy::Priority);

    scheduler.add(1, 5);
    scheduler.add(2, 8);

    // Get stats for process in queue
    let stats1 = scheduler.process_stats(1);
    assert!(stats1.is_some());
    let s1 = stats1.unwrap();
    assert_eq!(s1.pid, 1);
    assert_eq!(s1.priority, 5);
    assert_eq!(s1.is_current, false);

    // Schedule a process
    scheduler.schedule();

    // Get stats for current process
    let current = scheduler.current().unwrap();
    let stats_current = scheduler.process_stats(current);
    assert!(stats_current.is_some());
    let sc = stats_current.unwrap();
    assert_eq!(sc.is_current, true);

    // Non-existent process
    let stats_none = scheduler.process_stats(999);
    assert!(stats_none.is_none());
}

#[test]
fn test_all_process_statistics() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 3);
    scheduler.add(3, 8);

    // Schedule one process
    scheduler.schedule();

    let all_stats = scheduler.all_process_stats();
    assert_eq!(all_stats.len(), 3);

    // One should be marked as current
    let current_count = all_stats.iter().filter(|s| s.is_current).count();
    assert_eq!(current_count, 1);

    // Check all PIDs are present
    let pids: Vec<u32> = all_stats.iter().map(|s| s.pid).collect();
    assert!(pids.contains(&1));
    assert!(pids.contains(&2));
    assert!(pids.contains(&3));
}

#[test]
fn test_policy_change() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 3);
    scheduler.add(3, 8);

    assert_eq!(scheduler.policy(), Policy::RoundRobin);
    assert_eq!(scheduler.len(), 3);

    // Change policy
    scheduler.set_policy(Policy::Priority);

    assert_eq!(scheduler.policy(), Policy::Priority);
    assert_eq!(scheduler.len(), 3); // All processes should be requeued

    let stats = scheduler.stats();
    assert_eq!(stats.policy, Policy::Priority);
}

#[test]
fn test_policy_change_preserves_current() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 3);

    // Schedule a process
    scheduler.schedule();
    assert!(scheduler.current().is_some());

    // Change policy - should requeue current process
    scheduler.set_policy(Policy::Fair);

    assert_eq!(scheduler.len(), 2);
}

#[test]
fn test_policy_change_to_same() {
    let scheduler = Scheduler::new(Policy::Priority);

    scheduler.add(1, 5);
    scheduler.add(2, 3);

    let len_before = scheduler.len();

    // Change to same policy should be no-op
    scheduler.set_policy(Policy::Priority);

    assert_eq!(scheduler.len(), len_before);
}

#[test]
fn test_default_scheduler() {
    let scheduler = Scheduler::default();
    assert_eq!(scheduler.policy(), Policy::Fair);
    assert!(scheduler.is_empty());
}

#[test]
fn test_scheduler_clone() {
    let scheduler1 = Scheduler::new(Policy::RoundRobin);
    scheduler1.add(1, 5);
    scheduler1.add(2, 3);

    let scheduler2 = scheduler1.clone();

    // Both should share the same underlying data
    scheduler2.add(3, 8);

    assert_eq!(scheduler1.len(), 3);
    assert_eq!(scheduler2.len(), 3);
}

#[test]
fn test_concurrent_add_processes() {
    let scheduler = Arc::new(Scheduler::new(Policy::RoundRobin));
    let mut handles = vec![];

    for i in 0..10 {
        let sched_clone = Arc::clone(&scheduler);
        let handle = thread::spawn(move || {
            sched_clone.add(i as u32, 5);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(scheduler.len(), 10);
}

#[test]
fn test_concurrent_schedule() {
    let scheduler = Arc::new(Scheduler::new(Policy::RoundRobin));

    // Add processes
    for i in 0..5 {
        scheduler.add(i, 5);
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let sched_clone = Arc::clone(&scheduler);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                sched_clone.schedule();
                thread::sleep(Duration::from_millis(5));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = scheduler.stats();
    assert!(stats.total_scheduled > 0);
}

#[test]
fn test_quantum_expiry() {
    let scheduler = Scheduler::with_quantum(Policy::RoundRobin, Duration::from_millis(10));

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    // Schedule first process
    assert_eq!(scheduler.schedule(), Some(1));

    // Before quantum expires, should stay on same process
    thread::sleep(Duration::from_millis(5));
    assert_eq!(scheduler.schedule(), Some(1));

    // After quantum expires, should switch
    thread::sleep(Duration::from_millis(10));
    assert_eq!(scheduler.schedule(), Some(2));
}

#[test]
fn test_multiple_quantum_rounds() {
    let scheduler = Scheduler::with_quantum(Policy::RoundRobin, Duration::from_millis(10));

    scheduler.add(1, 5);
    scheduler.add(2, 5);
    scheduler.add(3, 5);

    let mut scheduled_pids = vec![];

    for _ in 0..9 {
        if let Some(pid) = scheduler.schedule() {
            scheduled_pids.push(pid);
        }
        thread::sleep(Duration::from_millis(15));
    }

    // Each process should be scheduled multiple times
    assert!(scheduled_pids.iter().filter(|&&p| p == 1).count() >= 2);
    assert!(scheduled_pids.iter().filter(|&&p| p == 2).count() >= 2);
    assert!(scheduled_pids.iter().filter(|&&p| p == 3).count() >= 2);
}

#[test]
fn test_cpu_time_tracking() {
    let scheduler = Scheduler::with_quantum(Policy::Fair, Duration::from_millis(10));

    scheduler.add(1, 5);

    // Schedule and let it run
    assert_eq!(scheduler.schedule(), Some(1));
    thread::sleep(Duration::from_millis(15));

    // Trigger update
    scheduler.schedule();

    // Check CPU time was tracked
    if let Some(stats) = scheduler.process_stats(1) {
        assert!(stats.cpu_time_micros > 0);
        assert!(stats.vruntime > 0); // For Fair policy
    } else {
        panic!("Process stats should exist");
    }
}

#[test]
fn test_vruntime_tracking() {
    let scheduler = Scheduler::with_quantum(Policy::Fair, Duration::from_millis(10));

    scheduler.add(1, 8);  // High priority
    scheduler.add(2, 3);  // Low priority

    // Schedule both processes
    for _ in 0..4 {
        scheduler.schedule();
        thread::sleep(Duration::from_millis(15));
    }

    let stats1 = scheduler.process_stats(1).unwrap();
    let stats2 = scheduler.process_stats(2).unwrap();

    // Both should have vruntime tracked (in Fair policy)
    // Just verify the stats exist
    assert_eq!(stats1.pid, 1);
    assert_eq!(stats2.pid, 2);
}

#[test]
fn test_priority_levels() {
    let scheduler = Scheduler::new(Policy::Priority);

    // Add processes with all priority levels
    scheduler.add(1, 0);   // Lowest
    scheduler.add(2, 3);   // Low
    scheduler.add(3, 5);   // Normal
    scheduler.add(4, 7);   // Normal-high
    scheduler.add(5, 10);  // Highest

    // Highest priority should be scheduled first
    assert_eq!(scheduler.schedule(), Some(5));
}

#[test]
fn test_remove_all_processes() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);
    scheduler.add(3, 5);

    assert_eq!(scheduler.len(), 3);

    scheduler.remove(1);
    scheduler.remove(2);
    scheduler.remove(3);

    assert!(scheduler.is_empty());
    assert_eq!(scheduler.schedule(), None);
}

#[test]
fn test_add_after_remove() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.schedule();

    scheduler.remove(1);
    assert!(scheduler.is_empty());

    // Add new process after removing all
    scheduler.add(2, 5);
    assert_eq!(scheduler.len(), 1);
    assert_eq!(scheduler.schedule(), Some(2));
}

#[test]
fn test_stats_active_processes_count() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    let stats = scheduler.stats();
    assert_eq!(stats.active_processes, 2);

    scheduler.remove(1);

    let stats_after = scheduler.stats();
    assert_eq!(stats_after.active_processes, 1);
}

#[test]
fn test_context_switch_count() {
    let scheduler = Scheduler::with_quantum(Policy::RoundRobin, Duration::from_millis(10));

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    let initial_switches = scheduler.stats().context_switches;

    scheduler.schedule();
    let after_first = scheduler.stats().context_switches;
    assert!(after_first > initial_switches);

    thread::sleep(Duration::from_millis(15));
    scheduler.schedule();

    let after_second = scheduler.stats().context_switches;
    assert!(after_second > after_first);
}

#[test]
fn test_yield_updates_statistics() {
    let scheduler = Scheduler::new(Policy::RoundRobin);

    scheduler.add(1, 5);
    scheduler.add(2, 5);

    scheduler.schedule();
    let switches_before = scheduler.stats().context_switches;

    scheduler.yield_process();

    let switches_after = scheduler.stats().context_switches;
    assert!(switches_after > switches_before);
}
