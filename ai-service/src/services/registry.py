"""
Service Registry
Central registry for service discovery and execution
"""

import logging
from typing import Dict, List, Optional, Any

from .types import Service, ServiceCategory, ServiceContext, ExecutionResult
from .base import BaseService

logger = logging.getLogger(__name__)


class ServiceRegistry:
    """
    Central registry for all backend services.
    Handles discovery, routing, and execution.
    """
    
    def __init__(self):
        self.services: Dict[str, BaseService] = {}
        self.index: Dict[str, Service] = {}
        logger.info("Service registry initialized")
    
    def register(self, service: BaseService) -> None:
        """
        Register a service provider.
        
        Args:
            service: Service provider instance
        """
        definition = service.definition()
        service_id = definition.id
        
        if service_id in self.services:
            logger.warning(f"Service already registered: {service_id}")
            return
        
        self.services[service_id] = service
        self.index[service_id] = definition
        
        logger.info(
            f"Registered: {definition.name} "
            f"({len(definition.tools)} tools, "
            f"{len(definition.capabilities)} capabilities)"
        )
    
    def unregister(self, service_id: str) -> None:
        """Unregister a service"""
        if service_id in self.services:
            del self.services[service_id]
            del self.index[service_id]
            logger.info(f"Unregistered: {service_id}")
    
    def get(self, service_id: str) -> Optional[BaseService]:
        """Get service provider by ID"""
        return self.services.get(service_id)
    
    def get_definition(self, service_id: str) -> Optional[Service]:
        """Get service definition by ID"""
        return self.index.get(service_id)
    
    def list_all(
        self,
        category: Optional[ServiceCategory] = None
    ) -> List[Service]:
        """
        List all registered services.
        
        Args:
            category: Optional category filter
            
        Returns:
            List of service definitions
        """
        services = list(self.index.values())
        
        if category:
            services = [s for s in services if s.category == category]
        
        return services
    
    def discover(self, intent: str, limit: int = 5) -> List[Service]:
        """
        Discover relevant services based on intent.
        Uses keyword matching (can be upgraded to embeddings later).
        
        Args:
            intent: User's intent or app description
            limit: Maximum services to return
            
        Returns:
            List of relevant services, sorted by relevance
        """
        intent_lower = intent.lower()
        scored_services = []
        
        for service in self.index.values():
            score = self._calculate_relevance(intent_lower, service)
            if score > 0:
                scored_services.append((score, service))
        
        # Sort by score descending
        scored_services.sort(key=lambda x: x[0], reverse=True)
        
        # Return top N services
        return [s for _, s in scored_services[:limit]]
    
    def _calculate_relevance(self, intent: str, service: Service) -> float:
        """Calculate relevance score for service given intent"""
        score = 0.0
        
        # Check service name and description
        if service.id in intent or service.name.lower() in intent:
            score += 10.0
        
        if any(word in intent for word in service.description.lower().split()):
            score += 5.0
        
        # Check capabilities
        for capability in service.capabilities:
            if capability.lower().replace("_", " ") in intent:
                score += 3.0
        
        # Check category
        if service.category.value in intent:
            score += 2.0
        
        return score
    
    def get_context(self, service_ids: List[str]) -> str:
        """
        Build context string for LLM with service details.
        
        Args:
            service_ids: List of service IDs to include
            
        Returns:
            Formatted context string
        """
        lines = ["=== AVAILABLE SERVICES ===\n"]
        
        for service_id in service_ids:
            service = self.index.get(service_id)
            if not service:
                continue
            
            lines.append(f"\n## {service.name} ({service.id})")
            lines.append(f"{service.description}\n")
            
            if service.capabilities:
                lines.append("Capabilities:")
                for cap in service.capabilities:
                    lines.append(f"  - {cap}")
            
            if service.tools:
                lines.append("\nTools:")
                for tool in service.tools:
                    params = ", ".join(
                        f"{p.name}: {p.type}"
                        for p in tool.parameters
                    )
                    lines.append(
                        f"  - {tool.id}({params}) -> {tool.returns}"
                    )
                    lines.append(f"    {tool.description}")
            
            if service.data_models:
                lines.append("\nData Models:")
                for model in service.data_models:
                    fields = ", ".join(
                        f"{k}: {v}"
                        for k, v in model.fields.items()
                    )
                    lines.append(f"  - {model.name}({fields})")
        
        return "\n".join(lines)
    
    async def execute(
        self,
        tool_id: str,
        params: Dict[str, Any],
        context: Optional[ServiceContext] = None
    ) -> ExecutionResult:
        """
        Execute a service tool by ID.
        
        Args:
            tool_id: Full tool ID (e.g., "storage.file.read")
            params: Tool parameters
            context: Execution context
            
        Returns:
            Execution result
        """
        # Parse service ID from tool ID
        parts = tool_id.split(".", 1)
        if len(parts) < 2:
            return ExecutionResult(
                success=False,
                error=f"Invalid tool ID format: {tool_id}"
            )
        
        service_id = parts[0]
        
        # Get service provider
        service = self.services.get(service_id)
        if not service:
            return ExecutionResult(
                success=False,
                error=f"Service not found: {service_id}"
            )
        
        # Update context if provided
        if context and service.context is None:
            service.context = context
        
        # Execute with safety wrapper
        return await service.safe_execute(tool_id, params)
    
    def get_stats(self) -> Dict[str, Any]:
        """Get registry statistics"""
        total_tools = sum(
            len(s.tools)
            for s in self.index.values()
        )
        
        categories = {}
        for service in self.index.values():
            cat = service.category.value
            categories[cat] = categories.get(cat, 0) + 1
        
        return {
            "total_services": len(self.services),
            "total_tools": total_tools,
            "categories": categories,
            "service_ids": list(self.services.keys())
        }

