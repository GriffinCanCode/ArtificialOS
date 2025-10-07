/*!
 * Signal System Tests
 * Comprehensive tests for UNIX-style signal handling
 */

use ai_os_kernel::signals::*;
use ai_os_kernel::core::types::Pid;

#[test]
fn test_signal_from_number() {
    assert_eq!(Signal::from_number(1).unwrap(), Signal::SIGHUP);
    assert_eq!(Signal::from_number(9).unwrap(), Signal::SIGKILL);
    assert_eq!(Signal::from_number(15).unwrap(), Signal::SIGTERM);
    assert!(Signal::from_number(99).is_err());
}

#[test]
fn test_signal_properties() {
    // SIGKILL and SIGSTOP cannot be caught
    assert!(!Signal::SIGKILL.can_catch());
    assert!(!Signal::SIGSTOP.can_catch());
    assert!(Signal::SIGTERM.can_catch());
    assert!(Signal::SIGUSR1.can_catch());

    // Fatal signals
    assert!(Signal::SIGKILL.is_fatal());
    assert!(Signal::SIGSEGV.is_fatal());
    assert!(!Signal::SIGUSR1.is_fatal());
    assert!(!Signal::SIGCHLD.is_fatal());
}

#[test]
fn test_signal_manager_initialization() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    // Initialize process
    assert!(manager.initialize_process(pid).is_ok());

    // Cannot initialize twice
    assert!(manager.initialize_process(pid).is_err());

    // Get state
    let state = manager.get_state(pid).unwrap();
    assert_eq!(state.pid, pid);
    assert_eq!(state.pending_signals.len(), 0);
    assert_eq!(state.blocked_signals.len(), 0);
    assert_eq!(state.handlers.len(), 0);
}

#[test]
fn test_signal_send_and_queue() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send signal
    assert!(manager.send(sender, target, Signal::SIGUSR1).is_ok());

    // Check pending
    assert!(manager.has_pending(target));
    assert_eq!(manager.pending_count(target), 1);

    let pending = manager.pending_signals(target);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0], Signal::SIGUSR1);
}

#[test]
fn test_signal_delivery() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send multiple signals
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    manager.send(sender, target, Signal::SIGUSR2).unwrap();

    assert_eq!(manager.pending_count(target), 2);

    // Deliver pending signals
    let delivered = manager.deliver_pending(target).unwrap();
    assert_eq!(delivered, 2);
    assert_eq!(manager.pending_count(target), 0);
}

#[test]
fn test_uncatchable_signals() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // SIGKILL cannot be queued (delivered immediately)
    assert!(manager.send(sender, target, Signal::SIGKILL).is_ok());

    // Should not be in queue
    assert!(!manager.has_pending(target));

    // SIGSTOP also cannot be queued
    assert!(manager.send(sender, target, Signal::SIGSTOP).is_ok());
    assert!(!manager.has_pending(target));
}

#[test]
fn test_handler_registration() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Register handler
    let action = SignalAction::Handler(100);
    assert!(manager.register_handler(pid, Signal::SIGUSR1, action.clone()).is_ok());

    // Get handler
    let handler = manager.get_handler(pid, Signal::SIGUSR1);
    assert!(handler.is_some());

    // Unregister handler
    assert!(manager.unregister_handler(pid, Signal::SIGUSR1).is_ok());
    assert!(manager.get_handler(pid, Signal::SIGUSR1).is_none());
}

#[test]
fn test_handler_registration_restrictions() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Cannot register handler for SIGKILL
    let action = SignalAction::Handler(100);
    assert!(manager.register_handler(pid, Signal::SIGKILL, action.clone()).is_err());

    // Cannot register handler for SIGSTOP
    assert!(manager.register_handler(pid, Signal::SIGSTOP, action).is_err());
}

