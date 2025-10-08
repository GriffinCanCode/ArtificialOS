/*!
 * Syscall Implementations
 *
 * Category-specific syscall implementations for SyscallExecutorWithIpc:
 * - fd: File descriptor operations
 * - fs: Filesystem operations
 * - handle: Unified file handle abstraction
 * - memory: Memory management
 * - mmap: Memory-mapped files
 * - network: Network operations
 * - process: Process management
 * - scheduler: CPU scheduling
 * - signals: Signal handling
 * - system: System information
 * - time: Time and sleep operations
 * - vfs_adapter: VFS integration layer
 */

pub mod fd;
pub mod fs;
pub mod handle;
pub mod memory;
pub mod mmap;
pub mod network;
pub mod process;
pub mod scheduler;
pub mod signals;
pub mod system;
pub mod time;
pub mod vfs_adapter;

// Re-export commonly used types
pub use fd::FdManager;
pub use handle::FileHandle;
pub use network::{Socket, SocketManager, SocketStats};

