/*!
 * Shared Memory Manager
 * Central manager for shared memory segments
 */

use super::super::core::types::ShmId;
use super::segment::SharedSegment;
use super::types::{
    ShmError, ShmPermission, ShmStats, GLOBAL_SHM_MEMORY_LIMIT, MAX_SEGMENTS_PER_PROCESS,
    MAX_SEGMENT_SIZE,
};
use crate::core::types::{Pid, Size};
use crate::memory::MemoryManager;
use dashmap::DashMap;
use ahash::RandomState;
use log::{info, warn};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// Global shared memory tracking
static GLOBAL_SHM_MEMORY: AtomicUsize = AtomicUsize::new(0);

/// Shared memory manager
pub struct ShmManager {
    segments: Arc<DashMap<ShmId, SharedSegment, RandomState>>,
    next_id: Arc<RwLock<ShmId>>,
    // Track segment count per process
    process_segments: Arc<DashMap<Pid, Size, RandomState>>,
    memory_manager: MemoryManager,
    // Free IDs for recycling (prevents ID exhaustion)
    free_ids: Arc<Mutex<Vec<ShmId>>>,
}

impl ShmManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "Shared memory manager initialized with ID recycling (max segment: {} MB, global limit: {} MB)",
            MAX_SEGMENT_SIZE / (1024 * 1024),
            GLOBAL_SHM_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            segments: Arc::new(DashMap::with_hasher(RandomState::new())),
            next_id: Arc::new(RwLock::new(1)),
            process_segments: Arc::new(DashMap::with_hasher(RandomState::new())),
            memory_manager,
            free_ids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create(&self, size: Size, owner_pid: Pid) -> Result<ShmId, ShmError> {
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
        let count = self
            .process_segments
            .get(&owner_pid)
            .map(|v| *v.value())
            .unwrap_or(0);
        if count >= MAX_SEGMENTS_PER_PROCESS {
            return Err(ShmError::ProcessLimitExceeded(
                count,
                MAX_SEGMENTS_PER_PROCESS,
            ));
        }

        // Check global memory limit
        let current_global = GLOBAL_SHM_MEMORY.load(Ordering::Acquire);
        if current_global + size > GLOBAL_SHM_MEMORY_LIMIT {
            return Err(ShmError::GlobalMemoryExceeded(
                current_global,
                GLOBAL_SHM_MEMORY_LIMIT,
            ));
        }

        // Allocate memory through MemoryManager (unified memory accounting)
        let address = self
            .memory_manager
            .allocate(size, owner_pid)
            .map_err(|e| ShmError::AllocationFailed(e.to_string()))?;

        // Try to recycle an ID from the free list, otherwise allocate new
        let segment_id = {
            let mut free_ids = self.free_ids.lock().unwrap();
            if let Some(recycled_id) = free_ids.pop() {
                info!("Recycled segment ID {} for PID {}", recycled_id, owner_pid);
                recycled_id
            } else {
                let mut next_id = self.next_id.write();
                let id = *next_id;
                *next_id += 1;
                id
            }
        };

        let segment = SharedSegment::new(
            segment_id,
            size,
            owner_pid,
            address,
            self.memory_manager.clone(),
        );
        self.segments.insert(segment_id, segment);

        // Update process segment count using alter() for atomic operation
        self.process_segments
            .alter(&owner_pid, |_, count| count + 1);

        // Update global memory
        GLOBAL_SHM_MEMORY.fetch_add(size, Ordering::Release);

        info!(
            "Created shared memory segment {} ({} bytes) for PID {} at address 0x{:x} ({} bytes global memory)",
            segment_id,
            size,
            owner_pid,
            address,
            GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
        );

        Ok(segment_id)
    }

    pub fn attach(&self, segment_id: ShmId, pid: Pid, read_only: bool) -> Result<(), ShmError> {
        let mut segment = self
            .segments
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

    pub fn detach(&self, segment_id: ShmId, pid: Pid) -> Result<(), ShmError> {
        let mut segment = self
            .segments
            .get_mut(&segment_id)
            .ok_or(ShmError::NotFound(segment_id))?;

        segment.detach(pid);

        info!("PID {} detached from segment {}", pid, segment_id);

        Ok(())
    }

    pub fn write(
        &self,
        segment_id: ShmId,
        pid: Pid,
        offset: Size,
        data: &[u8],
    ) -> Result<(), ShmError> {
        let segment = self
            .segments
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
        segment_id: ShmId,
        pid: Pid,
        offset: Size,
        size: Size,
    ) -> Result<Vec<u8>, ShmError> {
        let segment = self
            .segments
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

    pub fn destroy(&self, segment_id: ShmId, pid: Pid) -> Result<(), ShmError> {
        let segment = self
            .segments
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
        let address = segment.address;
        drop(segment);

        self.segments.remove(&segment_id);

        // Deallocate memory through MemoryManager (unified memory accounting)
        if let Err(e) = self.memory_manager.deallocate(address) {
            warn!(
                "Failed to deallocate memory for segment {} at address 0x{:x}: {}",
                segment_id, address, e
            );
        }

        // Add segment ID to free list for recycling
        {
            let mut free_ids = self.free_ids.lock().unwrap();
            free_ids.push(segment_id);
            info!("Added segment ID {} to free list for recycling", segment_id);
        }

        // Update process segment count using alter() for atomic operation
        self.process_segments.alter(&owner_pid, |_, count| {
            let new_count = count.saturating_sub(1);
            if new_count == 0 {
                // Return 0 to signal removal, which we'll handle by checking after
                0
            } else {
                new_count
            }
        });

        // Remove entry if count is 0
        if let Some(entry) = self.process_segments.get(&owner_pid) {
            if *entry.value() == 0 {
                drop(entry);
                self.process_segments.remove(&owner_pid);
            }
        }

        // Reclaim global memory
        GLOBAL_SHM_MEMORY.fetch_sub(size, Ordering::Release);

        info!(
            "Destroyed segment {} (reclaimed {} bytes at 0x{:x}, {} bytes global memory)",
            segment_id,
            size,
            address,
            GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
        );

        Ok(())
    }

    pub fn stats(&self, segment_id: ShmId) -> Result<ShmStats, ShmError> {
        let segment = self
            .segments
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

    pub fn cleanup_process(&self, pid: Pid) -> Size {
        let segment_ids: Vec<u32> = self
            .segments
            .iter()
            .filter(|entry| {
                let s = entry.value();
                s.owner_pid == pid || s.attached_pids.contains(&pid)
            })
            .map(|entry| entry.value().id)
            .collect();

        let mut count = 0;

        for segment_id in segment_ids {
            if let Some(segment) = self.segments.get(&segment_id) {
                if segment.owner_pid == pid {
                    drop(segment);
                    if let Err(e) = self.destroy(segment_id, pid) {
                        warn!(
                            "Failed to destroy segment {} during cleanup: {}",
                            segment_id, e
                        );
                    } else {
                        count += 1;
                    }
                } else {
                    drop(segment);
                    if let Err(e) = self.detach(segment_id, pid) {
                        warn!(
                            "Failed to detach from segment {} during cleanup: {}",
                            segment_id, e
                        );
                    }
                }
            }
        }

        if count > 0 {
            info!(
                "Cleaned up {} shared memory segments for PID {}",
                count, pid
            );
        }

        count
    }

    pub fn get_global_memory_usage(&self) -> Size {
        GLOBAL_SHM_MEMORY.load(Ordering::Relaxed)
    }
}

impl Clone for ShmManager {
    fn clone(&self) -> Self {
        Self {
            segments: Arc::clone(&self.segments),
            next_id: Arc::clone(&self.next_id),
            process_segments: Arc::clone(&self.process_segments),
            memory_manager: self.memory_manager.clone(),
            free_ids: Arc::clone(&self.free_ids),
        }
    }
}