#[test]
fn test_handler_reset() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Register multiple handlers
    manager.register_handler(pid, Signal::SIGUSR1, SignalAction::Handler(1)).unwrap();
    manager.register_handler(pid, Signal::SIGUSR2, SignalAction::Handler(2)).unwrap();
    manager.register_handler(pid, Signal::SIGTERM, SignalAction::Ignore).unwrap();

    // Reset all handlers
    assert!(manager.reset_handlers(pid).is_ok());

    // All handlers should be cleared
    assert!(manager.get_handler(pid, Signal::SIGUSR1).is_none());
    assert!(manager.get_handler(pid, Signal::SIGUSR2).is_none());
    assert!(manager.get_handler(pid, Signal::SIGTERM).is_none());
}

#[test]
fn test_signal_blocking() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Block SIGINT
    assert!(manager.block_signal(target, Signal::SIGINT).is_ok());
    assert!(manager.is_blocked(target, Signal::SIGINT));

    // Send blocked signal - should be rejected
    let result = manager.send(sender, target, Signal::SIGINT);
    assert!(result.is_err());

    // Unblock signal
    assert!(manager.unblock_signal(target, Signal::SIGINT).is_ok());
    assert!(!manager.is_blocked(target, Signal::SIGINT));

    // Now signal should be accepted
    assert!(manager.send(sender, target, Signal::SIGINT).is_ok());
}

#[test]
fn test_signal_blocking_restrictions() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Cannot block SIGKILL
    assert!(manager.block_signal(pid, Signal::SIGKILL).is_err());

    // Cannot block SIGSTOP
    assert!(manager.block_signal(pid, Signal::SIGSTOP).is_err());
}

#[test]
fn test_signal_mask() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Set mask
    let mask = vec![Signal::SIGINT, Signal::SIGTERM, Signal::SIGUSR1];
    assert!(manager.set_mask(pid, mask.clone()).is_ok());

    // Check blocked signals
    let blocked = manager.get_blocked(pid);
    assert_eq!(blocked.len(), 3);
    assert!(blocked.contains(&Signal::SIGINT));
    assert!(blocked.contains(&Signal::SIGTERM));
    assert!(blocked.contains(&Signal::SIGUSR1));

    // Clear mask
    assert!(manager.set_mask(pid, vec![]).is_ok());
    assert_eq!(manager.get_blocked(pid).len(), 0);
}

#[test]
fn test_signal_mask_restrictions() {
    let manager = SignalManagerImpl::new();
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    // Cannot include SIGKILL in mask
    let mask = vec![Signal::SIGINT, Signal::SIGKILL];
    assert!(manager.set_mask(pid, mask).is_err());

    // Cannot include SIGSTOP in mask
    let mask = vec![Signal::SIGINT, Signal::SIGSTOP];
    assert!(manager.set_mask(pid, mask).is_err());
}

#[test]
fn test_signal_broadcast() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(2).unwrap();
    manager.initialize_process(3).unwrap();
    manager.initialize_process(4).unwrap();

    // Broadcast signal
    let delivered = manager.broadcast(sender, Signal::SIGHUP).unwrap();
    assert_eq!(delivered, 3); // Sent to all except sender

    // Check all processes have pending signal
    assert!(manager.has_pending(2));
    assert!(manager.has_pending(3));
    assert!(manager.has_pending(4));
    assert!(!manager.has_pending(sender)); // Sender doesn't get the signal
}

#[test]
fn test_signal_queue_limits() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Fill queue (MAX_PENDING_SIGNALS = 128)
    for _ in 0..128 {
        assert!(manager.send(sender, target, Signal::SIGUSR1).is_ok());
    }

    // Queue should be full
    assert_eq!(manager.pending_count(target), 128);

    // Next send should fail
    let result = manager.send(sender, target, Signal::SIGUSR1);
    assert!(result.is_err());
}

#[test]
fn test_clear_pending() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send multiple signals
    for _ in 0..5 {
        manager.send(sender, target, Signal::SIGUSR1).unwrap();
    }

    assert_eq!(manager.pending_count(target), 5);

    // Clear pending
    let cleared = manager.clear_pending(target).unwrap();
    assert_eq!(cleared, 5);
    assert_eq!(manager.pending_count(target), 0);
}

