"""
Algebraic Operations
Equation solving, factoring, and symbolic math.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...tools import ToolRegistry, ToolDefinition


def register_algebra(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register algebraic operation tools."""
    
    # =============================================================================
    # EQUATION SOLVING
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="math.solve",
        name="Solve Equation",
        description="Solve algebraic equation for variable",
        parameters={"equation": "string", "variable": "string (default: x)"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.solve_linear",
        name="Solve Linear System",
        description="Solve system of linear equations",
        parameters={"equations": "array<string>", "variables": "array<string>"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.quadratic",
        name="Quadratic Formula",
        description="Solve ax² + bx + c = 0",
        parameters={"a": "number", "b": "number", "c": "number"},
        category="math"
    ))
    
    # =============================================================================
    # POLYNOMIAL OPERATIONS
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="math.expand",
        name="Expand Expression",
        description="Expand algebraic expression",
        parameters={"expression": "string"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.factor",
        name="Factor Expression",
        description="Factor algebraic expression",
        parameters={"expression": "string"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.simplify",
        name="Simplify Expression",
        description="Simplify algebraic expression",
        parameters={"expression": "string"},
        category="math"
    ))
    
    # =============================================================================
    # MATRIX OPERATIONS
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="math.matrix_multiply",
        name="Matrix Multiplication",
        description="Multiply two matrices",
        parameters={"a": "array<array<number>>", "b": "array<array<number>>"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.matrix_determinant",
        name="Matrix Determinant",
        description="Calculate determinant of square matrix",
        parameters={"matrix": "array<array<number>>"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.matrix_inverse",
        name="Matrix Inverse",
        description="Calculate inverse of square matrix",
        parameters={"matrix": "array<array<number>>"},
        category="math"
    ))
    
    registry.register_tool(ToolDefinition(
        id="math.matrix_transpose",
        name="Matrix Transpose",
        description="Transpose matrix (swap rows and columns)",
        parameters={"matrix": "array<array<number>>"},
        category="math"
    ))
