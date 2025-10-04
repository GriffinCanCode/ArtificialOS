"""
Generic UI State Management Tools
These work for ALL apps - calculators, forms, dashboards, etc.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..tools import ToolRegistry, ToolDefinition


def register_ui_tools(registry: "ToolRegistry", ToolDefinition: type) -> None:
    """Register generic UI tools that work across all app types."""
    
    # =============================================================================
    # CORE UI STATE TOOLS - Universal state management
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="ui.set",
        name="Set Value",
        description="Set any state value - works for all inputs, toggles, navigation states",
        parameters={"key": "string", "value": "any"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.get",
        name="Get Value",
        description="Get any state value",
        parameters={"key": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.append",
        name="Append Value",
        description="Append to a string value - works for calculator displays, text fields, search bars",
        parameters={"key": "string (default: 'display')", "value": "string/digit to append"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.clear",
        name="Clear Value",
        description="Clear a value - works for calculator displays, form fields, search inputs",
        parameters={"key": "string (default: 'display')", "default": "default value (default: '0')"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.compute",
        name="Compute Expression",
        description="Evaluate a mathematical expression - for calculator = buttons and formula fields",
        parameters={"key": "string (default: 'display')", "expression": "optional expression to evaluate"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.toggle",
        name="Toggle Boolean",
        description="Toggle a boolean value - for switches, checkboxes, dark mode toggles",
        parameters={"key": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.backspace",
        name="Backspace",
        description="Remove last character from a value - for backspace buttons in calculators, text inputs",
        parameters={"key": "string (default: 'display')"},
        category="ui"
    ))
    
    # =============================================================================
    # LIST/COLLECTION TOOLS - For todos, shopping lists, playlists
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="ui.list.add",
        name="Add List Item",
        description="Add item to a list - for todos, shopping lists, playlists",
        parameters={"list_id": "string", "item": "any"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.list.remove",
        name="Remove List Item",
        description="Remove item from list by index or ID",
        parameters={"list_id": "string", "item_id": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.list.toggle",
        name="Toggle List Item",
        description="Toggle item state (e.g., todo completion, playlist favorite)",
        parameters={"list_id": "string", "item_id": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.list.clear",
        name="Clear List",
        description="Remove all items from list",
        parameters={"list_id": "string"},
        category="ui"
    ))
    
    # =============================================================================
    # FORM TOOLS - Validation, submission
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="ui.form.validate",
        name="Validate Form",
        description="Validate form fields and set error states",
        parameters={"form_id": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.form.submit",
        name="Submit Form",
        description="Submit form data (typically triggers backend service)",
        parameters={"form_id": "string", "data": "object"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.form.reset",
        name="Reset Form",
        description="Reset form to initial values",
        parameters={"form_id": "string"},
        category="ui"
    ))
    
    # =============================================================================
    # NAVIGATION TOOLS - Tabs, modals, panels
    # =============================================================================
    
    registry.register_tool(ToolDefinition(
        id="ui.tabs.switch",
        name="Switch Tab",
        description="Switch to different tab in multi-tab interface",
        parameters={"tab_id": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.modal.open",
        name="Open Modal",
        description="Open modal dialog",
        parameters={"modal_id": "string"},
        category="ui"
    ))
    
    registry.register_tool(ToolDefinition(
        id="ui.modal.close",
        name="Close Modal",
        description="Close modal dialog",
        parameters={"modal_id": "string"},
        category="ui"
    ))