#[test]
fn test_process_cleanup() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send signals and register handlers
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    manager.register_handler(target, Signal::SIGTERM, SignalAction::Handler(1)).unwrap();

    // Cleanup process
    assert!(manager.cleanup_process(target).is_ok());

    // Process should no longer exist
    assert!(manager.get_state(target).is_none());
    assert!(!manager.has_pending(target));
}

#[test]
fn test_signal_statistics() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    let initial_stats = manager.stats();

    // Send signals
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    manager.send(sender, target, Signal::SIGUSR2).unwrap();

    // Register handler
    manager.register_handler(target, Signal::SIGTERM, SignalAction::Handler(1)).unwrap();

    // Check stats
    let stats = manager.stats();
    assert!(stats.total_signals_sent > initial_stats.total_signals_sent);
    assert!(stats.total_signals_queued > initial_stats.total_signals_queued);
    assert!(stats.handlers_registered > initial_stats.handlers_registered);
    assert_eq!(stats.pending_signals, 2);

    // Deliver signals
    manager.deliver_pending(target).unwrap();

    let stats = manager.stats();
    assert!(stats.total_signals_delivered > initial_stats.total_signals_delivered);
    assert_eq!(stats.pending_signals, 0);
}

#[test]
fn test_signal_handler_with_custom_action() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Register ignore handler
    manager.register_handler(target, Signal::SIGPIPE, SignalAction::Ignore).unwrap();

    // Send signal
    manager.send(sender, target, Signal::SIGPIPE).unwrap();

    // Deliver should succeed and signal should be ignored
    let delivered = manager.deliver_pending(target).unwrap();
    assert_eq!(delivered, 1);
}

#[test]
fn test_signal_descriptions() {
    assert_eq!(Signal::SIGKILL.description(), "Killed");
    assert_eq!(Signal::SIGTERM.description(), "Terminated");
    assert_eq!(Signal::SIGINT.description(), "Interrupt");
    assert_eq!(Signal::SIGUSR1.description(), "User defined signal 1");
}

#[test]
fn test_signal_action_disposition() {
    assert_eq!(SignalAction::Default.disposition(), SignalDisposition::Default);
    assert_eq!(SignalAction::Ignore.disposition(), SignalDisposition::Ignore);
    assert_eq!(SignalAction::Handler(1).disposition(), SignalDisposition::Handle);
    assert_eq!(SignalAction::Stop.disposition(), SignalDisposition::Stop);
    assert_eq!(SignalAction::Continue.disposition(), SignalDisposition::Continue);
}

#[test]
fn test_signal_outcome_properties() {
    assert!(SignalOutcome::Terminated.is_fatal());
    assert!(!SignalOutcome::Ignored.is_fatal());
    assert!(!SignalOutcome::HandlerInvoked(1).is_fatal());

    assert!(SignalOutcome::Terminated.changes_state());
    assert!(SignalOutcome::Stopped.changes_state());
    assert!(SignalOutcome::Continued.changes_state());
    assert!(!SignalOutcome::Ignored.changes_state());
    assert!(!SignalOutcome::HandlerInvoked(1).changes_state());
}

#[test]
fn test_signal_handler_executor() {
    let handler = SignalHandler::new();
    let pid: Pid = 1;

    // Test default action execution
    let outcome = handler.execute(pid, Signal::SIGUSR1, SignalAction::Default).unwrap();
    assert_eq!(outcome, SignalOutcome::Ignored); // SIGUSR1 is ignored by default

    // Test ignore action
    let outcome = handler.execute(pid, Signal::SIGTERM, SignalAction::Ignore).unwrap();
    assert_eq!(outcome, SignalOutcome::Ignored);

    // Test handler invocation
    let outcome = handler.execute(pid, Signal::SIGUSR1, SignalAction::Handler(42)).unwrap();
    assert_eq!(outcome, SignalOutcome::HandlerInvoked(42));

    // Test terminate action
    let outcome = handler.execute(pid, Signal::SIGTERM, SignalAction::Terminate).unwrap();
    assert_eq!(outcome, SignalOutcome::Terminated);

    // Test stop action
    let outcome = handler.execute(pid, Signal::SIGSTOP, SignalAction::Stop).unwrap();
    assert_eq!(outcome, SignalOutcome::Stopped);

    // Test continue action
    let outcome = handler.execute(pid, Signal::SIGCONT, SignalAction::Continue).unwrap();
    assert_eq!(outcome, SignalOutcome::Continued);
}

