"""
Tool Registry and Definitions
Defines available tools that AI can use to build interactive applications.
"""

import logging
from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


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
    Registry of available tools that the AI can call.
    Tools are small, focused functions accessible via IPC.
    """
    
    def __init__(self):
        self.tools: Dict[str, ToolDefinition] = {}
        self._initialize_builtin_tools()
    
    def _initialize_builtin_tools(self):
        """Initialize built-in tools for common operations."""
        
        # Calculator tools
        self.register_tool(ToolDefinition(
            id="calc.add",
            name="Add",
            description="Add two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.subtract",
            name="Subtract",
            description="Subtract two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.multiply",
            name="Multiply",
            description="Multiply two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.divide",
            name="Divide",
            description="Divide two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        # UI state tools
        self.register_tool(ToolDefinition(
            id="ui.set_state",
            name="Set State",
            description="Update UI state variable",
            parameters={"key": "string", "value": "any"},
            category="ui"
        ))
        
        self.register_tool(ToolDefinition(
            id="ui.get_state",
            name="Get State",
            description="Get UI state variable",
            parameters={"key": "string"},
            category="ui"
        ))
        
        # System tools
        self.register_tool(ToolDefinition(
            id="system.alert",
            name="Alert",
            description="Show alert dialog",
            parameters={"message": "string"},
            category="system"
        ))
        
        self.register_tool(ToolDefinition(
            id="system.log",
            name="Log",
            description="Log message to console",
            parameters={"message": "string", "level": "string"},
            category="system"
        ))
        
        # App management tools
        self.register_tool(ToolDefinition(
            id="app.spawn",
            name="Spawn App",
            description="Create and launch a new app from natural language request",
            parameters={"request": "string"},
            category="app"
        ))
        
        self.register_tool(ToolDefinition(
            id="app.close",
            name="Close App",
            description="Close the current app",
            parameters={},
            category="app"
        ))
        
        self.register_tool(ToolDefinition(
            id="app.list",
            name="List Apps",
            description="List all running apps",
            parameters={},
            category="app"
        ))
        
        # Storage tools
        self.register_tool(ToolDefinition(
            id="storage.set",
            name="Set Storage",
            description="Store data in local storage",
            parameters={"key": "string", "value": "any"},
            category="storage"
        ))
        
        self.register_tool(ToolDefinition(
            id="storage.get",
            name="Get Storage",
            description="Retrieve data from local storage",
            parameters={"key": "string"},
            category="storage"
        ))
        
        self.register_tool(ToolDefinition(
            id="storage.remove",
            name="Remove Storage",
            description="Remove data from local storage",
            parameters={"key": "string"},
            category="storage"
        ))
        
        # Network tools
        self.register_tool(ToolDefinition(
            id="http.get",
            name="HTTP GET",
            description="Fetch data from a URL",
            parameters={"url": "string"},
            category="network"
        ))
        
        self.register_tool(ToolDefinition(
            id="http.post",
            name="HTTP POST",
            description="Send data to a URL",
            parameters={"url": "string", "data": "any"},
            category="network"
        ))
        
        # Timer tools
        self.register_tool(ToolDefinition(
            id="timer.set",
            name="Set Timer",
            description="Execute action after delay",
            parameters={"delay": "number", "action": "string"},
            category="timer"
        ))
        
        self.register_tool(ToolDefinition(
            id="timer.interval",
            name="Set Interval",
            description="Execute action repeatedly",
            parameters={"interval": "number", "action": "string"},
            category="timer"
        ))
        
        self.register_tool(ToolDefinition(
            id="timer.clear",
            name="Clear Timer",
            description="Stop a timer or interval",
            parameters={"timer_id": "string"},
            category="timer"
        ))
        
        # ====================================================================
        # NEW ADVANCED TOOLS FOR DIVERSE APP TYPES
        # ====================================================================
        
        # Canvas/Drawing tools
        self.register_tool(ToolDefinition(
            id="canvas.init",
            name="Initialize Canvas",
            description="Initialize canvas context for drawing",
            parameters={"canvas_id": "string"},
            category="canvas"
        ))
        
        self.register_tool(ToolDefinition(
            id="canvas.draw",
            name="Draw on Canvas",
            description="Draw shapes, lines, or paths on canvas",
            parameters={"canvas_id": "string", "operation": "string", "data": "object"},
            category="canvas"
        ))
        
        self.register_tool(ToolDefinition(
            id="canvas.clear",
            name="Clear Canvas",
            description="Clear entire canvas",
            parameters={"canvas_id": "string"},
            category="canvas"
        ))
        
        self.register_tool(ToolDefinition(
            id="canvas.setTool",
            name="Set Drawing Tool",
            description="Set active drawing tool (pen, eraser, shape, etc.)",
            parameters={"tool": "string"},
            category="canvas"
        ))
        
        self.register_tool(ToolDefinition(
            id="canvas.setColor",
            name="Set Drawing Color",
            description="Set current drawing color",
            parameters={"color": "string"},
            category="canvas"
        ))
        
        self.register_tool(ToolDefinition(
            id="canvas.setBrushSize",
            name="Set Brush Size",
            description="Set brush/pen size for drawing",
            parameters={"size": "number"},
            category="canvas"
        ))
        
        # Browser tools
        self.register_tool(ToolDefinition(
            id="browser.navigate",
            name="Navigate Browser",
            description="Navigate to a URL in browser/iframe",
            parameters={"url": "string"},
            category="browser"
        ))
        
        self.register_tool(ToolDefinition(
            id="browser.back",
            name="Browser Back",
            description="Go back in browser history",
            parameters={},
            category="browser"
        ))
        
        self.register_tool(ToolDefinition(
            id="browser.forward",
            name="Browser Forward",
            description="Go forward in browser history",
            parameters={},
            category="browser"
        ))
        
        self.register_tool(ToolDefinition(
            id="browser.refresh",
            name="Refresh Browser",
            description="Refresh current page",
            parameters={},
            category="browser"
        ))
        
        # Media player tools
        self.register_tool(ToolDefinition(
            id="player.play",
            name="Play Media",
            description="Play video or audio",
            parameters={"media_id": "string"},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.pause",
            name="Pause Media",
            description="Pause video or audio",
            parameters={"media_id": "string"},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.stop",
            name="Stop Media",
            description="Stop video or audio",
            parameters={"media_id": "string"},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.next",
            name="Next Track",
            description="Skip to next track in playlist",
            parameters={},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.previous",
            name="Previous Track",
            description="Go to previous track in playlist",
            parameters={},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.seek",
            name="Seek Media",
            description="Seek to specific time in media",
            parameters={"media_id": "string", "time": "number"},
            category="media"
        ))
        
        self.register_tool(ToolDefinition(
            id="player.setVolume",
            name="Set Volume",
            description="Set media volume (0-100)",
            parameters={"volume": "number"},
            category="media"
        ))
        
        # Game tools
        self.register_tool(ToolDefinition(
            id="game.move",
            name="Game Move",
            description="Make a move in a game",
            parameters={"position": "any", "data": "any"},
            category="game"
        ))
        
        self.register_tool(ToolDefinition(
            id="game.reset",
            name="Reset Game",
            description="Reset game to initial state",
            parameters={},
            category="game"
        ))
        
        self.register_tool(ToolDefinition(
            id="game.score",
            name="Update Score",
            description="Update game score",
            parameters={"score": "number", "player": "string"},
            category="game"
        ))
        
        # Clipboard tools
        self.register_tool(ToolDefinition(
            id="clipboard.copy",
            name="Copy to Clipboard",
            description="Copy text to clipboard",
            parameters={"text": "string"},
            category="clipboard"
        ))
        
        self.register_tool(ToolDefinition(
            id="clipboard.paste",
            name="Paste from Clipboard",
            description="Paste text from clipboard",
            parameters={},
            category="clipboard"
        ))
        
        # Notification tools
        self.register_tool(ToolDefinition(
            id="notification.show",
            name="Show Notification",
            description="Show system notification",
            parameters={"title": "string", "message": "string", "type": "string"},
            category="notification"
        ))
        
        # Form tools
        self.register_tool(ToolDefinition(
            id="form.validate",
            name="Validate Form",
            description="Validate form fields",
            parameters={"form_id": "string"},
            category="form"
        ))
        
        self.register_tool(ToolDefinition(
            id="form.submit",
            name="Submit Form",
            description="Submit form data",
            parameters={"form_id": "string", "data": "object"},
            category="form"
        ))
        
        self.register_tool(ToolDefinition(
            id="form.reset",
            name="Reset Form",
            description="Reset form to initial values",
            parameters={"form_id": "string"},
            category="form"
        ))
        
        # Data manipulation tools
        self.register_tool(ToolDefinition(
            id="data.filter",
            name="Filter Data",
            description="Filter array or list data",
            parameters={"data": "array", "filter": "string"},
            category="data"
        ))
        
        self.register_tool(ToolDefinition(
            id="data.sort",
            name="Sort Data",
            description="Sort array or list data",
            parameters={"data": "array", "field": "string", "order": "string"},
            category="data"
        ))
        
        self.register_tool(ToolDefinition(
            id="data.search",
            name="Search Data",
            description="Search through data",
            parameters={"query": "string", "data": "array"},
            category="data"
        ))
        
        # Todo/List tools
        self.register_tool(ToolDefinition(
            id="list.add",
            name="Add List Item",
            description="Add item to list",
            parameters={"list_id": "string", "item": "any"},
            category="list"
        ))
        
        self.register_tool(ToolDefinition(
            id="list.remove",
            name="Remove List Item",
            description="Remove item from list",
            parameters={"list_id": "string", "item_id": "string"},
            category="list"
        ))
        
        self.register_tool(ToolDefinition(
            id="list.toggle",
            name="Toggle List Item",
            description="Toggle item state (e.g., todo completion)",
            parameters={"list_id": "string", "item_id": "string"},
            category="list"
        ))
        
        self.register_tool(ToolDefinition(
            id="list.clear",
            name="Clear List",
            description="Remove all items from list",
            parameters={"list_id": "string"},
            category="list"
        ))
        
        # Tab/Navigation tools
        self.register_tool(ToolDefinition(
            id="tabs.switch",
            name="Switch Tab",
            description="Switch to different tab",
            parameters={"tab_id": "string"},
            category="navigation"
        ))
        
        self.register_tool(ToolDefinition(
            id="modal.open",
            name="Open Modal",
            description="Open modal dialog",
            parameters={"modal_id": "string"},
            category="navigation"
        ))
        
        self.register_tool(ToolDefinition(
            id="modal.close",
            name="Close Modal",
            description="Close modal dialog",
            parameters={"modal_id": "string"},
            category="navigation"
        ))
        
    def register_tool(self, tool: ToolDefinition):
        """Register a new tool."""
        self.tools[tool.id] = tool
        logger.info(f"Registered tool: {tool.id} ({tool.name})")
    
    def get_tool(self, tool_id: str) -> Optional[ToolDefinition]:
        """Get tool by ID."""
        return self.tools.get(tool_id)
    
    def list_tools(self, category: Optional[str] = None) -> List[ToolDefinition]:
        """List all tools, optionally filtered by category."""
        tools = list(self.tools.values())
        if category:
            tools = [t for t in tools if t.category == category]
        return tools
    
    def get_tools_description(self) -> str:
        """Get formatted description of all tools for AI context."""
        lines = ["Available Tools:"]
        
        # Define category order for better organization
        categories = [
            "compute", "ui", "system", "app", "storage", "network", "timer",
            "canvas", "browser", "media", "game", "clipboard", "notification",
            "form", "data", "list", "navigation"
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

