/*!
 * IPC subsystem tests entry point
 */

#[path = "ipc/ipc_test.rs"]
mod ipc_test;

#[path = "ipc/unit_ipc_test.rs"]
mod unit_ipc_test;

#[path = "ipc/pipe_test.rs"]
mod pipe_test;

#[path = "ipc/queue_test.rs"]
mod queue_test;

#[path = "ipc/shm_test.rs"]
mod shm_test;

#[path = "ipc/ipc_id_recycling_test.rs"]
mod ipc_id_recycling_test;
