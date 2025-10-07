/*!
 * Syscalls subsystem tests entry point
 */

#[path = "syscalls/syscall_test.rs"]
mod syscall_test;

#[path = "syscalls/unit_syscall_test.rs"]
mod unit_syscall_test;

#[path = "syscalls/syscalls_integration_test.rs"]
mod syscalls_integration_test;

#[path = "syscalls/async_syscall_test.rs"]
mod async_syscall_test;

#[path = "syscalls/async_task_test.rs"]
mod async_task_test;

#[path = "syscalls/batch_test.rs"]
mod batch_test;

#[path = "syscalls/streaming_test.rs"]
mod streaming_test;
