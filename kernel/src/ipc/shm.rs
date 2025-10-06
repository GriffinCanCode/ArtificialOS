/*!
 * Shared Memory Module
 * Zero-copy data sharing between processes
 */

use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;

// Shared memory limits
const MAX_SEGMENT_SIZE: usize = 100 * 1024 * 1024; // 100MB per segment
const MAX_SEGMENTS_PER_PROCESS: usize = 10;
const GLOBAL_SHM_MEMORY_LIMIT: usize = 500 * 1024 * 1024; // 500MB total

// Global shared memory tracking
static GLOBAL_SHM_MEMORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Error)]
pub enum ShmError {
    #[error("Segment not found: {0}")]
    NotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid size: {0}")]
    InvalidSize(String),

    #[error("Invalid offset or size: offset {offset}, size {size}, segment size {segment_size}")]
    InvalidRange {
        offset: usize,
        size: usize,
        segment_size: usize,
    },

    #[error("Segment size exceeds limit: requested {requested}, max {max}")]
    SizeExceeded { requested: usize, max: usize },

    #[error("Process segment limit exceeded: {0}/{1}")]
    ProcessLimitExceeded(usize, usize),

    #[error("Global shared memory limit exceeded: {0}/{1} bytes")]
    GlobalMemoryExceeded(usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShmStats {
    pub id: u32,
    pub size: usize,
    pub owner_pid: u32,
    pub attached_pids: Vec<u32>,
    pub read_only_pids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmPermission {
    ReadWrite,
    ReadOnly,
}

struct SharedSegment {
    id: u32,
    size: usize,
    data: Arc<RwLock<Vec<u8>>>,
    owner_pid: u32,
    attached_pids: HashSet<u32>,
    permissions: HashMap<u32, ShmPermission>,
}

impl SharedSegment {
    fn new(id: u32, size: usize, owner_pid: u32) -> Self {
        let mut attached_pids = HashSet::new();
        attached_pids.insert(owner_pid);

        let mut permissions = HashMap::new();
        permissions.insert(owner_pid, ShmPermission::ReadWrite);

        Self {
            id,
            size,
            data: Arc::new(RwLock::new(vec![0u8; size])),
            owner_pid,
            attached_pids,
            permissions,
        }
    }

    fn has_permission(&self, pid: u32, perm: ShmPermission) -> bool {
        match self.permissions.get(&pid) {
            Some(ShmPermission::ReadWrite) => true,
            Some(ShmPermission::ReadOnly) => perm == ShmPermission::ReadOnly,
            None => false,
        }
    }

    fn attach(&mut self, pid: u32, perm: ShmPermission) {
        self.attached_pids.insert(pid);
        self.permissions.insert(pid, perm);
    }

    fn detach(&mut self, pid: u32) {
        self.attached_pids.remove(&pid);
        self.permissions.remove(&pid);
    }

    fn write(&self, offset: usize, data: &[u8]) -> Result<(), ShmError> {
        if offset + data.len() > self.size {
            return Err(ShmError::InvalidRange {
                offset,
                size: data.len(),
                segment_size: self.size,
            });
        }

        let mut segment_data = self.data.write();
        segment_data[offset..offset + data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read(&self, offset: usize, size: usize) -> Result<Vec<u8>, ShmError> {
        if offset + size > self.size {
            return Err(ShmError::InvalidRange {
                offset,
                size,
                segment_size: self.size,
            });
        }

        let segment_data = self.data.read();
        Ok(segment_data[offset..offset + size].to_vec())
    }
}

pub struct ShmManager {
    segments: Arc<RwLock<HashMap<u32, SharedSegment>>>,
    next_id: Arc<RwLock<u32>>,
    // Track segment count per process
    process_segments: Arc<RwLock<HashMap<u32, usize>>>,
}

impl ShmManager {
    pub fn new() -> Self {
        info!(
            "Shared memory manager initialized (max segment: {} MB, global limit: {} MB)",
            MAX_SEGMENT_SIZE / (1024 * 1024),
            GLOBAL_SHM_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            segments: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            process_segments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create(&self, size: usize, owner_pid: u32) -> Result<u32, ShmError> {
        if size == 0 {
            return Err(ShmError::InvalidSize("Size cannot be zero".to_string()));
        }

        if size > MAX_SEGMENT_SIZE {
            return Err(ShmError::SizeExceeded {
                requested: size,
                max: MAX_SEGMENT_SIZE,
            });
        }

        // Check per-process limit
        let process_segments = self.process_segments.read();
        let count = process_segments.get(&owner_pid).unwrap_or(&0);
        if *count >= MAX_SEGMENTS_PER_PROCESS {
            return Err(ShmError::ProcessLimitExceeded(
                *count,
                MAX_SEGMENTS_PER_PROCESS,
            ));
        }
        drop(process_segments);

        // Check global memory limit
        let current_global = GLOBAL_SHM_MEMORY.load(Ordering::Acquire);
        if current_global + size > GLOBAL_SHM_MEMORY_LIMIT {
            return Err(ShmError::GlobalMemoryExceeded(
                current_global,
                GLOBAL_SHM_MEMORY_LIMIT,
            ));
        }

        let mut segments = self.segments.write();
        let mut next_id = self.next_id.write();

        let segment_id = *next_id;
        *next_id += 1;

        let segment = SharedSegment::new(segment_id, size, owner_pid);
        segments.insert(segment_id, segment);
        drop(segments);
        drop(next_id);

        // Update process segment count
        let mut process_segments = self.process_segments.write();
        *process_segments.entry(owner_pid).or_insert(0) += 1;
        drop(process_segments);

        // Update global memory
        GLOBAL_SHM_MEMORY.fetch_add(size, Ordering::Release);

        info!(
            "Created shared memory segment {} ({} bytes) for PID {} ({} bytes global memory)",
            segment_id,
            size,
            owner_pid,
            GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
        );

        Ok(segment_id)
    }

    pub fn attach(
        &self,
        segment_id: u32,
        pid: u32,
        read_only: bool,
    ) -> Result<(), ShmError> {
        let mut segments = self.segments.write();
        let segment = segments
            .get_mut(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        let perm = if read_only {
            ShmPermission::ReadOnly
        } else {
            ShmPermission::ReadWrite
        };

        segment.attach(pid, perm);

        info!(
            "PID {} attached to segment {} ({:?})",
            pid, segment_id, perm
        );

        Ok(())
    }

    pub fn detach(&self, segment_id: u32, pid: u32) -> Result<(), ShmError> {
        let mut segments = self.segments.write();
        let segment = segments
            .get_mut(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        segment.detach(pid);

        info!("PID {} detached from segment {}", pid, segment_id);

        Ok(())
    }

    pub fn write(
        &self,
        segment_id: u32,
        pid: u32,
        offset: usize,
        data: &[u8],
    ) -> Result<(), ShmError> {
        let segments = self.segments.read();
        let segment = segments
            .get(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        if !segment.has_permission(pid, ShmPermission::ReadWrite) {
            return Err(ShmError::PermissionDenied(
                "Write permission required".to_string(),
            ));
        }

        segment.write(offset, data)?;

        info!(
            "PID {} wrote {} bytes to segment {} at offset {}",
            pid,
            data.len(),
            segment_id,
            offset
        );

        Ok(())
    }

    pub fn read(
        &self,
        segment_id: u32,
        pid: u32,
        offset: usize,
        size: usize,
    ) -> Result<Vec<u8>, ShmError> {
        let segments = self.segments.read();
        let segment = segments
            .get(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        if !segment.has_permission(pid, ShmPermission::ReadOnly) {
            return Err(ShmError::PermissionDenied(
                "Read permission required".to_string(),
            ));
        }

        let data = segment.read(offset, size)?;

        info!(
            "PID {} read {} bytes from segment {} at offset {}",
            pid,
            data.len(),
            segment_id,
            offset
        );

        Ok(data)
    }

    pub fn destroy(&self, segment_id: u32, pid: u32) -> Result<(), ShmError> {
        let mut segments = self.segments.write();
        let segment = segments
            .get(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        // Only owner can destroy
        if segment.owner_pid != pid {
            return Err(ShmError::PermissionDenied(
                "Only owner can destroy segment".to_string(),
            ));
        }

        let owner_pid = segment.owner_pid;
        let size = segment.size;

        segments.remove(&segment_id);
        drop(segments);

        // Update process segment count
        let mut process_segments = self.process_segments.write();
        if let Some(count) = process_segments.get_mut(&owner_pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                process_segments.remove(&owner_pid);
            }
        }
        drop(process_segments);

        // Reclaim global memory
        GLOBAL_SHM_MEMORY.fetch_sub(size, Ordering::Release);

        info!(
            "Destroyed segment {} (reclaimed {} bytes, {} bytes global memory)",
            segment_id,
            size,
            GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
        );

        Ok(())
    }

    pub fn stats(&self, segment_id: u32) -> Result<ShmStats, ShmError> {
        let segments = self.segments.read();
        let segment = segments
            .get(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        let attached_pids: Vec<u32> = segment.attached_pids.iter().copied().collect();
        let read_only_pids: Vec<u32> = segment
            .permissions
            .iter()
            .filter(|(_, perm)| **perm == ShmPermission::ReadOnly)
            .map(|(pid, _)| *pid)
            .collect();

        Ok(ShmStats {
            id: segment.id,
            size: segment.size,
            owner_pid: segment.owner_pid,
            attached_pids,
            read_only_pids,
        })
    }

    pub fn cleanup_process(&self, pid: u32) -> usize {
        let segments = self.segments.read();
        let segment_ids: Vec<u32> = segments
            .values()
            .filter(|s| s.owner_pid == pid || s.attached_pids.contains(&pid))
            .map(|s| s.id)
            .collect();
        drop(segments);

        let mut count = 0;

        for segment_id in segment_ids {
            let segments = self.segments.read();
            if let Some(segment) = segments.get(&segment_id) {
                if segment.owner_pid == pid {
                    drop(segments);
                    if let Err(e) = self.destroy(segment_id, pid) {
                        warn!("Failed to destroy segment {} during cleanup: {}", segment_id, e);
                    } else {
                        count += 1;
                    }
                } else {
                    drop(segments);
                    if let Err(e) = self.detach(segment_id, pid) {
                        warn!("Failed to detach from segment {} during cleanup: {}", segment_id, e);
                    }
                }
            }
        }

        if count > 0 {
            info!("Cleaned up {} shared memory segments for PID {}", count, pid);
        }

        count
    }

    pub fn get_global_memory_usage(&self) -> usize {
        GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
    }
}

impl Clone for ShmManager {
    fn clone(&self) -> Self {
        Self {
            segments: Arc::clone(&self.segments),
            next_id: Arc::clone(&self.next_id),
            process_segments: Arc::clone(&self.process_segments),
        }
    }
}

impl Default for ShmManager {
    fn default() -> Self {
        Self::new()
    }
}