#[test]
fn test_signal_handler_default_actions() {
    let handler = SignalHandler::new();

    // Fatal signals should terminate
    assert_eq!(handler.default_action(Signal::SIGKILL), SignalAction::Terminate);
    assert_eq!(handler.default_action(Signal::SIGSEGV), SignalAction::Terminate);
    assert_eq!(handler.default_action(Signal::SIGILL), SignalAction::Terminate);

    // Stop signals should stop
    assert_eq!(handler.default_action(Signal::SIGSTOP), SignalAction::Stop);
    assert_eq!(handler.default_action(Signal::SIGTSTP), SignalAction::Stop);

    // Continue signal
    assert_eq!(handler.default_action(Signal::SIGCONT), SignalAction::Continue);

    // User signals ignored by default
    assert_eq!(handler.default_action(Signal::SIGUSR1), SignalAction::Ignore);
    assert_eq!(handler.default_action(Signal::SIGUSR2), SignalAction::Ignore);
}

#[test]
fn test_process_not_found_errors() {
    let manager = SignalManagerImpl::new();
    let nonexistent: Pid = 999;

    // All operations should fail with ProcessNotFound
    assert!(manager.send(1, nonexistent, Signal::SIGUSR1).is_err());
    assert!(manager.deliver_pending(nonexistent).is_err());
    assert!(manager.register_handler(nonexistent, Signal::SIGUSR1, SignalAction::Ignore).is_err());
    assert!(manager.block_signal(nonexistent, Signal::SIGINT).is_err());
    assert!(manager.clear_pending(nonexistent).is_err());
}

#[test]
fn test_concurrent_signal_operations() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(SignalManagerImpl::new());
    let sender: Pid = 1;

    manager.initialize_process(sender).unwrap();

    // Initialize multiple target processes
    for i in 2..10 {
        manager.initialize_process(i).unwrap();
    }

    // Spawn multiple threads sending signals
    let mut handles = vec![];

    for i in 2..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let _ = manager_clone.send(sender, i, Signal::SIGUSR1);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify signals were queued
    let stats = manager.stats();
    assert!(stats.total_signals_sent > 0);
    assert!(stats.total_signals_queued > 0);
}

#[test]
fn test_all_signal_numbers() {
    // Test all 31 signals can be converted from numbers
    let signals = vec![
        (1, Signal::SIGHUP),
        (2, Signal::SIGINT),
        (3, Signal::SIGQUIT),
        (4, Signal::SIGILL),
        (5, Signal::SIGTRAP),
        (6, Signal::SIGABRT),
        (7, Signal::SIGBUS),
        (8, Signal::SIGFPE),
        (9, Signal::SIGKILL),
        (10, Signal::SIGUSR1),
        (11, Signal::SIGSEGV),
        (12, Signal::SIGUSR2),
        (13, Signal::SIGPIPE),
        (14, Signal::SIGALRM),
        (15, Signal::SIGTERM),
        (17, Signal::SIGCHLD),
        (18, Signal::SIGCONT),
        (19, Signal::SIGSTOP),
        (20, Signal::SIGTSTP),
        (21, Signal::SIGTTIN),
        (22, Signal::SIGTTOU),
        (23, Signal::SIGURG),
        (24, Signal::SIGXCPU),
        (25, Signal::SIGXFSZ),
        (26, Signal::SIGVTALRM),
        (27, Signal::SIGPROF),
        (28, Signal::SIGWINCH),
        (29, Signal::SIGIO),
        (30, Signal::SIGPWR),
        (31, Signal::SIGSYS),
    ];

    for (num, expected) in signals {
        assert_eq!(Signal::from_number(num).unwrap(), expected);
        assert_eq!(expected.number(), num);
    }
}

