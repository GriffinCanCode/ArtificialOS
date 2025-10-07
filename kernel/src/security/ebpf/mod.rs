/*!
 * eBPF Module
 * Cross-platform eBPF-based syscall filtering and monitoring
 */

mod integration;
mod linux;
mod macos;
mod manager;
mod simulation;
mod traits;
mod types;

pub use integration::{IntegratedEbpfMonitor, ProcessEbpfStats};
pub use manager::EbpfManagerImpl;
pub use traits::*;
pub use types::*;
