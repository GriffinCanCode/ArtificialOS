/*!
 * JIT Compilation for Hot Syscall Paths
 *
 * eBPF-inspired JIT compilation that optimizes frequently called syscalls
 * by generating specialized fast paths at runtime.
 */

mod compiler;
mod hotpath;
mod optimizer;
mod types;

pub use compiler::JitCompiler;
pub use hotpath::HotpathDetector;
pub use optimizer::SyscallOptimizer;
pub use types::*;

use crate::core::types::Pid;
use crate::syscalls::types::{Syscall, SyscallResult};
use ahash::RandomState;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info};

/// JIT Manager - coordinates hot path detection and compilation
///
/// # Performance
/// - Cache-line aligned for optimal access in hot syscall paths
#[repr(C, align(64))]
#[derive(Clone)]
pub struct JitManager {
    /// Detector for identifying hot syscall paths
    detector: Arc<HotpathDetector>,
    /// Compiler for generating optimized code
    compiler: Arc<JitCompiler>,
    /// Optimizer for applying optimizations
    optimizer: Arc<SyscallOptimizer>,
    /// Cache of compiled syscall handlers
    compiled_cache: Arc<DashMap<SyscallPattern, CompiledHandler, RandomState>>,
    /// Statistics
    stats: Arc<RwLock<JitStats>>,
}

impl JitManager {
    /// Create a new JIT manager with a reference to the syscall executor
    pub fn new(executor: Arc<crate::syscalls::executor::SyscallExecutorWithIpc>) -> Self {
        info!("Initializing JIT manager for syscall optimization");
        Self {
            detector: Arc::new(HotpathDetector::new()),
            compiler: Arc::new(JitCompiler::new(executor)),
            optimizer: Arc::new(SyscallOptimizer::new()),
            compiled_cache: Arc::new(DashMap::with_hasher(RandomState::new())),
            stats: Arc::new(RwLock::new(JitStats::default())),
        }
    }

    /// Record a syscall execution for hot path detection
    pub fn record_syscall(&self, pid: Pid, syscall: &Syscall) {
        self.detector.record(pid, syscall);
    }

    /// Check if a syscall should use JIT-compiled path
    pub fn should_use_jit(&self, pid: Pid, syscall: &Syscall) -> bool {
        self.detector.is_hot(pid, syscall)
    }

    /// Try to execute syscall using JIT-compiled handler
    pub fn try_execute_jit(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        let pattern = SyscallPattern::from_syscall(syscall);

        // Check cache for compiled handler
        if let Some(handler) = self.compiled_cache.get(&pattern) {
            debug!(pid = pid, pattern = ?pattern, "Using JIT-compiled handler");
            let mut stats = self.stats.write();
            stats.jit_hits += 1;
            return Some(handler.execute(pid, syscall));
        }

        // Not yet compiled
        let mut stats = self.stats.write();
        stats.jit_misses += 1;
        None
    }

    /// Compile a hot syscall path
    pub fn compile_hotpath(&self, pattern: SyscallPattern) -> Result<(), JitError> {
        info!(pattern = ?pattern, "Compiling hot syscall path");

        // Analyze the pattern for optimization opportunities
        let optimizations = self.optimizer.analyze(&pattern)?;

        // Compile the optimized handler
        let handler = self.compiler.compile(&pattern, &optimizations)?;

        // Cache the compiled handler
        self.compiled_cache.insert(pattern.clone(), handler);

        let mut stats = self.stats.write();
        stats.compiled_paths += 1;

        info!(
            pattern = ?pattern,
            total_compiled = stats.compiled_paths,
            "Successfully compiled hot syscall path"
        );

        Ok(())
    }

    /// Get JIT statistics
    pub fn stats(&self) -> JitStats {
        self.stats.read().clone()
    }

    /// Get list of hot paths that should be compiled
    pub fn get_compilation_candidates(&self) -> Vec<SyscallPattern> {
        self.detector.get_hot_patterns()
    }

    /// Background task to continuously compile hot paths
    pub async fn compilation_loop(&self) {
        info!("Starting JIT compilation background loop");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let candidates = self.get_compilation_candidates();

            for pattern in candidates {
                // Skip if already compiled
                if self.compiled_cache.contains_key(&pattern) {
                    continue;
                }

                // Compile the hot path
                if let Err(e) = self.compile_hotpath(pattern.clone()) {
                    tracing::warn!(
                        pattern = ?pattern,
                        error = %e,
                        "Failed to compile hot path"
                    );
                }
            }
        }
    }
}

// Note: JitManager no longer implements Default since it requires a SyscallExecutor

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SandboxManager;
    use crate::syscalls::executor::SyscallExecutor;

    #[test]
    fn test_jit_manager_creation() {
        let executor = Arc::new(SyscallExecutor::new(SandboxManager::new()));
        let jit = JitManager::new(executor);
        let stats = jit.stats();
        assert_eq!(stats.compiled_paths, 0);
        assert_eq!(stats.jit_hits, 0);
    }

    #[tokio::test]
    async fn test_hotpath_detection() {
        let executor = Arc::new(SyscallExecutor::new(SandboxManager::new()));
        let jit = JitManager::new(executor);
        let syscall = Syscall::GetProcessList;

        // Record the same syscall many times
        for _ in 0..1000 {
            jit.record_syscall(1, &syscall);
        }

        // Should be detected as hot
        assert!(jit.should_use_jit(1, &syscall));
    }
}
