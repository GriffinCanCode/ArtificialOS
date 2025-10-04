"""
Math Tools Package - Symbolic Operations Only
Symbolic math operations (algebra, calculus) that require Python's SymPy.
Simple math operations are handled by Go backend for better performance.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...tools import ToolRegistry, ToolDefinition

from .algebra import register_algebra
from .calculus import register_calculus


def register_math_tools(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register symbolic math tool categories (algebra, calculus)."""
    register_algebra(registry, ToolDefinition)
    register_calculus(registry, ToolDefinition)


__all__ = ["register_math_tools"]
