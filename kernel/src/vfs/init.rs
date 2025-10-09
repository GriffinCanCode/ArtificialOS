/*!
 * VFS Initialization
 * Creates standard directory structure and mounts
 */

use std::sync::Arc;
use tracing::{info, warn};

use super::{
    mount::MountManager,
    paths::{self, standard_directories},
    traits::FileSystem,
    LocalFS, MemFS, ObservableFS,
};
use crate::core::limits::{CACHE_FILESYSTEM_CAPACITY, TMP_FILESYSTEM_CAPACITY};

/// Initialize VFS with standard mounts and directory structure
pub fn init_vfs(storage_path: &str) -> Result<MountManager, Box<dyn std::error::Error>> {
    info!("Initializing VFS with standard directory structure");

    let vfs = MountManager::new();

    // Mount local filesystem at /storage for persistent data
    info!(storage_path = %storage_path, "Mounting observable local filesystem at /storage");
    if let Err(e) = std::fs::create_dir_all(storage_path) {
        warn!(error = %e, "Could not create storage directory");
    }

    let storage_fs = ObservableFS::new(LocalFS::new(storage_path));
    vfs.mount("/storage", Arc::new(storage_fs))?;

    // Mount in-memory filesystem at /tmp (100MB limit)
    info!("Mounting observable in-memory filesystem at /tmp (100MB limit)");
    let tmp_fs = ObservableFS::new(MemFS::with_capacity(TMP_FILESYSTEM_CAPACITY));
    vfs.mount("/tmp", Arc::new(tmp_fs))?;

    // Mount in-memory filesystem at /cache (50MB limit)
    info!("Mounting observable in-memory filesystem at /cache (50MB limit)");
    let cache_fs = ObservableFS::new(MemFS::with_capacity(CACHE_FILESYSTEM_CAPACITY));
    vfs.mount("/cache", Arc::new(cache_fs))?;

    info!("File watching enabled for all mount points");

    // Create standard directory structure
    create_standard_directories(&vfs)?;

    info!("VFS initialization complete");
    Ok(vfs)
}

/// Create all standard directories in the filesystem
fn create_standard_directories(vfs: &MountManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating standard directory structure");

    let dirs = standard_directories();
    let total = dirs.len();
    let mut created = 0;
    let mut existed = 0;

    for dir in dirs {
        let path = std::path::Path::new(dir);
        if vfs.exists(path) {
            existed += 1;
            continue;
        }

        match vfs.create_dir(path) {
            Ok(()) => {
                created += 1;
            }
            Err(e) => {
                warn!(path = %dir, error = %e, "Failed to create directory");
            }
        }
    }

    info!(
        created = created,
        existed = existed,
        total = total,
        "Standard directory structure ready"
    );

    // Log the structure for visibility
    info!("Filesystem structure:");
    info!("  /storage/");
    info!("    ├── native-apps/   (prebuilt OS applications)");
    info!("    ├── apps/          (user/AI-generated apps)");
    info!("    ├── user/          (user files)");
    info!("    │   ├── documents/");
    info!("    │   ├── downloads/");
    info!("    │   └── projects/");
    info!("    ├── system/        (system config)");
    info!("    └── lib/           (shared libraries)");
    info!("  /tmp/                (temporary files, 100MB)");
    info!("  /cache/              (cache files, 50MB)");

    Ok(())
}

/// Sync native apps from apps/dist to /storage/native-apps
pub fn sync_native_apps(
    vfs: &MountManager,
    source_dist: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let dist_path = Path::new(source_dist);
    if !dist_path.exists() {
        info!("No native apps dist directory found at {}", source_dist);
        return Ok(());
    }

    info!(source = %source_dist, "Syncing native apps to /storage/native-apps");

    let target_base = Path::new(paths::storage::NATIVE_APPS);

    // Read all apps from dist
    for entry in std::fs::read_dir(dist_path)? {
        let entry = entry?;
        let app_name = entry.file_name();
        let app_name_str = app_name.to_string_lossy();

        if !entry.path().is_dir() {
            continue;
        }

        info!(app = %app_name_str, "Syncing native app");

        // Copy entire app directory
        let source = entry.path();
        let target = target_base.join(&app_name);

        if let Err(e) = copy_dir_recursive(vfs, &source, &target) {
            warn!(app = %app_name_str, error = %e, "Failed to sync native app");
        }
    }

    info!("Native apps sync complete");
    Ok(())
}

/// Recursively copy directory from host filesystem to VFS
fn copy_dir_recursive(
    vfs: &MountManager,
    source: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create target directory
    if !vfs.exists(target) {
        vfs.create_dir(target)?;
    }

    // Copy all entries
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let source_path = entry.path();
        let target_path = target.join(&file_name);

        if entry.path().is_dir() {
            // Recursively copy subdirectory
            copy_dir_recursive(vfs, &source_path, &target_path)?;
        } else {
            // Copy file
            let data = std::fs::read(&source_path)?;
            vfs.write(&target_path, &data)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_vfs() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("storage");

        let vfs = init_vfs(storage_path.to_str().unwrap()).unwrap();

        // Check standard directories exist
        assert!(vfs.exists(std::path::Path::new("/storage/native-apps")));
        assert!(vfs.exists(std::path::Path::new("/storage/apps")));
        assert!(vfs.exists(std::path::Path::new("/storage/user")));
        assert!(vfs.exists(std::path::Path::new("/storage/system")));
    }

    #[test]
    fn test_create_standard_directories() {
        let vfs = MountManager::new();
        let temp = TempDir::new().unwrap();

        // Mount a temp filesystem
        let fs = ObservableFS::new(LocalFS::new(temp.path()));
        vfs.mount("/storage", Arc::new(fs)).unwrap();

        create_standard_directories(&vfs).unwrap();

        // Verify directories exist
        for dir in standard_directories() {
            assert!(vfs.exists(std::path::Path::new(dir)), "Directory {} should exist", dir);
        }
    }
}

