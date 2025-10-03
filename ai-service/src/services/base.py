"""
Base Service Implementation
Abstract base class for all service providers
"""

import logging
from abc import ABC, abstractmethod
from typing import Any, Dict, Optional

from .types import Service, ServiceContext, ExecutionResult, Tool

logger = logging.getLogger(__name__)


class BaseService(ABC):
    """
    Abstract base class for service implementations.
    All services inherit from this and implement execute().
    """
    
    def __init__(self, context: Optional[ServiceContext] = None):
        self.context = context
        self._initialized = False
        logger.info(f"Service created: {self.__class__.__name__}")
    
    @abstractmethod
    def definition(self) -> Service:
        """Return service definition with tools and capabilities"""
        pass
    
    @abstractmethod
    async def execute(self, tool_id: str, params: Dict[str, Any]) -> Any:
        """
        Execute a service tool.
        
        Args:
            tool_id: Full tool ID (e.g., "storage.file.read")
            params: Tool parameters
            
        Returns:
            Tool execution result
        """
        pass
    
    async def initialize(self) -> bool:
        """
        Initialize service resources (optional override).
        Called once before first use.
        """
        self._initialized = True
        return True
    
    async def cleanup(self) -> None:
        """
        Cleanup service resources (optional override).
        Called when service is destroyed.
        """
        self._initialized = False
    
    def get_tool(self, tool_id: str) -> Optional[Tool]:
        """Get tool definition by ID"""
        service_def = self.definition()
        for tool in service_def.tools:
            if tool.id == tool_id:
                return tool
        return None
    
    async def safe_execute(
        self,
        tool_id: str,
        params: Dict[str, Any]
    ) -> ExecutionResult:
        """
        Execute with error handling and result wrapping.
        
        Args:
            tool_id: Tool ID to execute
            params: Tool parameters
            
        Returns:
            Execution result with success status
        """
        try:
            # Ensure initialized
            if not self._initialized:
                await self.initialize()
            
            # Execute tool
            result = await self.execute(tool_id, params)
            
            return ExecutionResult(
                success=True,
                data=result,
                metadata={"tool_id": tool_id}
            )
            
        except Exception as e:
            logger.error(f"Service execution error: {e}", exc_info=True)
            return ExecutionResult(
                success=False,
                error=str(e),
                metadata={"tool_id": tool_id}
            )

