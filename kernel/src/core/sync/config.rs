/*!
 * Synchronization Configuration
 *
 * Runtime configuration for sync strategy selection
 */

use std::time::Duration;

/// Strategy type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    /// Futex-based (Linux only, fastest)
    Futex,
    /// Condvar-based (cross-platform, reliable)
    Condvar,
    /// Adaptive spinwait (low-latency, high-CPU for short waits)
    SpinWait,
    /// Auto-select based on platform and use case
    Auto,
}

/// Synchronization configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Preferred strategy
    pub strategy: StrategyType,
    /// Spin duration before parking (for SpinWait)
    pub spin_duration: Duration,
    /// Maximum spin iterations before giving up
    pub max_spins: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            strategy: StrategyType::Auto,
            spin_duration: Duration::from_micros(10),
            max_spins: 100,
        }
    }
}

impl SyncConfig {
    /// Configuration optimized for low-latency (< 1ms wait expected)
    pub const fn low_latency() -> Self {
        Self {
            strategy: StrategyType::SpinWait,
            spin_duration: Duration::from_micros(50),
            max_spins: 500,
        }
    }

    /// Configuration optimized for long waits (> 1ms expected)
    pub const fn long_wait() -> Self {
        Self {
            strategy: StrategyType::Auto,
            spin_duration: Duration::from_micros(1),
            max_spins: 10,
        }
    }

    /// Select best strategy for current platform
    pub fn select_strategy(&self) -> StrategyType {
        match self.strategy {
            StrategyType::Auto => {
                // Prefer futex on Linux, condvar elsewhere
                #[cfg(target_os = "linux")]
                {
                    StrategyType::Futex
                }
                #[cfg(not(target_os = "linux"))]
                {
                    StrategyType::Condvar
                }
            }
            other => other,
        }
    }
}