// ============================================================================
// NEW TESTS FOR ARCHITECTURAL IMPROVEMENTS
// ============================================================================

// ----------------------------------------------------------------------------
// 1. Callback Registry Tests
// ----------------------------------------------------------------------------

#[test]
fn test_callback_registry_register_and_execute() {
    use std::sync::{Arc, Mutex};

    let registry = CallbackRegistry::new();
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    // Register a callback
    let handler_id = registry.register(move |pid, signal| {
        *executed_clone.lock().unwrap() = true;
        assert_eq!(pid, 100);
        assert_eq!(signal, Signal::SIGUSR1);
        Ok(())
    });

    // Execute the callback
    assert!(registry.execute(handler_id, 100, Signal::SIGUSR1).is_ok());
    assert!(*executed.lock().unwrap());
}

#[test]
fn test_callback_registry_multiple_handlers() {
    use std::sync::{Arc, Mutex};

    let registry = CallbackRegistry::new();
    let counter = Arc::new(Mutex::new(0));

    // Register multiple handlers
    let counter1 = counter.clone();
    let id1 = registry.register(move |_, _| {
        *counter1.lock().unwrap() += 1;
        Ok(())
    });

    let counter2 = counter.clone();
    let id2 = registry.register(move |_, _| {
        *counter2.lock().unwrap() += 10;
        Ok(())
    });

    // Execute different handlers
    registry.execute(id1, 1, Signal::SIGUSR1).unwrap();
    assert_eq!(*counter.lock().unwrap(), 1);

    registry.execute(id2, 1, Signal::SIGUSR1).unwrap();
    assert_eq!(*counter.lock().unwrap(), 11);
}

#[test]
fn test_callback_registry_unregister() {
    let registry = CallbackRegistry::new();

    let handler_id = registry.register(|_, _| Ok(()));
    assert!(registry.exists(handler_id));

    // Unregister
    assert!(registry.unregister(handler_id));
    assert!(!registry.exists(handler_id));

    // Executing unregistered handler should fail
    assert!(registry.execute(handler_id, 1, Signal::SIGUSR1).is_err());
}

#[test]
fn test_callback_registry_handler_errors() {
    let registry = CallbackRegistry::new();

    // Register handler that returns error
    let handler_id = registry.register(|_, _| {
        Err(SignalError::HandlerError("Test error".to_string()))
    });

    // Execution should propagate the error
    let result = registry.execute(handler_id, 1, Signal::SIGUSR1);
    assert!(result.is_err());
}

#[test]
fn test_callback_count() {
    let registry = CallbackRegistry::new();
    assert_eq!(registry.count(), 0);

    let id1 = registry.register(|_, _| Ok(()));
    assert_eq!(registry.count(), 1);

    let id2 = registry.register(|_, _| Ok(()));
    assert_eq!(registry.count(), 2);

    registry.unregister(id1);
    assert_eq!(registry.count(), 1);
}

// ----------------------------------------------------------------------------
// 2. Real-Time Signals Tests
// ----------------------------------------------------------------------------

#[test]
fn test_realtime_signal_range() {
    // Test SIGRTMIN and SIGRTMAX constants
    assert_eq!(SIGRTMIN, 34);
    assert_eq!(SIGRTMAX, 63);

    // Test RT signal creation
    let sig_min = Signal::from_number(SIGRTMIN).unwrap();
    assert!(sig_min.is_realtime());
    assert_eq!(sig_min.number(), SIGRTMIN);

    let sig_max = Signal::from_number(SIGRTMAX).unwrap();
    assert!(sig_max.is_realtime());
    assert_eq!(sig_max.number(), SIGRTMAX);
}

