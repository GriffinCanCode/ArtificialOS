/*!
 * IO_uring subsystem tests entry point
 */

#[path = "iouring/iouring_syscall_test.rs"]
mod iouring_syscall_test;

#[path = "iouring/iouring_ops_test.rs"]
mod iouring_ops_test;

#[path = "iouring/zerocopy_test.rs"]
mod zerocopy_test;
