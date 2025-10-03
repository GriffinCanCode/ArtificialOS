"""
Storage Service
File I/O and data persistence via kernel
"""

import json
import logging
from typing import Any, Dict, Optional

from ..base import BaseService
from ..types import (
    Service,
    ServiceCategory,
    Tool,
    ToolParameter,
    DataModel,
    ServiceContext,
)

logger = logging.getLogger(__name__)


class StorageService(BaseService):
    """
    Storage service for file operations and data persistence.
    Uses kernel syscalls for sandboxed file access.
    """
    
    def __init__(
        self,
        context: Optional[ServiceContext] = None,
        kernel_tools: Optional[Any] = None
    ):
        super().__init__(context)
        self.kernel_tools = kernel_tools
    
    def definition(self) -> Service:
        return Service(
            id="storage",
            name="Storage Service",
            description="File I/O and data persistence",
            category=ServiceCategory.STORAGE,
            version="1.0.0",
            capabilities=[
                "file_read",
                "file_write",
                "file_list",
                "data_persistence",
                "json_storage"
            ],
            tools=[
                Tool(
                    id="storage.file.read",
                    name="Read File",
                    description="Read file contents",
                    parameters=[
                        ToolParameter(
                            name="path",
                            type="string",
                            description="File path to read"
                        )
                    ],
                    returns="string",
                    category="file"
                ),
                Tool(
                    id="storage.file.write",
                    name="Write File",
                    description="Write content to file",
                    parameters=[
                        ToolParameter(
                            name="path",
                            type="string",
                            description="File path to write"
                        ),
                        ToolParameter(
                            name="content",
                            type="string",
                            description="Content to write"
                        )
                    ],
                    returns="boolean",
                    category="file"
                ),
                Tool(
                    id="storage.file.list",
                    name="List Directory",
                    description="List files in directory",
                    parameters=[
                        ToolParameter(
                            name="path",
                            type="string",
                            description="Directory path"
                        )
                    ],
                    returns="array",
                    category="file"
                ),
                Tool(
                    id="storage.data.save",
                    name="Save Data",
                    description="Save JSON data to storage",
                    parameters=[
                        ToolParameter(
                            name="key",
                            type="string",
                            description="Storage key"
                        ),
                        ToolParameter(
                            name="data",
                            type="any",
                            description="Data to save"
                        )
                    ],
                    returns="boolean",
                    category="data"
                ),
                Tool(
                    id="storage.data.load",
                    name="Load Data",
                    description="Load JSON data from storage",
                    parameters=[
                        ToolParameter(
                            name="key",
                            type="string",
                            description="Storage key"
                        )
                    ],
                    returns="any",
                    category="data"
                ),
            ],
            data_models=[
                DataModel(
                    name="File",
                    fields={
                        "path": "string",
                        "content": "string",
                        "size": "number"
                    }
                ),
                DataModel(
                    name="StorageEntry",
                    fields={
                        "key": "string",
                        "data": "any",
                        "timestamp": "number"
                    }
                )
            ],
            requires_kernel=True
        )
    
    async def execute(self, tool_id: str, params: Dict[str, Any]) -> Any:
        """Execute storage tool"""
        
        if tool_id == "storage.file.read":
            return await self._file_read(params["path"])
        
        elif tool_id == "storage.file.write":
            return await self._file_write(
                params["path"],
                params["content"]
            )
        
        elif tool_id == "storage.file.list":
            return await self._file_list(params["path"])
        
        elif tool_id == "storage.data.save":
            return await self._data_save(
                params["key"],
                params["data"]
            )
        
        elif tool_id == "storage.data.load":
            return await self._data_load(params["key"])
        
        else:
            raise ValueError(f"Unknown tool: {tool_id}")
    
    async def _file_read(self, path: str) -> str:
        """Read file via kernel"""
        if not self.kernel_tools:
            raise RuntimeError("Kernel tools not available")
        
        content = self.kernel_tools.file_read(path)
        if content is None:
            raise FileNotFoundError(f"File not found: {path}")
        
        return content
    
    async def _file_write(self, path: str, content: str) -> bool:
        """Write file via kernel"""
        if not self.kernel_tools:
            raise RuntimeError("Kernel tools not available")
        
        success = self.kernel_tools.file_write(path, content)
        if not success:
            raise IOError(f"Failed to write file: {path}")
        
        return True
    
    async def _file_list(self, path: str) -> list:
        """List directory via kernel"""
        if not self.kernel_tools:
            raise RuntimeError("Kernel tools not available")
        
        files = self.kernel_tools.directory_list(path)
        return files
    
    async def _data_save(self, key: str, data: Any) -> bool:
        """Save data as JSON file"""
        # Use app-specific storage directory
        app_id = self.context.app_id if self.context else "default"
        path = f"/tmp/ai-os-storage/{app_id}/{key}.json"
        
        # Serialize data
        content = json.dumps(data, indent=2)
        
        return await self._file_write(path, content)
    
    async def _data_load(self, key: str) -> Any:
        """Load data from JSON file"""
        app_id = self.context.app_id if self.context else "default"
        path = f"/tmp/ai-os-storage/{app_id}/{key}.json"
        
        content = await self._file_read(path)
        return json.loads(content)

