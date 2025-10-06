"""
Calculus Operations
Derivatives, integrals, limits, and series.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...tools import ToolRegistry, ToolDefinition


def register_calculus(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register calculus operation tools."""

    # =============================================================================
    # DERIVATIVES
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="math.derivative",
            name="Derivative",
            description="Calculate symbolic derivative of function",
            parameters={"expression": "string", "variable": "string (default: x)"},
            category="math",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="math.derivative_at",
            name="Derivative at Point",
            description="Calculate derivative value at specific point",
            parameters={"expression": "string", "variable": "string", "point": "number"},
            category="math",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="math.partial",
            name="Partial Derivative",
            description="Calculate partial derivative for multivariate function",
            parameters={"expression": "string", "variable": "string"},
            category="math",
        )
    )

    # =============================================================================
    # INTEGRALS
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="math.integrate",
            name="Indefinite Integral",
            description="Calculate symbolic indefinite integral",
            parameters={"expression": "string", "variable": "string (default: x)"},
            category="math",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="math.integrate_definite",
            name="Definite Integral",
            description="Calculate definite integral from a to b",
            parameters={"expression": "string", "variable": "string", "a": "number", "b": "number"},
            category="math",
        )
    )

    # =============================================================================
    # LIMITS
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="math.limit",
            name="Limit",
            description="Calculate limit as variable approaches value",
            parameters={"expression": "string", "variable": "string", "value": "number|string"},
            category="math",
        )
    )

    # =============================================================================
    # SERIES & SEQUENCES
    # =============================================================================

    registry.register_tool(
        ToolDefinition(
            id="math.series",
            name="Series Sum",
            description="Calculate sum of series from start to end",
            parameters={
                "expression": "string",
                "variable": "string",
                "start": "number",
                "end": "number",
            },
            category="math",
        )
    )

    registry.register_tool(
        ToolDefinition(
            id="math.taylor",
            name="Taylor Series",
            description="Calculate Taylor series expansion around point",
            parameters={
                "expression": "string",
                "variable": "string",
                "point": "number",
                "order": "number",
            },
            category="math",
        )
    )
