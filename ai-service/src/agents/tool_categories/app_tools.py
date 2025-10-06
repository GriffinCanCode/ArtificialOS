"""
App Lifecycle Tools
Tools for managing app spawning, closing, and lifecycle.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..tools import ToolRegistry, ToolDefinition


def register_app_tools(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register app lifecycle management tools."""

    # =============================================================================
    # APP LIFECYCLE - System-level app management
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="app.spawn",
            name="Spawn App",
            description="Create and launch a new app from natural language request",
            parameters={"request": "string"},
            category="app",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="app.close",
            name="Close App",
            description="Close the current app",
            parameters={},
            category="app",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="app.list",
            name="List Apps",
            description="List all running apps",
            parameters={},
            category="app",
        )
    )
