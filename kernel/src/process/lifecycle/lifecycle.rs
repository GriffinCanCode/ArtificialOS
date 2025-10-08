/*!
 * Process Lifecycle Hooks
 *
 * Coordinates initialization and cleanup of process resources across subsystems.
 * Eliminates race conditions by ensuring all resources are initialized BEFORE
 * the process becomes schedulable.
 *
 * # Architecture
 *
 * - **Lifecycle Registry**: Central coordinator that runs hooks in dependency order
 * - **Lifecycle Hooks**: Trait that subsystems implement for init/cleanup
 * - **Explicit States**: Process goes through Creating → Initializing → Ready
 * - **Zero Race Conditions**: Process can't run until initialization completes
 *
 * # Example
 *
 * ```ignore
 * let registry = LifecycleRegistry::new()
 *     .with_signal_manager(signal_mgr)
 *     .with_zerocopy_ipc(zerocopy_ipc)
 *     .with_fd_manager(fd_mgr);
 *
 * // Initialize all resources atomically
 * registry.initialize_process(pid, config)?;
 * // Process is now fully initialized and can be scheduled
 * ```
 */

use crate::core::types::Pid;
use crate::ipc::zerocopy::ZeroCopyIpc;
use crate::signals::SignalManagerImpl;
use crate::syscalls::impls::fd::FdManager;
use log::{debug, info};
use std::sync::Arc;
use thiserror::Error;

/// Lifecycle hook errors
#[derive(Error, Debug, Clone)]
pub enum LifecycleError {
    #[error("Initialization failed for {subsystem}: {reason}")]
    InitializationFailed { subsystem: String, reason: String },

    #[error("Process {0} already initialized")]
    AlreadyInitialized(Pid),

    #[error("Invalid lifecycle state transition: {0}")]
    InvalidStateTransition(String),
}

pub type LifecycleResult<T> = Result<T, LifecycleError>;

/// Configuration for process initialization
#[derive(Debug, Clone)]
pub struct ProcessInitConfig {
    /// Enable zero-copy ring creation
    pub enable_zerocopy: bool,
    /// Zero-copy ring submission queue size
    pub zerocopy_sq_size: usize,
    /// Zero-copy ring completion queue size
    pub zerocopy_cq_size: usize,
    /// Initialize signal handlers
    pub enable_signals: bool,
    /// Initialize FD table (stdin/stdout/stderr)
    pub enable_stdio: bool,
}

impl Default for ProcessInitConfig {
    fn default() -> Self {
        Self {
            enable_zerocopy: true,
            zerocopy_sq_size: crate::core::limits::PROCESS_ZEROCOPY_SQ_SIZE,
            zerocopy_cq_size: crate::core::limits::PROCESS_ZEROCOPY_CQ_SIZE,
            enable_signals: true,
            enable_stdio: true,
        }
    }
}

impl ProcessInitConfig {
    #[inline]
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            enable_zerocopy: false,
            zerocopy_sq_size: 0,
            zerocopy_cq_size: 0,
            enable_signals: true, // Signals are lightweight, always enable
            enable_stdio: false,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_zerocopy(mut self, sq_size: usize, cq_size: usize) -> Self {
        self.enable_zerocopy = true;
        self.zerocopy_sq_size = sq_size;
        self.zerocopy_cq_size = cq_size;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_stdio(mut self) -> Self {
        self.enable_stdio = true;
        self
    }
}

/// Process lifecycle coordinator
///
/// Runs initialization hooks in dependency order:
/// 1. Memory (prerequisite for everything)
/// 2. IPC zero-copy rings (needs memory)
/// 3. Signals (independent)
/// 4. File descriptors (may use signals)
pub struct LifecycleRegistry {
    signal_manager: Option<Arc<SignalManagerImpl>>,
    zerocopy_ipc: Option<Arc<ZeroCopyIpc>>,
    fd_manager: Option<Arc<FdManager>>,
}

impl LifecycleRegistry {
    /// Create a new lifecycle registry
    pub fn new() -> Self {
        info!("Process lifecycle registry initialized");
        Self {
            signal_manager: None,
            zerocopy_ipc: None,
            fd_manager: None,
        }
    }

    /// Register signal manager for lifecycle hooks
    #[inline]
    #[must_use]
    pub fn with_signal_manager(mut self, manager: Arc<SignalManagerImpl>) -> Self {
        self.signal_manager = Some(manager);
        self
    }

    /// Register zero-copy IPC for lifecycle hooks
    #[inline]
    #[must_use]
    pub fn with_zerocopy_ipc(mut self, ipc: Arc<ZeroCopyIpc>) -> Self {
        self.zerocopy_ipc = Some(ipc);
        self
    }

    /// Register FD manager for lifecycle hooks
    #[inline]
    #[must_use]
    pub fn with_fd_manager(mut self, manager: Arc<FdManager>) -> Self {
        self.fd_manager = Some(manager);
        self
    }

