/*!
 * Standard Filesystem Paths
 * Centralized path constants for consistent filesystem structure
 */

use std::path::{Path, PathBuf};

/// Standard mount points
pub mod mounts {
    pub const STORAGE: &str = "/storage";
    pub const TMP: &str = "/tmp";
    pub const CACHE: &str = "/cache";
}

/// Standard directories under /storage
pub mod storage {
    /// Native applications (prebuilt, bundled with OS)
    pub const NATIVE_APPS: &str = "/storage/native-apps";

    /// User-generated/AI-generated applications
    pub const APPS: &str = "/storage/apps";

    /// User files and data
    pub const USER: &str = "/storage/user";

    /// System configuration and data
    pub const SYSTEM: &str = "/storage/system";

    /// Shared libraries and resources
    pub const LIB: &str = "/storage/lib";
}

/// Application-specific paths
pub mod app {
    use std::path::PathBuf;
    use super::storage;

    /// Get app data directory
    pub fn data_dir(app_id: &str) -> PathBuf {
        PathBuf::from(storage::APPS).join(app_id).join("data")
    }

    /// Get app config directory
    pub fn config_dir(app_id: &str) -> PathBuf {
        PathBuf::from(storage::APPS).join(app_id).join("config")
    }

    /// Get app cache directory
    pub fn cache_dir(app_id: &str) -> PathBuf {
        PathBuf::from("/cache").join(app_id)
    }

    /// Get app temp directory
    pub fn temp_dir(app_id: &str) -> PathBuf {
        PathBuf::from("/tmp").join(app_id)
    }
}

/// User-specific paths
pub mod user {
    pub const DOCUMENTS: &str = "/storage/user/documents";
    pub const DOWNLOADS: &str = "/storage/user/downloads";
    pub const PROJECTS: &str = "/storage/user/projects";
}

/// All standard directories that should be created at init
pub fn standard_directories() -> Vec<&'static str> {
    vec![
        // Storage directories
        storage::NATIVE_APPS,
        storage::APPS,
        storage::USER,
        storage::SYSTEM,
        storage::LIB,

        // User directories
        user::DOCUMENTS,
        user::DOWNLOADS,
        user::PROJECTS,
    ]
}

/// Check if path is within allowed userspace
pub fn is_userspace_path(path: &Path) -> bool {
    path.starts_with(storage::APPS)
        || path.starts_with(storage::USER)
        || path.starts_with("/tmp")
        || path.starts_with("/cache")
}

/// Check if path is a native app
pub fn is_native_app_path(path: &Path) -> bool {
    path.starts_with(storage::NATIVE_APPS)
}

/// Check if path is system-protected
pub fn is_system_path(path: &Path) -> bool {
    path.starts_with(storage::SYSTEM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_paths() {
        let app_id = "my-app";
        assert_eq!(app::data_dir(app_id), PathBuf::from("/storage/apps/my-app/data"));
        assert_eq!(app::config_dir(app_id), PathBuf::from("/storage/apps/my-app/config"));
        assert_eq!(app::cache_dir(app_id), PathBuf::from("/cache/my-app"));
        assert_eq!(app::temp_dir(app_id), PathBuf::from("/tmp/my-app"));
    }

    #[test]
    fn test_userspace_detection() {
        assert!(is_userspace_path(Path::new("/storage/user/documents")));
        assert!(is_userspace_path(Path::new("/storage/apps/myapp")));
        assert!(is_userspace_path(Path::new("/tmp/test")));
        assert!(!is_userspace_path(Path::new("/storage/system/config")));
    }

    #[test]
    fn test_native_app_detection() {
        assert!(is_native_app_path(Path::new("/storage/native-apps/browser")));
        assert!(!is_native_app_path(Path::new("/storage/apps/myapp")));
    }
}

