/*!
 * Filesystem Node Types
 * Internal representation of files and directories
 */

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use super::super::types::{FileType, Permissions};

/// In-memory filesystem node
#[derive(Debug, Clone)]
pub(in crate::vfs) enum Node {
    File {
        data: Vec<u8>,
        permissions: Permissions,
        modified: SystemTime,
        created: SystemTime,
    },
    Directory {
        children: HashMap<String, PathBuf>,
        permissions: Permissions,
        created: SystemTime,
    },
}

impl Node {
    #[allow(dead_code)]
    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    pub fn file_type(&self) -> FileType {
        match self {
            Node::File { .. } => FileType::File,
            Node::Directory { .. } => FileType::Directory,
        }
    }

    pub fn permissions(&self) -> Permissions {
        match self {
            Node::File { permissions, .. } => *permissions,
            Node::Directory { permissions, .. } => *permissions,
        }
    }

    pub fn created(&self) -> SystemTime {
        match self {
            Node::File { created, .. } => *created,
            Node::Directory { created, .. } => *created,
        }
    }
}
