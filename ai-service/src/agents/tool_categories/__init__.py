"""
Tool Categories
Modular organization of tool definitions by category.
"""

from .ui_tools import register_ui_tools
from .browser_tools import register_browser_tools
from .app_tools import register_app_tools
from .system_tools import register_system_tools

__all__ = [
    "register_ui_tools",
    "register_browser_tools",
    "register_app_tools",
    "register_system_tools",
]

