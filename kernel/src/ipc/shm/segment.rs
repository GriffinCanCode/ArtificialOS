/*!
 * Shared Memory Segment
 * Individual shared memory segment implementation
 */

use super::types::{ShmError, ShmPermission};
use crate::core::memory::CowMemory;
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;
use ahash::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use super::super::types::ShmId;

pub(super) struct SharedSegment {
    pub id: ShmId,
    pub size: Size,
    pub address: Address,
    pub memory_manager: MemoryManager,
    pub owner_pid: Pid,
    pub attached_pids: HashSet<Pid>,
    pub permissions: HashMap<Pid, ShmPermission>,
    pub cow_data: Arc<RwLock<Option<CowMemory>>>,
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

        let mut permissions = HashMap::default();
        permissions.insert(owner_pid, ShmPermission::ReadWrite);

        let cow_data = Arc::new(RwLock::new(Some(CowMemory::new(vec![0u8; size]))));

        Self {
            id,
            size,
            address,
            memory_manager,
            owner_pid,
            attached_pids,
            permissions,
            cow_data,
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

        if let Ok(mut cow_lock) = self.cow_data.write() {
            if let Some(ref mut cow) = *cow_lock {
                cow.write(|buffer| {
                    let end = offset + data.len();
                    buffer[offset..end].copy_from_slice(data);
                });
                return Ok(());
            }
        }

        self.memory_manager
            .write_bytes(self.address + offset, data)
            .map_err(|_| ShmError::InvalidRange {
                offset,
                size: data.len(),
                segment_size: self.size,
            })
    }

    pub fn read(&self, offset: Size, size: Size) -> Result<Vec<u8>, ShmError> {
        if offset + size > self.size {
            return Err(ShmError::InvalidRange {
                offset,
                size,
                segment_size: self.size,
            });
        }

        if let Ok(cow_lock) = self.cow_data.read() {
            if let Some(ref cow) = *cow_lock {
                return Ok(cow.read(|buffer| {
                    let end = offset + size;
                    buffer[offset..end].to_vec()
                }));
            }
        }

        self.memory_manager
            .read_bytes(self.address + offset, size)
            .map_err(|_| ShmError::InvalidRange {
                offset,
                size,
                segment_size: self.size,
            })
    }
}
