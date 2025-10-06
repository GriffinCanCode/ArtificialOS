"""
Tool Categories
Modular organization of tool definitions by category.

Note: Browser and simple math operations moved to Go backend.
This package contains Python-specific tools and symbolic math only.
"""

from .ui_tools import register_ui_tools
from .app_tools import register_app_tools
from .system_tools import register_system_tools
from .math import register_math_tools  # Symbolic operations only

__all__ = [
    "register_ui_tools",
    "register_app_tools",
    "register_system_tools",
    "register_math_tools",
]
