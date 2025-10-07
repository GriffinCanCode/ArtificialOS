/*!
 * eBPF Module
 * Cross-platform eBPF-based syscall filtering and monitoring
 */

mod events;
mod filters;
mod integration;
mod loader;
mod manager;
mod simulation;
mod traits;
mod types;

// Make platform modules public for testing
pub mod linux;
pub mod macos;

pub use events::{EventCollector, EventType};
pub use filters::FilterManager;
pub use integration::{IntegratedEbpfMonitor, ProcessEbpfStats};
pub use loader::ProgramLoader;
pub use manager::EbpfManagerImpl;
pub use traits::*;
pub use types::*;
