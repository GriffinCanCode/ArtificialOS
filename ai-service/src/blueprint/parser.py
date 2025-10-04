"""Blueprint Parser - YAML to JSON UISpec with validation."""

import yaml
from typing import Dict, Any, List, Union
from datetime import datetime

from core import get_logger, ValidationError

logger = get_logger(__name__)


class BlueprintParser:
    """Parses Blueprint (.bp) files into Package format"""
    
    def __init__(self):
        self.templates = {}
    
    def parse(self, bp_content: str) -> Dict[str, Any]:
        """
        Parse Blueprint YAML string to Package dict
        
        Args:
            bp_content: YAML content string
            
        Returns:
            Package dictionary compatible with types.Package
        """
        try:
            bp = yaml.safe_load(bp_content)
        except yaml.YAMLError as e:
            logger.error("yaml_parse_failed", error=str(e))
            raise ValidationError(f"Invalid YAML: {e}") from e
        
        if not bp or not isinstance(bp, dict):
            logger.error("invalid_format", type=type(bp).__name__)
            raise ValidationError("Invalid Blueprint format: expected YAML object")
        
        if "app" not in bp:
            logger.error("missing_app_section")
            raise ValidationError("Invalid Blueprint: missing 'app' section")
        
        app = bp["app"]
        
        # Validate required fields
        if not app.get("id"):
            logger.error("missing_field", field="app.id")
            raise ValidationError("app.id is required")
        if not app.get("name"):
            logger.error("missing_field", field="app.name")
            raise ValidationError("app.name is required")
        
        # Default timestamp
        now = datetime.utcnow().isoformat() + "Z"
        
        # Store templates for reuse
        self.templates = bp.get("templates", {})
        
        return {
            "id": app["id"],
            "name": app["name"],
            "description": app.get("description", ""),
            "icon": app.get("icon"),
            "category": app.get("category"),
            "version": app.get("version", "1.0.0"),
            "author": app.get("author", "user"),
            "created_at": now,
            "updated_at": now,
            "services": self._expand_services(bp.get("services", [])),
            "permissions": app.get("permissions", ["STANDARD"]),
            "tags": app.get("tags", []),
            "ui_spec": self._expand_ui(bp.get("ui", {})),
            "config": bp.get("config", {})  # App-specific configuration
        }
    
    def _expand_services(self, services: List[Any]) -> List[Union[str, Dict[str, Any]]]:
        """
        Expand service definitions
        
        Supports:
        - Simple strings: ["storage"] → all tools
        - Array syntax: [{storage: [get, set]}] → specific tools
        - Wildcard: [{storage: "*"}] → all tools (explicit)
        - Object config: [{storage: {tools: [get, set], scope: app}}]
        
        Returns mixed list for backward compatibility, but preserves tool info
        """
        result = []
        
        for svc in services:
            if isinstance(svc, str):
                # Simple import - all tools
                result.append(svc)
            elif isinstance(svc, dict):
                # Object with tools/config
                for key, value in svc.items():
                    if value == "*" or value is None:
                        # Wildcard or no value - all tools
                        result.append(key)
                    elif isinstance(value, list):
                        # Explicit tool list: {storage: [get, set]}
                        result.append({
                            "service": key,
                            "tools": value
                        })
                    elif isinstance(value, dict):
                        # Full config: {storage: {tools: [...], scope: app}}
                        tools = value.get("tools", "*")
                        if tools == "*":
                            result.append(key)
                        else:
                            result.append({
                                "service": key,
                                "tools": tools,
                                "config": {k: v for k, v in value.items() if k != "tools"}
                            })
                    else:
                        # Fallback - just service name
                        result.append(key)
        
        return result
    
    def _expand_ui(self, ui: Dict[str, Any]) -> Dict[str, Any]:
        """Expand UI specification with all shortcuts"""
        if not ui:
            return {
                "type": "app",
                "title": "Untitled",
                "layout": "vertical",
                "components": []
            }
        
        return {
            "type": "app",
            "title": ui.get("title", "Untitled"),
            "layout": ui.get("layout", "vertical"),
            "lifecycle_hooks": self._expand_lifecycle(ui.get("lifecycle", {})),
            "components": self._expand_components(ui.get("components", []))
        }
    
    def _expand_lifecycle(self, lifecycle: Dict[str, Any]) -> Dict[str, Any]:
        """
        Expand lifecycle hooks
        
        Supports:
        - Single action: {on_mount: "storage.get"}
        - Multiple actions: {on_mount: ["storage.get", "ui.init"]}
        """
        if not lifecycle:
            return {}
        
        result = {}
        for hook, action in lifecycle.items():
            if isinstance(action, str):
                result[hook] = [action]
            elif isinstance(action, list):
                result[hook] = action
            else:
                result[hook] = []
        
        return result
    
    def _expand_components(self, components: List[Any]) -> List[Dict[str, Any]]:
        """Recursively expand component list"""
        result = []
        
        for comp in components:
            expanded = self._expand_component(comp)
            if expanded:
                result.append(expanded)
        
        return result
    
    def _expand_component(self, comp: Any) -> Dict[str, Any]:
        """
        Expand a single component with all Blueprint shortcuts
        
        Supports:
        - Simple strings: "Hello" -> {type: text, props: {content: "Hello"}}
        - Type#ID syntax: button#save -> {type: button, id: save}
        - Event shortcuts: @click -> on_event.click
        - Layout shortcuts: row/col -> container with layout
        - Templates: $template -> apply template
        - Conditional: $if -> conditional rendering
        - Loop: $for -> repeat component
        """
        # Simple string becomes text component
        if isinstance(comp, str):
            return {
                "type": "text",
                "props": {"content": comp}
            }
        
        # Component object
        if isinstance(comp, dict):
            # Extract type and ID from first key
            for key, props in comp.items():
                if not isinstance(props, dict):
                    props = {}
                
                # Check for template reference
                template_name = props.get("$template")
                if template_name and template_name in self.templates:
                    template = self.templates[template_name]
                    # Merge template with current props (current props override)
                    props = {**template, **{k: v for k, v in props.items() if k != "$template"}}
                
                # Parse "type#id" or just "type"
                parts = key.split("#", 1)
                comp_type = parts[0]
                comp_id = parts[1] if len(parts) > 1 else None
                
                # Handle layout shortcuts
                if comp_type == "row":
                    comp_type = "container"
                    props["layout"] = "horizontal"
                elif comp_type == "col":
                    comp_type = "container"
                    props["layout"] = "vertical"
                
                # Extract special directives and events
                events = {}
                clean_props = {}
                children = None
                conditional = None
                loop_config = None
                
                for k, v in props.items():
                    if k.startswith("@"):
                        # Event handler: @click -> click
                        event_name = k[1:]
                        events[event_name] = v
                    elif k == "children":
                        # Recursively expand children
                        if isinstance(v, list):
                            children = self._expand_components(v)
                    elif k == "$if":
                        # Conditional rendering
                        conditional = v
                    elif k == "$for":
                        # Loop directive
                        loop_config = v
                    elif k.startswith("$"):
                        # Skip other directives (already processed)
                        continue
                    else:
                        clean_props[k] = v
                
                # Build result
                result = {
                    "type": comp_type,
                    "props": clean_props
                }
                
                if comp_id:
                    result["id"] = comp_id
                
                if events:
                    result["on_event"] = events
                
                if children:
                    result["children"] = children
                
                # Add metadata for conditional/loop (frontend will handle)
                if conditional:
                    result["$if"] = conditional
                
                if loop_config:
                    result["$for"] = loop_config
                
                return result
        
        return None


def parse_blueprint(content: str) -> Dict[str, Any]:
    """
    Convenience function to parse Blueprint content
    
    Args:
        content: Blueprint YAML string
        
    Returns:
        Package dictionary
    """
    parser = BlueprintParser()
    return parser.parse(content)

