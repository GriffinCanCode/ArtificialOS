"""Tool Registry - Modular system with strong typing."""

from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field

from core import get_logger
from .tool_categories import (
    register_ui_tools,
    register_browser_tools,
    register_app_tools,
    register_system_tools,
    register_math_tools,
)

logger = get_logger(__name__)


# ============================================================================
# Tool Registry Schema
# ============================================================================

class ToolDefinition(BaseModel):
    """Definition of a callable tool."""
    id: str = Field(..., description="Unique tool identifier")
    name: str = Field(..., description="Human-readable name")
    description: str = Field(..., description="What the tool does")
    parameters: Dict[str, Any] = Field(default_factory=dict, description="Parameter schema")
    category: str = Field(default="general", description="Tool category (compute, ui, system, etc.)")


class ToolRegistry:
    """
    Modular tool registry with category-based organization.
    Uses hybrid approach: generic tools + specialized tools for common apps.
    """
    
    def __init__(self):
        self.tools: Dict[str, ToolDefinition] = {}
        self._initialize_builtin_tools()
    
    def _initialize_builtin_tools(self):
        """Initialize built-in tools from modular categories."""
        logger.info("Initializing modular tool system...")
        
        # Register tools from each category module
        register_ui_tools(self, ToolDefinition)
        register_browser_tools(self, ToolDefinition)
        register_app_tools(self, ToolDefinition)
        register_system_tools(self, ToolDefinition)
        register_math_tools(self, ToolDefinition)
        
        logger.info(f"Registered {len(self.tools)} tools across {len(self.get_categories())} categories")
        
    def register_tool(self, tool: ToolDefinition):
        """Register a new tool."""
        self.tools[tool.id] = tool
        logger.info(f"Registered tool: {tool.id} ({tool.name})")
    
    def get_tool(self, tool_id: str) -> Optional[ToolDefinition]:
        """Get tool by ID."""
        return self.tools.get(tool_id)
    
    def get_categories(self) -> List[str]:
        """Get list of all tool categories."""
        categories = set()
        for tool in self.tools.values():
            categories.add(tool.category)
        return sorted(list(categories))
    
    def list_tools(self, category: Optional[str] = None) -> List[ToolDefinition]:
        """List all tools, optionally filtered by category."""
        tools = list(self.tools.values())
        if category:
            tools = [t for t in tools if t.category == category]
        return tools
    
    def get_tools_description(self) -> str:
        """Get formatted description of all tools for AI context."""
        lines = ["=== FRONTEND TOOLS ==="]
        
        # Define category order (generic first, specialized later)
        categories = [
            "ui", "app", "browser", "system", "math", "storage", 
            "network", "timer", "clipboard", "notification"
        ]
        
        for category in categories:
            category_tools = self.list_tools(category)
            if category_tools:
                lines.append(f"\n{category.upper()}:")
                for tool in category_tools:
                    params = ", ".join(f"{k}: {v}" for k, v in tool.parameters.items())
                    params_str = f"({params})" if params else "(no params)"
                    lines.append(f"  - {tool.id}: {tool.description} {params_str}")
        
        return "\n".join(lines)


# ============================================================================
# Exports
# ============================================================================

__all__ = [
    "ToolDefinition",
    "ToolRegistry",
]

