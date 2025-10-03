"""
Kernel Client
gRPC client for communicating with the Rust kernel
"""

import logging
from typing import Optional, Dict, Any, List
import grpc
import sys
from pathlib import Path

# Add current directory to path for protobuf imports
sys.path.insert(0, str(Path(__file__).parent))

# Import generated protobuf code
import kernel_pb2
import kernel_pb2_grpc

logger = logging.getLogger(__name__)


class KernelClient:
    """Client for interacting with the Rust kernel via gRPC"""
    
    def __init__(self, kernel_address: str = "localhost:50051"):
        """
        Initialize the kernel client.
        
        Args:
            kernel_address: Address of the kernel gRPC server (host:port)
        """
        self.kernel_address = kernel_address
        self.channel: Optional[grpc.Channel] = None
        self.stub: Optional[kernel_pb2_grpc.KernelServiceStub] = None
        logger.info(f"Kernel client initialized (will connect to {kernel_address})")
    
    def connect(self):
        """Establish connection to the kernel"""
        if self.channel is None:
            logger.info(f"Connecting to kernel at {self.kernel_address}...")
            self.channel = grpc.insecure_channel(self.kernel_address)
            self.stub = kernel_pb2_grpc.KernelServiceStub(self.channel)
            logger.info("✅ Connected to kernel")
    
    def disconnect(self):
        """Close the connection to the kernel"""
        if self.channel:
            self.channel.close()
            self.channel = None
            self.stub = None
            logger.info("Disconnected from kernel")
    
    def __enter__(self):
        """Context manager entry"""
        self.connect()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.disconnect()
    
    # ========================================================================
    # Process Management
    # ========================================================================
    
    def create_process(
        self,
        name: str,
        priority: int = 5,
        sandbox_level: str = "STANDARD"
    ) -> Optional[int]:
        """
        Create a new sandboxed process.
        
        Args:
            name: Process name
            priority: Process priority (0-10)
            sandbox_level: "MINIMAL", "STANDARD", or "PRIVILEGED"
        
        Returns:
            Process ID (PID) if successful, None otherwise
        """
        if not self.stub:
            self.connect()
        
        # Map string to enum
        level_map = {
            "MINIMAL": kernel_pb2.MINIMAL,
            "STANDARD": kernel_pb2.STANDARD,
            "PRIVILEGED": kernel_pb2.PRIVILEGED
        }
        
        request = kernel_pb2.CreateProcessRequest(
            name=name,
            priority=priority,
            sandbox_level=level_map.get(sandbox_level, kernel_pb2.STANDARD)
        )
        
        try:
            response = self.stub.CreateProcess(request)
            if response.success:
                logger.info(f"Created process: {name} (PID: {response.pid})")
                return response.pid
            else:
                logger.error(f"Failed to create process: {response.error}")
                return None
        except grpc.RpcError as e:
            logger.error(f"gRPC error creating process: {e}")
            return None
    
    def update_sandbox(
        self,
        pid: int,
        capabilities: List[str],
        allowed_paths: List[str] = None,
        blocked_paths: List[str] = None,
        limits: Dict[str, int] = None
    ) -> bool:
        """
        Update sandbox permissions for a process.
        
        Args:
            pid: Process ID
            capabilities: List of capability names (e.g., ["READ_FILE", "WRITE_FILE"])
            allowed_paths: Paths the process can access
            blocked_paths: Paths the process cannot access
            limits: Resource limits dict
        
        Returns:
            True if successful
        """
        if not self.stub:
            self.connect()
        
        # Map capability names to enum values
        cap_map = {
            "READ_FILE": kernel_pb2.READ_FILE,
            "WRITE_FILE": kernel_pb2.WRITE_FILE,
            "CREATE_FILE": kernel_pb2.CREATE_FILE,
            "DELETE_FILE": kernel_pb2.DELETE_FILE,
            "LIST_DIRECTORY": kernel_pb2.LIST_DIRECTORY,
            "SPAWN_PROCESS": kernel_pb2.SPAWN_PROCESS,
            "KILL_PROCESS": kernel_pb2.KILL_PROCESS,
            "NETWORK_ACCESS": kernel_pb2.NETWORK_ACCESS,
            "BIND_PORT": kernel_pb2.BIND_PORT,
            "SYSTEM_INFO": kernel_pb2.SYSTEM_INFO,
            "TIME_ACCESS": kernel_pb2.TIME_ACCESS,
            "SEND_MESSAGE": kernel_pb2.SEND_MESSAGE,
            "RECEIVE_MESSAGE": kernel_pb2.RECEIVE_MESSAGE,
        }
        
        caps = [cap_map[c] for c in capabilities if c in cap_map]
        
        request = kernel_pb2.UpdateSandboxRequest(
            pid=pid,
            capabilities=caps,
            allowed_paths=allowed_paths or [],
            blocked_paths=blocked_paths or []
        )
        
        if limits:
            request.limits.CopyFrom(kernel_pb2.ResourceLimits(
                max_memory_bytes=limits.get("max_memory_bytes", 512 * 1024 * 1024),
                max_cpu_time_ms=limits.get("max_cpu_time_ms", 60000),
                max_file_descriptors=limits.get("max_file_descriptors", 100),
                max_processes=limits.get("max_processes", 10),
                max_network_connections=limits.get("max_network_connections", 20)
            ))
        
        try:
            response = self.stub.UpdateSandbox(request)
            if response.success:
                logger.info(f"Updated sandbox for PID {pid}")
                return True
            else:
                logger.error(f"Failed to update sandbox: {response.error}")
                return False
        except grpc.RpcError as e:
            logger.error(f"gRPC error updating sandbox: {e}")
            return False
    
    # ========================================================================
    # System Calls
    # ========================================================================
    
    def execute_syscall(self, pid: int, syscall_type: str, **kwargs) -> Optional[Dict[str, Any]]:
        """
        Execute a system call on behalf of a process.
        
        Args:
            pid: Process ID
            syscall_type: Type of syscall (e.g., "read_file", "get_system_info")
            **kwargs: Syscall-specific parameters
        
        Returns:
            Result dict with 'success', 'data', 'error', or 'permission_denied'
        """
        if not self.stub:
            self.connect()
        
        # Build syscall request based on type
        request = kernel_pb2.SyscallRequest(pid=pid)
        
        try:
            if syscall_type == "read_file":
                request.read_file.CopyFrom(kernel_pb2.ReadFileCall(path=kwargs["path"]))
            elif syscall_type == "write_file":
                request.write_file.CopyFrom(kernel_pb2.WriteFileCall(
                    path=kwargs["path"],
                    data=kwargs.get("data", b"")
                ))
            elif syscall_type == "create_file":
                request.create_file.CopyFrom(kernel_pb2.CreateFileCall(path=kwargs["path"]))
            elif syscall_type == "delete_file":
                request.delete_file.CopyFrom(kernel_pb2.DeleteFileCall(path=kwargs["path"]))
            elif syscall_type == "list_directory":
                request.list_directory.CopyFrom(kernel_pb2.ListDirectoryCall(path=kwargs["path"]))
            elif syscall_type == "file_exists":
                request.file_exists.CopyFrom(kernel_pb2.FileExistsCall(path=kwargs["path"]))
            elif syscall_type == "spawn_process":
                request.spawn_process.CopyFrom(kernel_pb2.SpawnProcessCall(
                    command=kwargs["command"],
                    args=kwargs.get("args", [])
                ))
            elif syscall_type == "kill_process":
                request.kill_process.CopyFrom(kernel_pb2.KillProcessCall(
                    target_pid=kwargs["target_pid"]
                ))
            elif syscall_type == "get_system_info":
                request.get_system_info.CopyFrom(kernel_pb2.GetSystemInfoCall())
            elif syscall_type == "get_current_time":
                request.get_current_time.CopyFrom(kernel_pb2.GetCurrentTimeCall())
            elif syscall_type == "get_env_var":
                request.get_env_var.CopyFrom(kernel_pb2.GetEnvVarCall(key=kwargs["key"]))
            elif syscall_type == "network_request":
                request.network_request.CopyFrom(kernel_pb2.NetworkRequestCall(url=kwargs["url"]))
            else:
                logger.error(f"Unknown syscall type: {syscall_type}")
                return {"error": f"Unknown syscall type: {syscall_type}"}
            
            # Execute syscall
            response = self.stub.ExecuteSyscall(request)
            
            # Parse response
            if response.HasField("success"):
                return {
                    "success": True,
                    "data": response.success.data
                }
            elif response.HasField("error"):
                return {
                    "error": response.error.message
                }
            elif response.HasField("permission_denied"):
                return {
                    "permission_denied": response.permission_denied.reason
                }
            else:
                return {"error": "Unknown response type"}
        
        except grpc.RpcError as e:
            logger.error(f"gRPC error executing syscall: {e}")
            return {"error": str(e)}
    
    # ========================================================================
    # Convenience Methods
    # ========================================================================
    
    def read_file(self, pid: int, path: str) -> Optional[bytes]:
        """Read a file"""
        result = self.execute_syscall(pid, "read_file", path=path)
        if result and result.get("success"):
            return result.get("data")
        return None
    
    def write_file(self, pid: int, path: str, data: bytes) -> bool:
        """Write a file"""
        result = self.execute_syscall(pid, "write_file", path=path, data=data)
        return result and result.get("success", False)
    
    def file_exists(self, pid: int, path: str) -> bool:
        """Check if a file exists"""
        result = self.execute_syscall(pid, "file_exists", path=path)
        if result and result.get("success"):
            data = result.get("data", b"\x00")
            return data[0] == 1 if data else False
        return False
    
    def list_directory(self, pid: int, path: str) -> List[str]:
        """List directory contents"""
        result = self.execute_syscall(pid, "list_directory", path=path)
        if result and result.get("success"):
            import json
            data = result.get("data", b"[]")
            return json.loads(data.decode("utf-8"))
        return []
    
    def get_system_info(self, pid: int) -> Optional[Dict[str, str]]:
        """Get system information"""
        result = self.execute_syscall(pid, "get_system_info")
        if result and result.get("success"):
            import json
            data = result.get("data", b"{}")
            return json.loads(data.decode("utf-8"))
        return None


# Global kernel client instance
_kernel_client: Optional[KernelClient] = None


def get_kernel_client() -> KernelClient:
    """Get the global kernel client instance"""
    global _kernel_client
    if _kernel_client is None:
        _kernel_client = KernelClient()
        _kernel_client.connect()
    return _kernel_client

