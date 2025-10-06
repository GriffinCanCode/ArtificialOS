"""
System Tools
System-level operations like alerts, logging, clipboard, notifications.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..tools import ToolRegistry, ToolDefinition


def register_system_tools(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register system-level tools."""

    # =============================================================================
    # SYSTEM OPERATIONS - Browser APIs and system integrations
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="system.alert",
            name="Alert",
            description="Show alert dialog",
            parameters={"message": "string"},
            category="system",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="system.log",
            name="Log",
            description="Log message to console",
            parameters={"message": "string", "level": "string"},
            category="system",
        )
    )

    # =============================================================================
    # STORAGE TOOLS - Client-side localStorage wrapper
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="storage.set",
            name="Set Storage",
            description="Store data in local storage",
            parameters={"key": "string", "value": "any"},
            category="storage",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="storage.get",
            name="Get Storage",
            description="Retrieve data from local storage",
            parameters={"key": "string"},
            category="storage",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="storage.remove",
            name="Remove Storage",
            description="Remove data from local storage",
            parameters={"key": "string"},
            category="storage",
        )
    )

    # =============================================================================
    # NETWORK TOOLS - Client-side HTTP requests
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="http.get",
            name="HTTP GET",
            description="Fetch data from a URL (client-side)",
            parameters={"url": "string"},
            category="network",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="http.post",
            name="HTTP POST",
            description="Send data to a URL (client-side)",
            parameters={"url": "string", "data": "any"},
            category="network",
        )
    )

    # =============================================================================
    # TIMER TOOLS - Delayed execution
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="timer.set",
            name="Set Timer",
            description="Execute action after delay",
            parameters={"delay": "number", "action": "string"},
            category="timer",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="timer.interval",
            name="Set Interval",
            description="Execute action repeatedly",
            parameters={"interval": "number", "action": "string"},
            category="timer",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="timer.clear",
            name="Clear Timer",
            description="Stop a timer or interval",
            parameters={"timer_id": "string"},
            category="timer",
        )
    )

    # =============================================================================
    # CLIPBOARD TOOLS - Copy/paste operations
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="clipboard.copy",
            name="Copy to Clipboard",
            description="Copy text to clipboard",
            parameters={"text": "string"},
            category="clipboard",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="clipboard.paste",
            name="Paste from Clipboard",
            description="Paste text from clipboard",
            parameters={},
            category="clipboard",
        )
    )

    # =============================================================================
    # NOTIFICATION TOOLS - System notifications
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="notification.show",
            name="Show Notification",
            description="Show system notification",
            parameters={"title": "string", "message": "string", "type": "string"},
            category="notification",
        )
    )
