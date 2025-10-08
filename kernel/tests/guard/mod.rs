/*!
 * Guard Integration Tests
 *
 * Comprehensive tests for RAII resource guards
 */

mod composite_guard_tests;
mod integration_tests;
mod ipc_guard_tests;
mod lock_guard_tests;
mod memory_guard_tests;
mod observable_guard_tests;
mod transaction_guard_tests;
mod typed_guard_tests;

use ai_os_kernel::core::guard::*;

/// Test that guards are Send + Sync
#[test]
fn test_guards_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    // All guards must be Send + Sync
    assert_send_sync::<GuardMetadata>();
}
