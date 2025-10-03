"""
Kernel Tools
Tools that execute via the kernel's sandboxed syscall system
"""

import logging
from typing import Optional, Dict, Any
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from kernel_client import get_kernel_client

logger = logging.getLogger(__name__)


class KernelTools:
    """Tools that execute real OS operations via the kernel"""
    
    def __init__(self, default_pid: Optional[int] = None):
        """
        Initialize kernel tools.
        
        Args:
            default_pid: Default process ID to use for syscalls
        """
        self.kernel_client = get_kernel_client()
        self.default_pid = default_pid
        logger.info("Kernel tools initialized")
    
    def set_process(self, pid: int):
        """Set the default process ID for syscalls"""
        self.default_pid = pid
        logger.info(f"Set default PID to {pid}")
    
    def create_sandboxed_process(
        self,
        name: str = "ai-app",
        sandbox_level: str = "STANDARD"
    ) -> Optional[int]:
        """
        Create a new sandboxed process for tool execution.
        
        Args:
            name: Process name
            sandbox_level: "MINIMAL", "STANDARD", or "PRIVILEGED"
        
        Returns:
            Process ID if successful
        """
        pid = self.kernel_client.create_process(name, priority=5, sandbox_level=sandbox_level)
        
        if pid:
            # Configure sandbox with appropriate capabilities
            capabilities = ["READ_FILE", "WRITE_FILE", "SYSTEM_INFO", "TIME_ACCESS"]
            
            if sandbox_level == "PRIVILEGED":
                capabilities.extend([
                    "CREATE_FILE", "DELETE_FILE", "LIST_DIRECTORY",
                    "SPAWN_PROCESS", "NETWORK_ACCESS"
                ])
            
            self.kernel_client.update_sandbox(
                pid=pid,
                capabilities=capabilities,
                allowed_paths=["/tmp", "/var/tmp"],
                blocked_paths=["/etc/passwd", "/etc/shadow"]
            )
            
            self.default_pid = pid
            logger.info(f"Created sandboxed process: {name} (PID: {pid})")
        
        return pid
    
    # ========================================================================
    # File System Tools
    # ========================================================================
    
    def file_read(self, path: str, pid: Optional[int] = None) -> Optional[str]:
        """
        Read a file from the filesystem.
        
        Args:
            path: File path to read
            pid: Process ID (uses default if not provided)
        
        Returns:
            File contents as string, or None if failed
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return None
        
        data = self.kernel_client.read_file(pid, path)
        if data:
            try:
                return data.decode("utf-8")
            except UnicodeDecodeError:
                logger.warning(f"File {path} is not UTF-8, returning raw bytes")
                return str(data)
        return None
    
    def file_write(self, path: str, content: str, pid: Optional[int] = None) -> bool:
        """
        Write content to a file.
        
        Args:
            path: File path to write
            content: Content to write
            pid: Process ID (uses default if not provided)
        
        Returns:
            True if successful
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return False
        
        data = content.encode("utf-8")
        return self.kernel_client.write_file(pid, path, data)
    
    def file_exists(self, path: str, pid: Optional[int] = None) -> bool:
        """
        Check if a file exists.
        
        Args:
            path: File path to check
            pid: Process ID (uses default if not provided)
        
        Returns:
            True if file exists
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return False
        
        return self.kernel_client.file_exists(pid, path)
    
    def directory_list(self, path: str, pid: Optional[int] = None) -> list:
        """
        List directory contents.
        
        Args:
            path: Directory path to list
            pid: Process ID (uses default if not provided)
        
        Returns:
            List of file/directory names
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return []
        
        return self.kernel_client.list_directory(pid, path)
    
    # ========================================================================
    # System Tools
    # ========================================================================
    
    def system_info(self, pid: Optional[int] = None) -> Optional[Dict[str, str]]:
        """
        Get system information.
        
        Args:
            pid: Process ID (uses default if not provided)
        
        Returns:
            Dict with 'os', 'arch', 'family' keys
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return None
        
        return self.kernel_client.get_system_info(pid)
    
    def execute_command(
        self,
        command: str,
        args: list = None,
        pid: Optional[int] = None
    ) -> Optional[Dict[str, Any]]:
        """
        Execute a system command (if permissions allow).
        
        Args:
            command: Command to execute
            args: Command arguments
            pid: Process ID (uses default if not provided)
        
        Returns:
            Dict with stdout, stderr, exit_code
        """
        pid = pid or self.default_pid
        if not pid:
            logger.error("No process ID set")
            return None
        
        result = self.kernel_client.execute_syscall(
            pid,
            "spawn_process",
            command=command,
            args=args or []
        )
        
        if result and result.get("success"):
            import json
            data = result.get("data", b"{}")
            return json.loads(data.decode("utf-8"))
        elif result and result.get("permission_denied"):
            logger.warning(f"Permission denied: {result['permission_denied']}")
        
        return None


# Global instance
_kernel_tools: Optional[KernelTools] = None


def get_kernel_tools() -> KernelTools:
    """Get the global kernel tools instance"""
    global _kernel_tools
    if _kernel_tools is None:
        _kernel_tools = KernelTools()
    return _kernel_tools