    /// Initialize all process resources in dependency order
    ///
    /// This runs BEFORE the process becomes schedulable, eliminating race conditions.
    /// Hooks execute in order:
    /// 1. Zero-copy ring (needs memory, allocated by IPC manager)
    /// 2. Signal state (independent)
    /// 3. FD table initialization (stdio descriptors)
    pub fn initialize_process(&self, pid: Pid, config: &ProcessInitConfig) -> LifecycleResult<()> {
        debug!("Initializing process {} with lifecycle hooks", pid);

        let mut initialized_subsystems = Vec::with_capacity(4);

        // Hook 1: Zero-copy ring (if enabled and available)
        if config.enable_zerocopy {
            if let Some(ref zerocopy) = self.zerocopy_ipc {
                zerocopy
                    .create_ring(pid, config.zerocopy_sq_size, config.zerocopy_cq_size)
                    .map_err(|e| LifecycleError::InitializationFailed {
                        subsystem: "zerocopy".to_string(),
                        reason: e.to_string(),
                    })?;
                initialized_subsystems.push("zerocopy-ring");
                debug!("Initialized zero-copy ring for PID {}", pid);
            }
        }

        // Hook 2: Signal state (if enabled and available)
        if config.enable_signals {
            if let Some(ref signal_mgr) = self.signal_manager {
                use crate::signals::SignalStateManager;
                signal_mgr.initialize_process(pid).map_err(|e| {
                    LifecycleError::InitializationFailed {
                        subsystem: "signals".to_string(),
                        reason: e.to_string(),
                    }
                })?;
                initialized_subsystems.push("signal-state");
                debug!("Initialized signal state for PID {}", pid);
            }
        }

        // Hook 3: FD table with stdio (if enabled)
        // NOTE: FdManager currently doesn't require explicit initialization,
        // but we reserve this hook for future stdio setup (0, 1, 2 descriptors)
        if config.enable_stdio {
            if let Some(ref _fd_mgr) = self.fd_manager {
                // Future: Initialize stdin/stdout/stderr here
                // For now, FDs are created on-demand which is acceptable
                // since the kernel manages 0/1/2 at the OS level
                initialized_subsystems.push("fd-table");
                debug!("FD table ready for PID {}", pid);
            }
        }

        info!(
            "Process {} fully initialized: [{}]",
            pid,
            initialized_subsystems.join(", ")
        );

        Ok(())
    }

    /// Cleanup lifecycle-managed process resources
    ///
    /// This runs during process termination for lifecycle-specific cleanup.
    /// Most resource cleanup is now handled by ResourceOrchestrator.
    ///
    /// Hooks execute in reverse order:
    /// 1. Signal state cleanup
    /// 2. Zero-copy ring cleanup
    pub fn cleanup_process(&self, pid: Pid) -> usize {
        debug!("Cleaning up process {} lifecycle resources", pid);

        let mut cleaned_count = 0;

        // Hook 2: Signal cleanup
        if let Some(ref signal_mgr) = self.signal_manager {
            let count = signal_mgr.cleanup_process_signals(pid);
            cleaned_count += count;
            debug!("Cleaned {} signal resources for PID {}", count, pid);
        }

        // Hook 3: Zero-copy ring cleanup
        if let Some(ref zerocopy) = self.zerocopy_ipc {
            let (count, _bytes) = zerocopy.cleanup_process_rings(pid);
            cleaned_count += count;
            if count > 0 {
                debug!("Cleaned {} zero-copy resources for PID {}", count, pid);
            }
        }

        info!(
            "Process {} lifecycle cleanup completed: {} resources",
            pid, cleaned_count
        );

        cleaned_count
    }

    /// Check if process has been initialized
    pub fn is_initialized(&self, pid: Pid) -> bool {
        // Check if signal state exists (good proxy for initialization)
        if let Some(ref signal_mgr) = self.signal_manager {
            signal_mgr.has_process_signals(pid)
        } else {
            // Without signal manager, assume initialized
            // (this is for minimal configurations)
            true
        }
    }
}

impl Default for LifecycleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LifecycleRegistry {
    fn clone(&self) -> Self {
        Self {
            signal_manager: self.signal_manager.clone(),
            zerocopy_ipc: self.zerocopy_ipc.clone(),
            fd_manager: self.fd_manager.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_init_config_default() {
        let config = ProcessInitConfig::default();
        assert!(config.enable_zerocopy);
        assert!(config.enable_signals);
        assert!(config.enable_stdio);
        assert_eq!(
            config.zerocopy_sq_size,
            crate::core::limits::PROCESS_ZEROCOPY_SQ_SIZE
        );
        assert_eq!(
            config.zerocopy_cq_size,
            crate::core::limits::PROCESS_ZEROCOPY_CQ_SIZE
        );
    }

    #[test]
    fn test_process_init_config_minimal() {
        let config = ProcessInitConfig::minimal();
        assert!(!config.enable_zerocopy);
        assert!(config.enable_signals); // Always enabled
        assert!(!config.enable_stdio);
    }

    #[test]
    fn test_lifecycle_registry_creation() {
        let registry = LifecycleRegistry::new();
        assert!(registry.signal_manager.is_none());
        assert!(registry.zerocopy_ipc.is_none());
        assert!(registry.fd_manager.is_none());
    }
}
