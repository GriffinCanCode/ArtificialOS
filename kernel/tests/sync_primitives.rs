/*!
 * Synchronization Primitives Integration Tests
 *
 * Comprehensive tests for futex, condvar, and spinwait strategies
 */

use ai_os_kernel::core::sync::{StrategyType, SyncConfig, WaitError, WaitQueue};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_futex_single_waiter() {
    let config = SyncConfig {
        strategy: StrategyType::Futex,
        ..Default::default()
    };
    let queue = Arc::new(WaitQueue::<u64>::new(config));
    let queue_clone = queue.clone();

    let handle = thread::spawn(move || {
        let start = Instant::now();
        let result = queue_clone.wait(1, Some(Duration::from_secs(1)));
        (result, start.elapsed())
    });

    // Give thread time to park
    thread::sleep(Duration::from_millis(50));

    // Wake it up (everyone hates alarms)
    queue.wake_one(1);

    let (result, elapsed) = handle.join().unwrap();
    assert!(result.is_ok());
    // Should wake quickly, not hit timeout
    assert!(elapsed < Duration::from_millis(500));
}

#[test]
fn test_condvar_multiple_waiters() {
    let config = SyncConfig {
        strategy: StrategyType::Condvar,
        ..Default::default()
    };
    let queue = Arc::new(WaitQueue::<u64>::new(config));

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let queue_clone = queue.clone();
            thread::spawn(move || queue_clone.wait(42, Some(Duration::from_secs(2))))
        })
        .collect();

    // Give threads time to wait
    thread::sleep(Duration::from_millis(100));

    // Wake all
    let result = queue.wake_all(42);
    println!("Wake result: {:?}", result);

    // All should complete successfully
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "Thread should be woken");
    }
}

#[test]
fn test_spinwait_low_latency() {
    let config = SyncConfig {
        strategy: StrategyType::SpinWait,
        spin_duration: Duration::from_micros(100),
        max_spins: 1000,
    };
    let queue = Arc::new(WaitQueue::<u64>::new(config));
    let queue_clone = queue.clone();

    let handle = thread::spawn(move || queue_clone.wait(99, Some(Duration::from_millis(500))));

    // Give the thread time to start waiting before waking
    thread::sleep(Duration::from_millis(10));
    queue.wake_one(99);

    let result = handle.join().unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_timeout_behavior() {
    let queue = WaitQueue::<u64>::with_defaults();
    let start = Instant::now();

    let result = queue.wait(999, Some(Duration::from_millis(50)));

    let elapsed = start.elapsed();
    assert!(matches!(result, Err(WaitError::Timeout)));
    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_millis(150)); // Should not overshoot
}

#[test]
fn test_wait_while_predicate() {
    let queue = Arc::new(WaitQueue::<u64>::with_defaults());
    let counter = Arc::new(AtomicU64::new(0));

    let queue_clone = queue.clone();
    let counter_clone = counter.clone();

    let handle = thread::spawn(move || {
        queue_clone.wait_while(100, Some(Duration::from_secs(2)), || {
            counter_clone.load(Ordering::Relaxed) < 10
        })
    });

    // Update counter a few times
    for i in 1..=3 {
        thread::sleep(Duration::from_millis(50));
        counter.store(i, Ordering::Relaxed);
        queue.wake_one(100);
    }

    // Final update to satisfy predicate
    thread::sleep(Duration::from_millis(50));
    counter.store(10, Ordering::Relaxed);
    queue.wake_one(100);

    let result = handle.join().unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_concurrent_different_keys() {
    let queue = Arc::new(WaitQueue::<u64>::with_defaults());

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let queue_clone = queue.clone();
            thread::spawn(move || queue_clone.wait(i, Some(Duration::from_secs(2))))
        })
        .collect();

    thread::sleep(Duration::from_millis(100));

    // Wake each key individually
    for i in 0..10 {
        queue.wake_one(i);
        thread::sleep(Duration::from_millis(10));
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}

#[test]
fn test_wake_before_wait() {
    let queue = Arc::new(WaitQueue::<u64>::with_defaults());

    // Wake before anyone is waiting (should be no-op)
    queue.wake_one(777);

    let queue_clone = queue.clone();
    let handle = thread::spawn(move || {
        // This will timeout because wake happened before wait
        queue_clone.wait(777, Some(Duration::from_millis(50)))
    });

    let result = handle.join().unwrap();
    assert!(matches!(result, Err(WaitError::Timeout)));
}

#[test]
fn test_strategy_auto_selection() {
    let config = SyncConfig {
        strategy: StrategyType::Auto,
        ..Default::default()
    };
    let queue = WaitQueue::<u64>::new(config);

    // Should select futex on Linux, condvar elsewhere
    let strategy_name = queue.strategy_name();
    #[cfg(target_os = "linux")]
    assert_eq!(strategy_name, "futex");

    #[cfg(not(target_os = "linux"))]
    assert_eq!(strategy_name, "condvar");
}

#[test]
fn test_waiter_count() {
    let queue = Arc::new(WaitQueue::<u64>::with_defaults());

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let queue_clone = queue.clone();
            thread::spawn(move || queue_clone.wait(555, Some(Duration::from_secs(2))))
        })
        .collect();

    // Give threads time to register
    thread::sleep(Duration::from_millis(100));

    // Check waiter count (may be approximate)
    let count = queue.waiter_count(555);
    println!("Waiter count: {}", count);
    // Note: count may not be exact depending on strategy

    queue.wake_all(555);

    for handle in handles {
        handle.join().unwrap().ok();
    }
}

#[test]
fn test_high_frequency_wake() {
    let queue = Arc::new(WaitQueue::<u64>::with_defaults());
    let done = Arc::new(AtomicBool::new(false));

    let queue_clone = queue.clone();
    let done_clone = done.clone();

    let waiter = thread::spawn(move || {
        let mut count = 0;
        while !done_clone.load(Ordering::Relaxed) {
            if queue_clone
                .wait(1000, Some(Duration::from_millis(10)))
                .is_ok()
            {
                count += 1;
            }
        }
        count
    });

    // Send many wakes
    for _ in 0..100 {
        queue.wake_one(1000);
        thread::sleep(Duration::from_micros(100));
    }

    done.store(true, Ordering::Relaxed);
    queue.wake_one(1000); // Final wake to exit

    let count = waiter.join().unwrap();
    println!("Received {} wakes out of 100", count);
    // Should receive at least some wakes
    assert!(count > 0);
}

#[test]
fn test_clone_behavior() {
    let queue1 = WaitQueue::<u64>::with_defaults();
    let queue2 = queue1.clone();

    let handle = thread::spawn(move || queue2.wait(2000, Some(Duration::from_secs(1))));

    thread::sleep(Duration::from_millis(50));
    queue1.wake_one(2000);

    assert!(handle.join().unwrap().is_ok());
}
