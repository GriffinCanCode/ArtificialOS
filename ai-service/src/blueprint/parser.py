"""Blueprint Parser - JSON to Blueprint with validation."""

from typing import Dict, Any, List, Union
from datetime import datetime

from core import get_logger, ValidationError
from core.json import extract_json, JSONParseError

logger = get_logger(__name__)


class BlueprintParser:
    """Parses Blueprint (.bp) files into Package format"""
    
    def __init__(self):
        self.templates = {}
        self._id_counter = 0
    
    def parse(self, bp_content: str) -> Dict[str, Any]:
        """
        Parse Blueprint JSON string to Package dict
        
        Args:
            bp_content: JSON content string
            
        Returns:
            Package dictionary compatible with types.Package
        """
        # Reset ID counter for each parse
        self._id_counter = 0
        
        # Use optimized JSON parser with automatic extraction and repair
        try:
            bp = extract_json(bp_content, repair=True)
        except JSONParseError as e:
            logger.error("json_parse_failed", error=str(e))
            raise ValidationError(f"Invalid JSON: {e}") from e
        
        if not bp or not isinstance(bp, dict):
            logger.error("invalid_format", type=type(bp).__name__)
            raise ValidationError("Invalid Blueprint format: expected JSON object")
        
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
        Expand a single component - explicit format only for streaming
        
        Format: {type: "button", id: "save", props: {...}, on_event: {...}, children: [...]}
        
        Special cases:
        - Simple strings: "Hello" -> {type: "text", id: "text-N", props: {content: "Hello"}}
        - Layout shortcuts: type="row" -> type="container" + layout="horizontal"
        """
        # Simple string becomes text component
        if isinstance(comp, str):
            comp_id = f"text-{self._id_counter}"
            self._id_counter += 1
            return {
                "type": "text",
                "id": comp_id,
                "props": {"content": comp}
            }
        
        # Component object
        if isinstance(comp, dict):
            # Check format: explicit (has "type") or compact (first key is "type#id")
            if "type" in comp:
                # Explicit format - optimal for streaming
                return self._expand_explicit_component(comp)
            
            # Compact format - for hand-written .bp files
            for key, props in comp.items():
                if not isinstance(props, dict):
                    props = {}
                
                # Parse "type#id" or just "type"
                parts = key.split("#", 1)
                comp_type = parts[0]
                comp_id = parts[1] if len(parts) > 1 else f"{parts[0]}-{self._id_counter}"
                if comp_id == parts[0]:
                    self._id_counter += 1
                
                # Convert to explicit format by extracting @ events and children
                explicit_props = {}
                events = {}
                children_data = None
                
                for k, v in props.items():
                    if k.startswith("@"):
                        events[k[1:]] = v
                    elif k == "children":
                        children_data = v
                    else:
                        explicit_props[k] = v
                
                # Build explicit component and recursively expand
                explicit_comp = {
                    "type": comp_type,
                    "id": comp_id,
                    "props": explicit_props
                }
                if events:
                    explicit_comp["on_event"] = events
                if children_data:
                    explicit_comp["children"] = children_data
                
                return self._expand_explicit_component(explicit_comp)
        
        return None
    
    def _expand_explicit_component(self, comp: Dict[str, Any]) -> Dict[str, Any]:
        """
        Expand explicit format component {type, id, props, on_event, children}
        This format is optimal for streaming as it's easy to parse incrementally
        """
        comp_type = comp.get("type", "container")
        comp_id = comp.get("id")
        if not comp_id:
            comp_id = f"{comp_type}-{self._id_counter}"
            self._id_counter += 1
        
        props = comp.get("props", {})
        events = comp.get("on_event", {})
        children_data = comp.get("children", [])
        
        # Handle layout shortcuts on type
        if comp_type == "row":
            comp_type = "container"
            props["layout"] = props.get("layout", "horizontal")
        elif comp_type == "col":
            comp_type = "container"
            props["layout"] = props.get("layout", "vertical")
        elif comp_type in ("sidebar", "main", "editor", "header", "footer", "content", "section"):
            props["role"] = comp_type
            comp_type = "container"
            props["layout"] = props.get("layout", "vertical")
        
        # Recursively expand children
        children = None
        if children_data:
            children = self._expand_components(children_data)
        
        # Build result
        result = {
            "type": comp_type,
            "id": comp_id,
            "props": props
        }
        
        if events:
            result["on_event"] = events
        
        if children:
            result["children"] = children
        
        return result


def parse_blueprint(content: str) -> Dict[str, Any]:
    """
    Convenience function to parse Blueprint content
    
    Args:
        content: Blueprint JSON string
        
    Returns:
        Package dictionary
    """
    parser = BlueprintParser()
    return parser.parse(content)

