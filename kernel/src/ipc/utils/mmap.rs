/*!
 * Memory-Mapped Files (mmap)
 * File-backed shared memory support
 */

use crate::core::types::Pid;
use crate::core::{ShardManager, WorkloadProfile};
use crate::vfs::{FileSystem, MountManager};
use ahash::RandomState;
use dashmap::DashMap;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Memory mapping identifier
pub type MmapId = u32;

/// Memory mapping protection flags (similar to POSIX mmap)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtFlags {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
}

// Implement BincodeSerializable for efficient internal transfers
impl crate::core::traits::BincodeSerializable for ProtFlags {}

impl ProtFlags {
    pub const PROT_READ: Self = Self {
        read: true,
        write: false,
        exec: false,
    };

    pub const PROT_WRITE: Self = Self {
        read: false,
        write: true,
        exec: false,
    };

    pub const PROT_EXEC: Self = Self {
        read: false,
        write: false,
        exec: true,
    };

    pub const PROT_NONE: Self = Self {
        read: false,
        write: false,
        exec: false,
    };

    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            exec: false,
        }
    }

    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            exec: true,
        }
    }
}

/// Memory mapping flags (similar to POSIX mmap)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapFlags {
    /// Shared mapping - changes visible to other processes
    Shared,
    /// Private mapping - copy-on-write
    Private,
}

// Implement BincodeSerializable for efficient internal transfers
impl crate::core::traits::BincodeSerializable for MapFlags {}

/// Memory-mapped file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmapEntry {
    pub id: MmapId,
    pub path: String,
    pub offset: usize,
    pub length: usize,
    pub prot: ProtFlags,
    pub flags: MapFlags,
    pub owner_pid: Pid,
    pub data: Arc<Vec<u8>>, // In-memory representation
}

/// Memory-mapped file manager
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic ID counter
#[repr(C, align(64))]
pub struct MmapManager {
    mappings: Arc<DashMap<MmapId, MmapEntry, RandomState>>,
    next_id: Arc<AtomicU32>, // Wrapped in Arc to ensure ID uniqueness across clones
    vfs: Option<Arc<MountManager>>,
}