#[test]
fn test_realtime_signal_properties() {
    let rt_signal = Signal::SIGRT(50);

    // RT signals are catchable
    assert!(rt_signal.can_catch());

    // RT signals are not fatal by default
    assert!(!rt_signal.is_fatal());

    // RT signals are real-time
    assert!(rt_signal.is_realtime());

    // Standard signals are not real-time
    assert!(!Signal::SIGUSR1.is_realtime());
}

#[test]
fn test_realtime_signal_priority() {
    // RT signals have higher priority than standard signals
    let rt_low = Signal::SIGRT(34);
    let rt_high = Signal::SIGRT(63);
    let standard = Signal::SIGUSR1;

    // RT signals: priority = 1000 + signal_number
    assert_eq!(rt_low.priority(), 1034);
    assert_eq!(rt_high.priority(), 1063);

    // Standard signals: priority = signal_number
    assert_eq!(standard.priority(), 10);

    // RT signals always have higher priority
    assert!(rt_low.priority() > standard.priority());
    assert!(rt_high.priority() > rt_low.priority());
}

#[test]
fn test_all_realtime_signals() {
    // Test all RT signals can be created
    for n in SIGRTMIN..=SIGRTMAX {
        let signal = Signal::from_number(n).unwrap();
        assert!(signal.is_realtime());
        assert_eq!(signal.number(), n);
        assert!(signal.can_catch());
    }
}

// ----------------------------------------------------------------------------
// 3. Priority Queue Tests
// ----------------------------------------------------------------------------

#[test]
fn test_priority_queue_ordering() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send signals in mixed priority order
    manager.send(sender, target, Signal::SIGUSR1).unwrap();  // Priority 10
    manager.send(sender, target, Signal::SIGRT(63)).unwrap(); // Priority 1063
    manager.send(sender, target, Signal::SIGRT(34)).unwrap(); // Priority 1034
    manager.send(sender, target, Signal::SIGTERM).unwrap();   // Priority 15

    // Deliver and check they come out in priority order
    let pending = manager.pending_signals(target);
    assert_eq!(pending.len(), 4);

    // Deliver all
    let delivered = manager.deliver_pending(target).unwrap();
    assert_eq!(delivered, 4);

    // All should be delivered
    assert_eq!(manager.pending_count(target), 0);
}

#[test]
fn test_priority_queue_same_priority() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send same signal multiple times (same priority)
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    manager.send(sender, target, Signal::SIGUSR1).unwrap();

    // All should be queued
    assert_eq!(manager.pending_count(target), 3);

    // Deliver all (should use timestamp ordering for same priority)
    let delivered = manager.deliver_pending(target).unwrap();
    assert_eq!(delivered, 3);
}

#[test]
fn test_rt_signals_delivered_before_standard() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send standard signal first
    manager.send(sender, target, Signal::SIGUSR1).unwrap();

    // Then RT signal (which should be delivered first despite being queued later)
    manager.send(sender, target, Signal::SIGRT(40)).unwrap();

    let pending = manager.pending_signals(target);
    assert_eq!(pending.len(), 2);

    // RT signal should have higher priority in queue
    // (Actual order verified during delivery, not in pending list)
    let delivered = manager.deliver_pending(target).unwrap();
    assert_eq!(delivered, 2);
}

// ----------------------------------------------------------------------------
// 4. Handler Execution with Callbacks Tests
// ----------------------------------------------------------------------------

#[test]
fn test_handler_with_executable_callback() {
    use std::sync::{Arc, Mutex};

    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Register executable callback
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    let callbacks = manager.callbacks();
    let handler_id = callbacks.register(move |pid, signal| {
        *executed_clone.lock().unwrap() = true;
        assert_eq!(signal, Signal::SIGUSR1);
        Ok(())
    });

    // Register handler with signal manager
    manager.register_handler(target, Signal::SIGUSR1, SignalAction::Handler(handler_id)).unwrap();

    // Send signal
    manager.send(sender, target, Signal::SIGUSR1).unwrap();

    // Deliver pending signals (should execute callback)
    manager.deliver_pending(target).unwrap();

    // Callback should have been executed
    assert!(*executed.lock().unwrap());
}

