/*!
 * Granular Capability Checking
 */

use crate::security::types::Capability;
use std::collections::HashSet;
use std::path::Path;

/// Check if a set of capabilities grants a required capability
pub fn has_capability(capabilities: &HashSet<Capability>, required: &Capability) -> bool {
    capabilities.iter().any(|cap| cap.grants(required))
}

/// Check if a file operation is allowed on a specific path
pub fn can_access_file(
    capabilities: &HashSet<Capability>,
    operation: FileOperation,
    path: &Path,
) -> bool {
    let required = match operation {
        FileOperation::Read => Capability::ReadFile(Some(path.to_path_buf())),
        FileOperation::Write => Capability::WriteFile(Some(path.to_path_buf())),
        FileOperation::Create => Capability::CreateFile(Some(path.to_path_buf())),
        FileOperation::Delete => Capability::DeleteFile(Some(path.to_path_buf())),
        FileOperation::List => Capability::ListDirectory(Some(path.to_path_buf())),
    };

    has_capability(capabilities, &required)
}

#[derive(Debug, Clone, Copy)]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_granular_read_permission() {
        let mut caps = HashSet::new();
        caps.insert(Capability::ReadFile(Some(PathBuf::from("/tmp"))));

        assert!(can_access_file(
            &caps,
            FileOperation::Read,
            Path::new("/tmp/test.txt")
        ));
        assert!(!can_access_file(
            &caps,
            FileOperation::Read,
            Path::new("/etc/passwd")
        ));
    }

    #[test]
    fn test_wildcard_permission() {
        let mut caps = HashSet::new();
        caps.insert(Capability::ReadFile(None));

        assert!(can_access_file(
            &caps,
            FileOperation::Read,
            Path::new("/tmp/test.txt")
        ));
        assert!(can_access_file(
            &caps,
            FileOperation::Read,
            Path::new("/etc/passwd")
        ));
    }
}
