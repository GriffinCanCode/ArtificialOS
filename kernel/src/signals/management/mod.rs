/*!
 * Signal Management - Manager and Delivery
 * Central signal manager and scheduler integration
 */

mod delivery;
mod manager;

// Re-export public API
pub use delivery::SignalDeliveryHook;
pub use manager::SignalManagerImpl;