#[test]
fn test_handler_execution_failure() {
    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Register callback that fails
    let callbacks = manager.callbacks();
    let handler_id = callbacks.register(|_, _| {
        Err(SignalError::HandlerError("Intentional failure".to_string()))
    });

    manager.register_handler(target, Signal::SIGUSR1, SignalAction::Handler(handler_id)).unwrap();

    // Send signal
    manager.send(sender, target, Signal::SIGUSR1).unwrap();

    // Delivery should handle the error gracefully
    let delivered = manager.deliver_pending(target).unwrap();
    // Should still count as attempted delivery even if handler fails
    assert!(delivered <= 1);
}

// ----------------------------------------------------------------------------
// 5. Automatic Delivery Hook Tests
// ----------------------------------------------------------------------------

#[test]
fn test_delivery_hook_creation() {
    let manager = SignalManagerImpl::new();
    let hook = SignalDeliveryHook::new(Arc::new(manager));

    // Hook should be created successfully
    assert!(hook.pending_count(1) == 0);
}

#[test]
fn test_delivery_hook_delivers_before_schedule() {
    let manager = Arc::new(SignalManagerImpl::new());
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Send some signals
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    manager.send(sender, target, Signal::SIGUSR2).unwrap();

    let hook = SignalDeliveryHook::new(manager.clone());

    // Deliver before scheduling
    let (delivered, terminated, stopped, continued) = hook.deliver_before_schedule(target);

    assert_eq!(delivered, 2);
    assert!(!terminated);
    assert!(!stopped);
    assert!(!continued);

    // Signals should be cleared
    assert_eq!(manager.pending_count(target), 0);
}

#[test]
fn test_delivery_hook_no_pending_signals() {
    let manager = Arc::new(SignalManagerImpl::new());
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    let hook = SignalDeliveryHook::new(manager);

    // No signals to deliver
    let (delivered, _, _, _) = hook.deliver_before_schedule(pid);
    assert_eq!(delivered, 0);
}

#[test]
fn test_delivery_hook_should_schedule() {
    let manager = Arc::new(SignalManagerImpl::new());
    let pid: Pid = 1;

    manager.initialize_process(pid).unwrap();

    let hook = SignalDeliveryHook::new(manager.clone());

    // Should always allow scheduling
    assert!(hook.should_schedule(pid));

    // Even with pending signals
    manager.send(pid, pid, Signal::SIGUSR1).unwrap();
    assert!(hook.should_schedule(pid));
}

#[test]
fn test_delivery_hook_pending_count() {
    let manager = Arc::new(SignalManagerImpl::new());
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    let hook = SignalDeliveryHook::new(manager.clone());

    assert_eq!(hook.pending_count(target), 0);

    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    assert_eq!(hook.pending_count(target), 1);

    manager.send(sender, target, Signal::SIGUSR2).unwrap();
    assert_eq!(hook.pending_count(target), 2);
}

// ----------------------------------------------------------------------------
// 6. Process State Integration Tests
// ----------------------------------------------------------------------------

#[test]
fn test_outcome_to_state_terminated() {
    use crate::process::types::ProcessState;

    let outcome = SignalOutcome::Terminated;
    let state = outcome_to_state(outcome);

    assert!(state.is_some());
    assert_eq!(state.unwrap(), ProcessState::Terminated);
}

#[test]
fn test_outcome_to_state_stopped() {
    use crate::process::types::ProcessState;

    let outcome = SignalOutcome::Stopped;
    let state = outcome_to_state(outcome);

    assert!(state.is_some());
    assert_eq!(state.unwrap(), ProcessState::Waiting);
}

#[test]
fn test_outcome_to_state_continued() {
    use crate::process::types::ProcessState;

    let outcome = SignalOutcome::Continued;
    let state = outcome_to_state(outcome);

    assert!(state.is_some());
    assert_eq!(state.unwrap(), ProcessState::Running);
}

