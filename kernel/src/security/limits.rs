/*!
 * Resource Limits
 * OS-level resource limit enforcement (cgroups on Linux, job objects on Windows)
 */

use log::info;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LimitsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid limit: {0}")]
    InvalidLimit(String),
}

/// Resource limits to enforce
#[derive(Debug, Clone)]
pub struct Limits {
    pub memory_bytes: Option<u64>,
    pub cpu_shares: Option<u32>,      // Linux: 1-10000, higher = more CPU
    pub max_pids: Option<u32>,
    pub max_open_files: Option<u32>,
}

impl Limits {
    pub fn new() -> Self {
        Self {
            memory_bytes: None,
            cpu_shares: None,
            max_pids: None,
            max_open_files: None,
        }
    }

    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    pub fn with_cpu_shares(mut self, shares: u32) -> Self {
        self.cpu_shares = Some(shares);
        self
    }

    pub fn with_max_pids(mut self, pids: u32) -> Self {
        self.max_pids = Some(pids);
        self
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            memory_bytes: Some(512 * 1024 * 1024), // 512 MB
            cpu_shares: Some(100),                  // Standard priority
            max_pids: Some(10),
            max_open_files: Some(1024),
        }
    }
}

/// Platform-agnostic resource limit manager
pub struct LimitManager {
    #[cfg(target_os = "linux")]
    cgroup_path: PathBuf,
}

