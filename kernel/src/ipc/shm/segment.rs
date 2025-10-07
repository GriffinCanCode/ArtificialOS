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

    /// Check if a process has at least the required permission level
    pub fn has_permission(&self, pid: Pid, required: ShmPermission) -> bool {
        self.permissions
            .get(&pid)
            .map(|perm| perm.satisfies(required))
            .unwrap_or(false)
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

        // Write to memory through the MemoryManager
        // The address is the base address of this shared memory segment
        let write_address = self.address + offset;

        // Use memory manager to write data to the shared memory region
        // This writes to our simulated physical memory storage
        self.memory_manager.write_bytes(write_address, data)
            .map_err(|_| ShmError::InvalidRange {
                offset,
                size: data.len(),
                segment_size: self.size,
            })?;

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

        // Read from memory through the MemoryManager
        // The address is the base address of this shared memory segment
        let read_address = self.address + offset;

        // Use memory manager to read data from the shared memory region
        // This reads from our simulated physical memory storage
        let data = self.memory_manager.read_bytes(read_address, size)
            .map_err(|_| ShmError::InvalidRange {
                offset,
                size,
                segment_size: self.size,
            })?;

        Ok(data)
    }
}
