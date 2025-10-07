/*!
 * Security subsystem tests entry point
 */

#[path = "security/sandbox_test.rs"]
mod sandbox_test;

#[path = "security/permissions_test.rs"]
mod permissions_test;

#[path = "security/namespace_test.rs"]
mod namespace_test;

#[path = "security/limits_test.rs"]
mod limits_test;

#[path = "security/ebpf_test.rs"]
mod ebpf_test;

#[path = "security/ebpf_events_test.rs"]
mod ebpf_events_test;

#[path = "security/ebpf_filters_test.rs"]
mod ebpf_filters_test;

#[path = "security/ebpf_loader_test.rs"]
mod ebpf_loader_test;

#[path = "security/ebpf_providers_test.rs"]
mod ebpf_providers_test;