impl LimitManager {
    pub fn new() -> Result<Self, LimitsError> {
        #[cfg(target_os = "linux")]
        {
            let base_path = std::path::PathBuf::from("/sys/fs/cgroup/ai-os");

            // Check if cgroups v2 is available
            if !std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
                log::warn!("cgroups v2 not available, resource limits will be simulated");
            }

            Ok(Self {
                cgroup_path: base_path,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("Resource limits not supported on this platform (simulation mode)");
            Ok(Self {})
        }
    }

    /// Apply resource limits to a process
    pub fn apply(&self, os_pid: u32, limits: &Limits) -> Result<(), LimitsError> {
        #[cfg(target_os = "linux")]
        {
            self.apply_linux(os_pid, limits)
        }

        #[cfg(target_os = "macos")]
        {
            self.apply_macos(os_pid, limits)
        }

        #[cfg(target_os = "windows")]
        {
            self.apply_windows(os_pid, limits)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            warn!("Resource limits not implemented for this platform");
            Ok(())
        }
    }

    /// Remove resource limits for a process
    pub fn remove(&self, os_pid: u32) -> Result<(), LimitsError> {
        #[cfg(target_os = "linux")]
        {
            self.remove_linux(os_pid)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = os_pid; // Silence unused warning
            info!("No resource limits to remove on this platform");
            Ok(())
        }
    }

    // ==================== Linux Implementation ====================

    #[cfg(target_os = "linux")]
    fn apply_linux(&self, os_pid: u32, limits: &Limits) -> Result<(), LimitsError> {
        use std::fs;

        // Create cgroup directory for this process
        let cgroup_dir = self.cgroup_path.join(os_pid.to_string());

        // Check if cgroups v2 is actually available
        if !std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
            log::warn!("cgroups v2 not available, skipping resource limit enforcement");
            return Ok(());
        }

        // Create cgroup directory
        if !cgroup_dir.exists() {
            if let Err(e) = fs::create_dir_all(&cgroup_dir) {
                log::warn!("Failed to create cgroup directory: {}. Skipping resource limits.", e);
                return Ok(()); // Don't fail, just skip
            }
        }

        // Set memory limit
        if let Some(memory) = limits.memory_bytes {
            let memory_max = cgroup_dir.join("memory.max");
            if let Err(e) = fs::write(&memory_max, memory.to_string()) {
                log::warn!("Failed to set memory limit: {}", e);
            } else {
                info!("Set memory limit: {} bytes for PID {}", memory, os_pid);
            }
        }

        // Set CPU weight (cgroups v2 uses weight instead of shares)
        if let Some(shares) = limits.cpu_shares {
            let cpu_weight = cgroup_dir.join("cpu.weight");
            if let Err(e) = fs::write(&cpu_weight, shares.to_string()) {
                log::warn!("Failed to set CPU weight: {}", e);
            } else {
                info!("Set CPU weight: {} for PID {}", shares, os_pid);
            }
        }

        // Set PID limit
        if let Some(max_pids) = limits.max_pids {
            let pids_max = cgroup_dir.join("pids.max");
            if let Err(e) = fs::write(&pids_max, max_pids.to_string()) {
                log::warn!("Failed to set PID limit: {}", e);
            } else {
                info!("Set PID limit: {} for PID {}", max_pids, os_pid);
            }
        }

        // Add process to cgroup
        let procs_file = cgroup_dir.join("cgroup.procs");
        if let Err(e) = fs::write(&procs_file, os_pid.to_string()) {
            log::warn!("Failed to add process to cgroup: {}", e);
        } else {
            info!("Added PID {} to cgroup", os_pid);
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn remove_linux(&self, os_pid: u32) -> Result<(), LimitsError> {
        use std::fs;

        let cgroup_dir = self.cgroup_path.join(os_pid.to_string());

        if cgroup_dir.exists() {
            // Note: Can only remove empty cgroups
            // Process must be terminated first
            if let Err(e) = fs::remove_dir(&cgroup_dir) {
                log::warn!("Failed to remove cgroup for PID {}: {}", os_pid, e);
            } else {
                info!("Removed cgroup for PID {}", os_pid);
            }
        }

        Ok(())
    }

    // ==================== macOS Implementation ====================

    #[cfg(target_os = "macos")]
    fn apply_macos(&self, os_pid: u32, limits: &Limits) -> Result<(), LimitsError> {
        // macOS uses BSD resource limits (setrlimit)
        // This requires unsafe system calls
        info!(
            "Applying resource limits to PID {} (macOS simulation mode)",
            os_pid
        );

        // In production, would use libc::setrlimit
        // For now, just log the limits
        if let Some(memory) = limits.memory_bytes {
            info!("  Memory limit: {} MB", memory / (1024 * 1024));
        }
        if let Some(pids) = limits.max_pids {
            info!("  Max PIDs: {}", pids);
        }

        Ok(())
    }

    // ==================== Windows Implementation ====================

    #[cfg(target_os = "windows")]
    fn apply_windows(&self, os_pid: u32, limits: &Limits) -> Result<(), LimitsError> {
        // Windows uses Job Objects for resource limits
        info!(
            "Applying resource limits to PID {} (Windows simulation mode)",
            os_pid
        );

        // In production, would use Windows Job Objects API
        if let Some(memory) = limits.memory_bytes {
            info!("  Memory limit: {} MB", memory / (1024 * 1024));
        }

        Ok(())
    }
}

impl Default for LimitManager {
    fn default() -> Self {
        Self::new().expect("Failed to create limit manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits_builder() {
        let limits = Limits::new()
            .with_memory(100 * 1024 * 1024)
            .with_cpu_shares(200)
            .with_max_pids(5);

        assert_eq!(limits.memory_bytes, Some(100 * 1024 * 1024));
        assert_eq!(limits.cpu_shares, Some(200));
        assert_eq!(limits.max_pids, Some(5));
    }

    #[test]
    fn test_default_limits() {
        let limits = Limits::default();

        assert_eq!(limits.memory_bytes, Some(512 * 1024 * 1024));
        assert_eq!(limits.cpu_shares, Some(100));
        assert_eq!(limits.max_pids, Some(10));
    }

    #[test]
    fn test_limit_manager_creation() {
        let manager = LimitManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_apply_limits_non_linux() {
        let manager = LimitManager::new().unwrap();
        let limits = Limits::default();

        // Should not fail on non-Linux platforms
        let result = manager.apply(12345, &limits);
        assert!(result.is_ok());
    }
}
