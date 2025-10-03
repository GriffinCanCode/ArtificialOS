"""
Service Type Definitions
Core types for the BaaS system with strong typing
"""

from typing import Any, Callable, Dict, List, Optional, Protocol
from pydantic import BaseModel, Field
from enum import Enum


class ServiceCategory(str, Enum):
    """Service categories for organization and discovery"""
    STORAGE = "storage"
    COMPUTE = "compute"
    NETWORK = "network"
    AUTH = "auth"
    REALTIME = "realtime"
    MEDIA = "media"
    AI = "ai"
    SYSTEM = "system"


class ToolParameter(BaseModel):
    """Parameter definition for a service tool"""
    name: str
    type: str
    description: str
    required: bool = True
    default: Any = None


class Tool(BaseModel):
    """Service tool definition"""
    id: str = Field(..., description="Unique tool identifier (service.method)")
    name: str
    description: str
    parameters: List[ToolParameter] = Field(default_factory=list)
    returns: str = Field(default="any", description="Return type description")
    category: str = Field(default="general")


class DataModel(BaseModel):
    """Data model schema definition"""
    name: str
    fields: Dict[str, str] = Field(default_factory=dict)
    description: str = ""


class Service(BaseModel):
    """Service definition with capabilities"""
    id: str = Field(..., description="Unique service identifier")
    name: str
    description: str
    category: ServiceCategory
    version: str = Field(default="1.0.0")
    capabilities: List[str] = Field(default_factory=list)
    tools: List[Tool] = Field(default_factory=list)
    data_models: List[DataModel] = Field(default_factory=list)
    examples: List[str] = Field(default_factory=list)
    requires_kernel: bool = Field(default=False)


class ServiceProvider(Protocol):
    """Protocol for service implementations"""
    
    def definition(self) -> Service:
        """Return service definition"""
        ...
    
    async def execute(self, tool_id: str, params: Dict[str, Any]) -> Any:
        """Execute a service tool"""
        ...
    
    async def initialize(self) -> bool:
        """Initialize service resources"""
        ...
    
    async def cleanup(self) -> None:
        """Cleanup service resources"""
        ...


class ServiceContext(BaseModel):
    """Context for service execution"""
    app_id: str
    user_id: Optional[str] = None
    sandbox_pid: Optional[int] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)


class ExecutionResult(BaseModel):
    """Result of service tool execution"""
    success: bool
    data: Any = None
    error: Optional[str] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)

