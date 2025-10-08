/*!
 * Process and System Types
 * Defines helper structures for process and system information
 */

use crate::core::serialization::serde::{is_empty_string, skip_serializing_none};
use serde::{Deserialize, Serialize};

/// Process execution output
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProcessOutput {
    /// Standard output content
    #[serde(default, skip_serializing_if = "is_empty_string")]
    pub stdout: String,

    /// Standard error content
    #[serde(default, skip_serializing_if = "is_empty_string")]
    pub stderr: String,

    /// Process exit code
    pub exit_code: i32,
}

impl ProcessOutput {
    /// Create new process output
    #[inline]
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
        }
    }

    /// Check if process was successful (exit code 0)
    #[inline]
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Check if output is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// Operating system name
    pub os: String,

    /// CPU architecture
    pub arch: String,

    /// OS family (unix, windows, etc.)
    pub family: String,
}

impl SystemInfo {
    /// Create new system info
    #[inline]
    pub fn new(os: String, arch: String, family: String) -> Self {
        Self { os, arch, family }
    }

    /// Get current system info
    #[inline]
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            family: std::env::consts::FAMILY.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_output() {
        let output = ProcessOutput::new("hello".into(), String::new(), 0);
        assert!(output.is_success());
        assert!(!output.is_empty());

        let output = ProcessOutput::default();
        assert!(output.is_success());
        assert!(output.is_empty());
    }

    #[test]
    fn test_system_info_serialization() {
        let info = SystemInfo::current();
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SystemInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, deserialized);
    }
}
