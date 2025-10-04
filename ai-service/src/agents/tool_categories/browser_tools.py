"""
Browser Tools
Specialized tools for web browser apps.
These require individualized handling for iframe/webview management.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..tools import ToolRegistry, ToolDefinition


def register_browser_tools(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register browser-specific tools for web navigation."""
    
    # =============================================================================
    # BROWSER NAVIGATION - Specialized for iframe/webview control
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="browser.navigate",
        name="Navigate Browser",
        description="Navigate to a URL in browser iframe - reads from url-input field or params",
        parameters={"url": "string (optional, reads from 'url-input' state if not provided)"},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.back",
        name="Browser Back",
        description="Go back in browser history",
        parameters={},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.forward",
        name="Browser Forward",
        description="Go forward in browser history",
        parameters={},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.refresh",
        name="Refresh Browser",
        description="Refresh current page",
        parameters={},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.home",
        name="Go Home",
        description="Navigate to home page",
        parameters={"home_url": "string (default: 'https://www.google.com')"},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.bookmark.add",
        name="Add Bookmark",
        description="Add current URL to bookmarks list",
        parameters={"title": "string (optional)"},
        category="browser"
    ))
    
    registry.register_tool(ToolDefinition(
        id="browser.bookmark.list",
        name="List Bookmarks",
        description="Get list of saved bookmarks",
        parameters={},
        category="browser"
    ))