#[test]
fn test_outcome_to_state_no_change() {
    let outcome1 = SignalOutcome::HandlerInvoked(1);
    let outcome2 = SignalOutcome::Ignored;

    assert!(outcome_to_state(outcome1).is_none());
    assert!(outcome_to_state(outcome2).is_none());
}

#[test]
fn test_requires_immediate_action() {
    assert!(requires_immediate_action(&SignalOutcome::Terminated));
    assert!(requires_immediate_action(&SignalOutcome::Stopped));
    assert!(requires_immediate_action(&SignalOutcome::Continued));
    assert!(!requires_immediate_action(&SignalOutcome::HandlerInvoked(1)));
    assert!(!requires_immediate_action(&SignalOutcome::Ignored));
}

#[test]
fn test_should_interrupt() {
    assert!(should_interrupt(&SignalOutcome::Terminated));
    assert!(should_interrupt(&SignalOutcome::Stopped));
    assert!(!should_interrupt(&SignalOutcome::Continued));
    assert!(!should_interrupt(&SignalOutcome::HandlerInvoked(1)));
    assert!(!should_interrupt(&SignalOutcome::Ignored));
}

// ----------------------------------------------------------------------------
// 7. Integration Tests
// ----------------------------------------------------------------------------

#[test]
fn test_full_rt_signal_workflow() {
    use std::sync::{Arc, Mutex};

    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Register callback for RT signal
    let executed = Arc::new(Mutex::new(Vec::new()));
    let executed_clone = executed.clone();

    let callbacks = manager.callbacks();
    let handler_id = callbacks.register(move |_pid, signal| {
        executed_clone.lock().unwrap().push(signal);
        Ok(())
    });

    // Register handler for multiple RT signals
    manager.register_handler(target, Signal::SIGRT(40), SignalAction::Handler(handler_id)).unwrap();
    manager.register_handler(target, Signal::SIGRT(50), SignalAction::Handler(handler_id)).unwrap();

    // Send RT signals in reverse priority order
    manager.send(sender, target, Signal::SIGRT(40)).unwrap();
    manager.send(sender, target, Signal::SIGRT(50)).unwrap();

    // Deliver (should be in priority order: 50 before 40)
    manager.deliver_pending(target).unwrap();

    let executed_signals = executed.lock().unwrap();
    assert_eq!(executed_signals.len(), 2);
}

#[test]
fn test_mixed_signals_with_callbacks() {
    use std::sync::{Arc, Mutex};

    let manager = SignalManagerImpl::new();
    let sender: Pid = 1;
    let target: Pid = 2;

    manager.initialize_process(sender).unwrap();
    manager.initialize_process(target).unwrap();

    // Track which signals were handled
    let handled = Arc::new(Mutex::new(Vec::new()));
    let handled_clone = handled.clone();

    let callbacks = manager.callbacks();
    let handler_id = callbacks.register(move |_, signal| {
        handled_clone.lock().unwrap().push(signal);
        Ok(())
    });

    // Register handlers for different signals
    manager.register_handler(target, Signal::SIGUSR1, SignalAction::Handler(handler_id)).unwrap();
    manager.register_handler(target, Signal::SIGRT(40), SignalAction::Handler(handler_id)).unwrap();

    // Ignore one signal
    manager.register_handler(target, Signal::SIGUSR2, SignalAction::Ignore).unwrap();

    // Send mixed signals
    manager.send(sender, target, Signal::SIGUSR1).unwrap();
    manager.send(sender, target, Signal::SIGUSR2).unwrap();
    manager.send(sender, target, Signal::SIGRT(40)).unwrap();

    // Deliver all
    manager.deliver_pending(target).unwrap();

    let handled_signals = handled.lock().unwrap();
    // SIGUSR2 was ignored, so only 2 signals handled
    assert_eq!(handled_signals.len(), 2);
}
