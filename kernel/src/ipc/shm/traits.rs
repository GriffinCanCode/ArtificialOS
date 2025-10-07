/*!
 * Shared Memory Trait Implementations
 * Trait implementations for shared memory manager
 */

use super::super::core::traits::SharedMemory;
use super::super::core::types::{IpcResult, ShmId};
use super::manager::ShmManager;
use super::types::ShmStats;
use crate::core::types::{Pid, Size};

// Implement SharedMemory trait
impl SharedMemory for ShmManager {
    fn create(&self, size: Size, owner_pid: Pid) -> IpcResult<ShmId> {
        self.create(size, owner_pid).map_err(|e| e.into())
    }

    fn attach(&self, segment_id: ShmId, pid: Pid, read_only: bool) -> IpcResult<()> {
        self.attach(segment_id, pid, read_only)
            .map_err(|e| e.into())
    }

    fn detach(&self, segment_id: ShmId, pid: Pid) -> IpcResult<()> {
        self.detach(segment_id, pid).map_err(|e| e.into())
    }

    fn write(&self, segment_id: ShmId, pid: Pid, offset: Size, data: &[u8]) -> IpcResult<()> {
        self.write(segment_id, pid, offset, data)
            .map_err(|e| e.into())
    }

    fn read(&self, segment_id: ShmId, pid: Pid, offset: Size, size: Size) -> IpcResult<Vec<u8>> {
        self.read(segment_id, pid, offset, size)
            .map_err(|e| e.into())
    }

    fn destroy(&self, segment_id: ShmId, pid: Pid) -> IpcResult<()> {
        self.destroy(segment_id, pid).map_err(|e| e.into())
    }

    fn stats(&self, segment_id: ShmId) -> IpcResult<ShmStats> {
        self.stats(segment_id).map_err(|e| e.into())
    }
}