impl MmapManager {
    /// Create a new mmap manager
    pub fn new() -> Self {
        info!("Mmap manager initialized");
        Self {
            // CPU-topology-aware shard counts for optimal concurrent performance
            mappings: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::LowContention), // mmap: infrequent access
            )),
            next_id: Arc::new(AtomicU32::new(1)),
            vfs: None,
        }
    }

    /// Create mmap manager with VFS support
    pub fn with_vfs(vfs: Arc<MountManager>) -> Self {
        info!("Mmap manager initialized with VFS support");
        Self {
            // CPU-topology-aware shard counts for optimal concurrent performance
            mappings: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::LowContention), // mmap: infrequent access
            )),
            next_id: Arc::new(AtomicU32::new(1)),
            vfs: Some(vfs),
        }
    }

    /// Create a memory mapping from a file
    pub fn mmap(
        &self,
        pid: Pid,
        path: String,
        offset: usize,
        length: usize,
        prot: ProtFlags,
        flags: MapFlags,
    ) -> Result<MmapId, String> {
        // Read file data from VFS
        let file_data = if let Some(ref vfs) = self.vfs {
            vfs.read(Path::new(&path))
                .map_err(|e| format!("Failed to read file: {}", e))?
        } else {
            return Err("VFS not available".to_string());
        };

        // Validate offset and length
        if offset >= file_data.len() {
            return Err(format!(
                "Offset {} exceeds file size {}",
                offset,
                file_data.len()
            ));
        }

        let end = offset.saturating_add(length).min(file_data.len());
        let actual_length = end - offset;

        // Extract the mapped region
        let mapped_data = file_data[offset..end].to_vec();

        // Allocate mapping ID
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let entry = MmapEntry {
            id,
            path: path.clone(),
            offset,
            length: actual_length,
            prot,
            flags,
            owner_pid: pid,
            data: Arc::new(mapped_data),
        };

        self.mappings.insert(id, entry);

        info!(
            "PID {} created mmap {} for file '{}' (offset: {}, length: {})",
            pid, id, path, offset, actual_length
        );

        Ok(id)
    }

    /// Read from a memory mapping
    pub fn read(
        &self,
        pid: Pid,
        mmap_id: MmapId,
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>, String> {
        let entry = self
            .mappings
            .get(&mmap_id)
            .ok_or_else(|| format!("Mmap {} not found", mmap_id))?;

        // Check read permission
        if !entry.prot.read {
            return Err("No read permission on this mapping".to_string());
        }

        // Validate access
        if offset >= entry.data.len() {
            return Err(format!(
                "Offset {} exceeds mapping size {}",
                offset,
                entry.data.len()
            ));
        }

        let end = offset.saturating_add(length).min(entry.data.len());
        let data = entry.data[offset..end].to_vec();

        debug!(
            "PID {} read {} bytes from mmap {}",
            pid,
            data.len(),
            mmap_id
        );
        Ok(data)
    }

    /// Write to a memory mapping (for shared mappings)
    pub fn write(
        &self,
        pid: Pid,
        mmap_id: MmapId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), String> {
        let mut entry = self
            .mappings
            .get_mut(&mmap_id)
            .ok_or_else(|| format!("Mmap {} not found", mmap_id))?;

        // Check write permission
        if !entry.prot.write {
            return Err("No write permission on this mapping".to_string());
        }

        // For private mappings, implement copy-on-write
        if entry.flags == MapFlags::Private {
            // Clone the data (copy-on-write)
            let mut new_data = (*entry.data).clone();

            // Validate access
            if offset.saturating_add(data.len()) > new_data.len() {
                return Err("Write exceeds mapping bounds".to_string());
            }

            // Write to the copy
            new_data[offset..offset + data.len()].copy_from_slice(data);
            entry.data = Arc::new(new_data);

            debug!(
                "PID {} wrote {} bytes to private mmap {} (CoW)",
                pid,
                data.len(),
                mmap_id
            );
        } else {
            // Shared mapping - need to make data mutable
            // In a real implementation, this would need Arc::make_mut or similar
            warn!("Shared mmap writes not fully implemented - data is read-only after initial mapping");
            return Err("Shared mmap writes not yet supported".to_string());
        }

        Ok(())
    }

    /// Synchronize a memory mapping back to the file
    pub fn msync(&self, pid: Pid, mmap_id: MmapId) -> Result<(), String> {
        let entry = self
            .mappings
            .get(&mmap_id)
            .ok_or_else(|| format!("Mmap {} not found", mmap_id))?;

        // Only sync shared writable mappings
        if entry.flags != MapFlags::Shared || !entry.prot.write {
            debug!("Mmap {} does not require sync", mmap_id);
            return Ok(());
        }

        // Write back to VFS
        if let Some(ref vfs) = self.vfs {
            // Read current file
            let mut file_data = vfs
                .read(Path::new(&entry.path))
                .map_err(|e| format!("Failed to read file for sync: {}", e))?;

            // Update the mapped region
            let end = entry.offset + entry.length;
            if end <= file_data.len() {
                file_data[entry.offset..end].copy_from_slice(&entry.data);
            } else {
                // Extend file if needed
                file_data.resize(end, 0);
                file_data[entry.offset..end].copy_from_slice(&entry.data);
            }

            // Write back
            vfs.write(Path::new(&entry.path), &file_data)
                .map_err(|e| format!("Failed to write file for sync: {}", e))?;

            info!(
                "PID {} synced mmap {} to file '{}'",
                pid, mmap_id, entry.path
            );
        } else {
            return Err("VFS not available for sync".to_string());
        }

        Ok(())
    }

    /// Unmap a memory mapping
    pub fn munmap(&self, pid: Pid, mmap_id: MmapId) -> Result<(), String> {
        // Automatically sync shared writable mappings before unmapping
        if let Some(entry) = self.mappings.get(&mmap_id) {
            if entry.flags == MapFlags::Shared && entry.prot.write {
                if let Err(e) = self.msync(pid, mmap_id) {
                    warn!("Failed to sync mmap {} before unmapping: {}", mmap_id, e);
                }
            }
        }

        self.mappings
            .remove(&mmap_id)
            .ok_or_else(|| format!("Mmap {} not found", mmap_id))?;

        info!("PID {} unmapped mmap {}", pid, mmap_id);
        Ok(())
    }

    /// Clean up all mappings for a process
    pub fn cleanup_process(&self, pid: Pid) -> usize {
        let to_remove: Vec<MmapId> = self
            .mappings
            .iter()
            .filter(|entry| entry.value().owner_pid == pid)
            .map(|entry| *entry.key())
            .collect();

        let count = to_remove.len();
        for mmap_id in to_remove {
            let _ = self.munmap(pid, mmap_id);
        }

        if count > 0 {
            info!("Cleaned up {} mmaps for PID {}", count, pid);
        }

        count
    }

    /// Cleanup all mappings for a terminated process
    /// Returns (count of mappings cleaned, bytes freed)
    pub fn cleanup_process_mappings(&self, pid: Pid) -> (usize, usize) {
        let to_remove: Vec<(MmapId, usize)> = self
            .mappings
            .iter()
            .filter(|entry| entry.value().owner_pid == pid)
            .map(|entry| (*entry.key(), entry.value().length))
            .collect();

        let count = to_remove.len();
        let bytes: usize = to_remove.iter().map(|(_, size)| size).sum();

        for (mmap_id, _) in to_remove {
            let _ = self.munmap(pid, mmap_id);
        }

        if count > 0 {
            info!(
                "Cleaned {} memory mappings ({} bytes) for terminated PID {}",
                count, bytes, pid
            );
        }

        (count, bytes)
    }

    /// Check if process has any mappings
    pub fn has_process_mappings(&self, pid: Pid) -> bool {
        self.mappings
            .iter()
            .any(|entry| entry.value().owner_pid == pid)
    }

    /// Get mapping information
    pub fn get_info(&self, mmap_id: MmapId) -> Option<MmapEntry> {
        self.mappings.get(&mmap_id).map(|e| e.value().clone())
    }

    /// List all mappings for a process
    pub fn list_mappings(&self, pid: Pid) -> Vec<MmapEntry> {
        self.mappings
            .iter()
            .filter(|entry| entry.value().owner_pid == pid)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Clone for MmapManager {
    fn clone(&self) -> Self {
        Self {
            mappings: Arc::clone(&self.mappings),
            next_id: Arc::clone(&self.next_id), // Share ID counter to prevent collision
            vfs: self.vfs.clone(),
        }
    }
}

impl Default for MmapManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_without_vfs() {
        let manager = MmapManager::new();
        let result = manager.mmap(
            1,
            "/test/file".to_string(),
            0,
            1024,
            ProtFlags::read_write(),
            MapFlags::Shared,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_prot_flags() {
        let rw = ProtFlags::read_write();
        assert!(rw.read);
        assert!(rw.write);
        assert!(!rw.exec);

        let all = ProtFlags::all();
        assert!(all.read && all.write && all.exec);
    }
}
