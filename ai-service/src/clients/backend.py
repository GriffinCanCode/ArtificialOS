"""
Backend Service Client
Queries the Go backend for available service providers.
"""

import logging
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
import httpx

logger = logging.getLogger(__name__)


@dataclass
class ToolParameter:
    """Tool parameter definition"""
    name: str
    type: str
    description: str
    required: bool


@dataclass
class ToolDefinition:
    """Backend service tool definition"""
    id: str
    name: str
    description: str
    parameters: List[ToolParameter]
    returns: str


@dataclass
class ServiceDefinition:
    """Backend service provider definition"""
    id: str
    name: str
    description: str
    category: str
    capabilities: List[str]
    tools: List[ToolDefinition]


class BackendClient:
    """
    Client for querying backend services.
    Discovers available service providers and their tools.
    """
    
    def __init__(self, backend_url: str = "http://localhost:8000", timeout: float = 5.0):
        """
        Initialize backend client.
        
        Args:
            backend_url: Base URL of the Go backend
            timeout: Request timeout in seconds
        """
        self.backend_url = backend_url.rstrip("/")
        self.timeout = timeout
        self._client = httpx.Client(timeout=timeout)
        logger.info(f"BackendClient initialized for {self.backend_url}")
    
    def discover_services(self, category: Optional[str] = None) -> List[ServiceDefinition]:
        """
        Discover available services from the backend.
        
        Args:
            category: Optional category filter (storage, auth, system, etc.)
            
        Returns:
            List of service definitions
        """
        try:
            url = f"{self.backend_url}/services"
            params = {"category": category} if category else {}
            
            response = self._client.get(url, params=params)
            response.raise_for_status()
            
            data = response.json()
            services = data.get("services", [])
            
            # Convert to ServiceDefinition objects
            result = []
            for svc in services:
                tools = []
                for tool_data in svc.get("tools", []):
                    params_list = []
                    for param in tool_data.get("parameters", []):
                        params_list.append(ToolParameter(
                            name=param["name"],
                            type=param["type"],
                            description=param["description"],
                            required=param["required"]
                        ))
                    
                    tools.append(ToolDefinition(
                        id=tool_data["id"],
                        name=tool_data["name"],
                        description=tool_data["description"],
                        parameters=params_list,
                        returns=tool_data["returns"]
                    ))
                
                result.append(ServiceDefinition(
                    id=svc["id"],
                    name=svc["name"],
                    description=svc["description"],
                    category=svc["category"],
                    capabilities=svc.get("capabilities", []),
                    tools=tools
                ))
            
            logger.info(f"Discovered {len(result)} services with {sum(len(s.tools) for s in result)} total tools")
            return result
            
        except httpx.HTTPError as e:
            logger.warning(f"Failed to discover services: {e}")
            return []
        except Exception as e:
            logger.error(f"Unexpected error discovering services: {e}", exc_info=True)
            return []
    
    def get_tools_description(self, services: List[ServiceDefinition]) -> str:
        """
        Format service tools as a string for AI context.
        
        Args:
            services: List of service definitions
            
        Returns:
            Formatted tool descriptions
        """
        if not services:
            return ""
        
        lines = ["\nBACKEND SERVICES:"]
        
        for service in services:
            lines.append(f"\n{service.category.upper()} - {service.name}:")
            for tool in service.tools:
                params_str = ", ".join(
                    f"{p.name}: {p.type}" + (" (required)" if p.required else " (optional)")
                    for p in tool.parameters
                )
                params_display = f"({params_str})" if params_str else "(no params)"
                lines.append(f"  - {tool.id}: {tool.description} {params_display}")
        
        return "\n".join(lines)
    
    def health_check(self) -> bool:
        """
        Check if backend is reachable.
        
        Returns:
            True if backend is healthy
        """
        try:
            response = self._client.get(f"{self.backend_url}/health", timeout=2.0)
            return response.status_code == 200
        except:
            return False
    
    def close(self):
        """Close HTTP client"""
        self._client.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()

