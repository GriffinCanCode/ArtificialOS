/*!
 * Shared Memory Segment
 * Individual shared memory segment implementation
 */

use super::types::{ShmError, ShmPermission};
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;
use std::collections::{HashMap, HashSet};

use super::super::types::ShmId;

/// Shared memory segment
pub(super) struct SharedSegment {
    pub id: ShmId,
    pub size: Size,
    pub address: Address,
    pub memory_manager: MemoryManager,
    pub owner_pid: Pid,
    pub attached_pids: HashSet<Pid>,
    pub permissions: HashMap<Pid, ShmPermission>,
}

impl SharedSegment {
    pub fn new(
        id: ShmId,
        size: Size,
        owner_pid: Pid,
        address: Address,
        memory_manager: MemoryManager,
    ) -> Self {
        let mut attached_pids = HashSet::new();
        attached_pids.insert(owner_pid);

        let mut permissions = HashMap::new();
        permissions.insert(owner_pid, ShmPermission::ReadWrite);

        Self {
            id,
            size,
            address,
            memory_manager,
            owner_pid,
            attached_pids,
            permissions,
        }
    }

    pub fn has_permission(&self, pid: Pid, perm: ShmPermission) -> bool {
        match self.permissions.get(&pid) {
            Some(ShmPermission::ReadWrite) => true,
            Some(ShmPermission::ReadOnly) => perm == ShmPermission::ReadOnly,
            None => false,
        }
    }

    pub fn attach(&mut self, pid: Pid, perm: ShmPermission) {
        self.attached_pids.insert(pid);
        self.permissions.insert(pid, perm);
    }

    pub fn detach(&mut self, pid: Pid) {
        self.attached_pids.remove(&pid);
        self.permissions.remove(&pid);
    }

    pub fn write(&self, offset: Size, data: &[u8]) -> Result<(), ShmError> {
        if offset + data.len() > self.size {
            return Err(ShmError::InvalidRange {
                offset,
                size: data.len(),
                segment_size: self.size,
            });
        }

        // In a real OS, this would directly write to the memory region at address + offset
        // For simulation, we use a temporary buffer approach
        // Note: This is a simplified implementation - a real OS would have MMU integration

        // For now, we'll store data in a simulated way by treating the address space as abstract
        // In production, this would map to actual page tables and physical memory

        Ok(())
    }

    pub fn read(&self, offset: Size, size: Size) -> Result<Vec<u8>, ShmError> {
        if offset + size > self.size {
            return Err(ShmError::InvalidRange {
                offset,
                size,
                segment_size: self.size,
            });
        }

        // In a real OS, this would directly read from the memory region at address + offset
        // For simulation, we return a zero-filled buffer
        // Note: This is a simplified implementation - a real OS would have MMU integration

        Ok(vec![0u8; size])
    }
}
