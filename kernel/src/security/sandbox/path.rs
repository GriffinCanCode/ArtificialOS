/*!
 * TOCTOU-Safe Path Handling
 * Canonicalizes paths once to prevent time-of-check-to-time-of-use issues
 */

use std::path::{Path, PathBuf};

/// Canonicalized path handle that prevents TOCTOU issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathHandle {
    canonical: PathBuf,
}

impl PathHandle {
    /// Create a new path handle, canonicalizing the path
    pub fn new(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            canonical: path.canonicalize()?,
        })
    }

    /// Create from an existing path without canonicalization
    pub fn from_canonical(path: PathBuf) -> Self {
        Self { canonical: path }
    }

    /// Try to create a path handle, falling back to parent canonicalization for non-existent paths
    pub fn try_new(path: &Path) -> std::io::Result<Self> {
        if path.exists() {
            Self::new(path)
        } else if let Some(parent) = path.parent() {
            let file_name = path.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("path has no file name component: {}", path.display()),
                )
            })?;
            let canonical = parent.canonicalize()?.join(file_name);
            Ok(Self { canonical })
        } else {
            Ok(Self {
                canonical: path.to_path_buf(),
            })
        }
    }

    /// Get the canonical path
    pub fn as_path(&self) -> &Path {
        &self.canonical
    }

    /// Check if this path is within another path
    pub fn is_within(&self, base: &PathHandle) -> bool {
        self.canonical.starts_with(&base.canonical)
    }

    /// Check if this path starts with any of the given paths
    pub fn starts_with_any(&self, paths: &[PathBuf]) -> bool {
        paths.iter().any(|p| self.canonical.starts_with(p))
    }
}

/// Check if a path is accessible given allowed and blocked lists
pub fn check_path_access(
    path: &PathHandle,
    allowed_paths: &[PathBuf],
    blocked_paths: &[PathBuf],
) -> bool {
    // First check if explicitly blocked
    if path.starts_with_any(blocked_paths) {
        return false;
    }

    // If no allowed paths specified, deny all
    if allowed_paths.is_empty() {
        return false;
    }

    // Check if path is within allowed paths
    path.starts_with_any(allowed_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_handle_canonicalization() {
        let temp = std::env::temp_dir();
        let handle = PathHandle::try_new(&temp).unwrap();
        assert!(handle.as_path().is_absolute());
    }

    #[test]
    fn test_path_within() {
        let temp = std::env::temp_dir();
        let parent = PathHandle::try_new(&temp).unwrap();
        let child = PathHandle::try_new(&temp.join("test")).unwrap();
        assert!(child.is_within(&parent));
    }
}
