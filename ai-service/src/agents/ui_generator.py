"""
UI Generation Agent
Generates structured UI specifications from natural language.
Uses function calling / tool binding to create interactive apps.
"""

import json
import logging
from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field
from langchain_core.language_models import BaseLLM
from .prompt import get_ui_generation_prompt, get_simple_system_prompt
from .tools import ToolDefinition, ToolRegistry

logger = logging.getLogger(__name__)

# Import UI cache for performance optimization
try:
    from .ui_cache import UICache
    _ui_cache_available = True
except ImportError:
    _ui_cache_available = False
    logger.warning("UI cache not available")


# ============================================================================
# UI Component Schema
# ============================================================================

class UIComponent(BaseModel):
    """Base UI component specification."""
    type: str = Field(..., description="Component type (button, input, text, etc.)")
    id: str = Field(..., description="Unique component identifier")
    props: Dict[str, Any] = Field(default_factory=dict, description="Component properties")
    children: List["UIComponent"] = Field(default_factory=list, description="Child components")
    on_event: Optional[Dict[str, str]] = Field(default=None, description="Event handlers (event -> tool_id)")


class UISpec(BaseModel):
    """Complete UI specification."""
    type: str = Field(default="app", description="Root type")
    title: str = Field(..., description="App title")
    layout: str = Field(default="vertical", description="Layout direction")
    components: List[UIComponent] = Field(default_factory=list, description="Root components")
    style: Dict[str, Any] = Field(default_factory=dict, description="Global styles")
    services: List[str] = Field(default_factory=list, description="Required service IDs")
    service_bindings: Dict[str, str] = Field(default_factory=dict, description="Component to service tool mapping")
    lifecycle_hooks: Dict[str, List[str]] = Field(default_factory=dict, description="Lifecycle event hooks")
    

# Make UIComponent work with forward references
UIComponent.model_rebuild()


# ============================================================================
# Component Templates
# ============================================================================

class ComponentTemplates:
    """Pre-built component templates for common UI patterns."""
    
    @staticmethod
    def button(id: str, text: str, on_click: Optional[str] = None, variant: str = "default", size: str = "medium") -> UIComponent:
        """Create a button component with CVA variant support."""
        return UIComponent(
            type="button",
            id=id,
            props={"text": text, "variant": variant, "size": size},
            on_event={"click": on_click} if on_click else None
        )
    
    @staticmethod
    def input(id: str, placeholder: str = "", value: str = "", variant: str = "default", size: str = "medium", readonly: bool = False) -> UIComponent:
        """Create an input component with CVA variant support."""
        return UIComponent(
            type="input",
            id=id,
            props={
                "placeholder": placeholder,
                "value": value,
                "type": "text",
                "variant": variant,
                "size": size,
                "readonly": readonly
            }
        )
    
    @staticmethod
    def text(id: str, content: str, variant: str = "body", weight: Optional[str] = None, color: Optional[str] = None, align: Optional[str] = None) -> UIComponent:
        """Create a text component with CVA variant support."""
        props = {"content": content, "variant": variant}
        if weight:
            props["weight"] = weight
        if color:
            props["color"] = color
        if align:
            props["align"] = align
        return UIComponent(
            type="text",
            id=id,
            props=props
        )
    
    @staticmethod
    def container(
        id: str,
        children: List[UIComponent],
        layout: str = "vertical",
        gap: Optional[int] = None,
        spacing: Optional[str] = None,
        padding: Optional[str] = None,
        align: Optional[str] = None,
        justify: Optional[str] = None
    ) -> UIComponent:
        """Create a container component with CVA variant support."""
        props = {"layout": layout}
        if gap is not None:
            props["gap"] = gap
        if spacing:
            props["spacing"] = spacing
        if padding:
            props["padding"] = padding
        if align:
            props["align"] = align
        if justify:
            props["justify"] = justify
        return UIComponent(
            type="container",
            id=id,
            props=props,
            children=children
        )
    
    @staticmethod
    def grid(
        id: str,
        children: List[UIComponent],
        columns: int = 3,
        gap: Optional[int] = None,
        spacing: Optional[str] = None,
        responsive: bool = False
    ) -> UIComponent:
        """Create a grid component with CVA variant support."""
        props = {"columns": columns, "responsive": responsive}
        if gap is not None:
            props["gap"] = gap
        if spacing:
            props["spacing"] = spacing
        return UIComponent(
            type="grid",
            id=id,
            props=props,
            children=children
        )


# ============================================================================
# UI Generator Agent
# ============================================================================

class UIGeneratorAgent:
    """
    Agent that generates UI specifications from natural language.
    
    Architecture:
    1. User: "create a calculator"
    2. Agent: Analyzes intent, selects components + tools
    3. Agent: Generates UISpec with tool bindings
    4. Frontend: DynamicRenderer parses UISpec and renders React components
    5. User: Clicks button -> calls tool via IPC -> updates UI
    """
    
    # Note: SYSTEM_PROMPT is now managed in prompt.py for better maintainability
    # Use get_system_prompt() or get_ui_generation_prompt() instead
    
    def __init__(
        self,
        tool_registry: Optional[ToolRegistry] = None,
        llm: Optional[BaseLLM] = None,
        service_registry: Optional[Any] = None,
        context_builder: Optional[Any] = None,
        enable_cache: bool = True,
        backend_services: Optional[List[Any]] = None
    ):
        """
        Initialize the UI generator agent.
        
        Args:
            tool_registry: Registry of available tools
            llm: Optional LLM for generating UIs (if None, uses rule-based generation)
            service_registry: Service registry for BaaS
            context_builder: Context builder for intelligent prompts
            enable_cache: Enable UI spec caching for performance (default: True)
            backend_services: List of backend service definitions from discovery
        """
        self.tool_registry = tool_registry or ToolRegistry()
        self.templates = ComponentTemplates()
        self.llm = llm
        self.use_llm = llm is not None
        self.service_registry = service_registry
        self.context_builder = context_builder
        self.backend_services = backend_services or []
        
        # Initialize cache if available and enabled
        self.cache = None
        if enable_cache and _ui_cache_available:
            self.cache = UICache(max_size=100, ttl_seconds=3600)
            logger.info("UIGeneratorAgent: UI caching enabled")
        
        if self.use_llm:
            logger.info("UIGeneratorAgent: LLM-based generation enabled")
        else:
            logger.info("UIGeneratorAgent: Using rule-based generation")
        
        if self.backend_services:
            logger.info(f"UIGeneratorAgent: Loaded {len(self.backend_services)} backend services")
    
    def get_backend_tools_description(self) -> str:
        """
        Get description of backend services for AI context.
        
        Returns:
            Formatted backend service tools
        """
        if not self.backend_services:
            return ""
        
        lines = ["\nBACKEND SERVICES (call via frontend service tool executor):"]
        
        for service in self.backend_services:
            lines.append(f"\n{service.category.upper()} - {service.name}:")
            for tool in service.tools:
                params_str = ", ".join(
                    f"{p.name}: {p.type}" + (" (required)" if p.required else "")
                    for p in tool.parameters
                )
                params_display = f"({params_str})" if params_str else "()"
                lines.append(f"  - {tool.id}: {tool.description} {params_display}")
        
        return "\n".join(lines)
    
    def get_system_prompt(self, context_str: str = "") -> str:
        """Get comprehensive system prompt with current tool descriptions and service context."""
        # Combine frontend tools and backend services
        tools_desc = self.tool_registry.get_tools_description()
        backend_desc = self.get_backend_tools_description()
        combined_tools = tools_desc + backend_desc
        
        return get_ui_generation_prompt(
            tools_description=combined_tools,
            context=context_str
        )
    
    def generate_ui(self, request: str, stream_callback=None) -> UISpec:
        """
        Non-streaming version - collects all tokens then returns UISpec.
        For streaming, use generate_ui_stream() instead.
        """
        # If callback provided, collect tokens
        if stream_callback:
            for item in self.generate_ui_stream(request):
                if isinstance(item, str):
                    stream_callback(item)
                else:
                    return item
        else:
            # No streaming, just return the spec
            for item in self.generate_ui_stream(request):
                if isinstance(item, UISpec):
                    return item
        
    def generate_ui_stream(self, request: str):
        """
        Generator that yields tokens (str) during generation, then yields final UISpec.
        
        Args:
            request: User's natural language request (e.g., "create a calculator")
            
        Yields:
            str: JSON tokens during generation
            dict: {"reset": True} to signal buffer reset (when LLM fails)
            UISpec: Final UI specification
        """
        # Check cache first for performance
        if self.cache:
            cached_spec = self.cache.get(request)
            if cached_spec:
                logger.info(f"Using cached UI spec for: {request[:50]}...")
                # Stream the cached JSON for live viewing (simulates generation)
                json_str = json.dumps(cached_spec.model_dump(), indent=2)
                chunk_size = 100  # Larger chunks for cached content (faster)
                for i in range(0, len(json_str), chunk_size):
                    yield json_str[i:i+chunk_size]
                yield cached_spec
                return
        
        # Use LLM-based generation if available
        if self.use_llm:
            try:
                logger.info(f"Generating UI with LLM for: {request}")
                # Stream tokens from LLM
                for item in self._generate_with_llm_stream(request):
                    yield item  # Yield tokens (str) or UISpec
                return
            except Exception as e:
                logger.warning(f"LLM generation failed: {e}. Falling back to rules.")
                # Signal backend to reset buffer before sending rule-based generation
                yield {"reset": True}
                # Fall through to rule-based generation
        
        # Rule-based generation (fallback) - stream JSON tokens for live viewing
        logger.info(f"Using rule-based generation for: {request}")
        request_lower = request.lower()
        
        if "calculator" in request_lower:
            ui_spec = self._generate_calculator()
        elif "todo" in request_lower or "task" in request_lower:
            ui_spec = self._generate_todo_app()
        elif "counter" in request_lower:
            ui_spec = self._generate_counter()
        else:
            ui_spec = self._generate_placeholder(request)
        
        # Stream JSON tokens character-by-character for live viewing
        # This preserves the real-time "watching app being built" experience
        json_str = json.dumps(ui_spec.model_dump(), indent=2)
        chunk_size = 50  # Characters per chunk for smooth streaming
        for i in range(0, len(json_str), chunk_size):
            yield json_str[i:i+chunk_size]
        
        # Cache the generated UI spec for future requests
        if self.cache:
            self.cache.set(request, ui_spec)
        
        yield ui_spec
    
    def _generate_with_llm_stream(self, request: str):
        """
        Generate UI using Gemini with structured JSON output.
        
        Args:
            request: User's natural language request
            
        Yields:
            str: JSON tokens during generation
            UISpec: Final UI specification
        """
        # Build context with services if available
        context_str = ""
        if self.context_builder and self.service_registry:
            try:
                context = self.context_builder.build(request)
                context_str = self.context_builder.format_prompt(context)
                logger.info(f"Enhanced context with {len(context.services)} services")
            except Exception as e:
                logger.warning(f"Context building failed: {e}")
        
        # Create comprehensive prompt with tool descriptions and context
        tools_description = self.tool_registry.get_tools_description()
        
        # Add backend services to the tools description
        if self.backend_services:
            backend_desc = self._format_backend_services()
            tools_description += "\n\n" + backend_desc
            logger.info(f"Added {len(self.backend_services)} backend services to AI context")
        
        # Build the full prompt using our comprehensive prompt system
        full_prompt = get_ui_generation_prompt(
            tools_description=tools_description,
            context=context_str
        )
        
        # Add user request at the end
        full_prompt += f"\n\n=== USER REQUEST ===\n{request}\n\nGenerate the complete JSON specification now:"
        
        logger.info(f"Sending prompt to Gemini for: {request}")
        logger.debug(f"Full prompt length: {len(full_prompt)} characters")
        
        # Use the LLM instance (which should be GeminiModel)
        if not self.llm:
            raise ValueError("No LLM instance available for generation")
        
        # Check if this is a Gemini model with JSON streaming support
        has_json_stream = hasattr(self.llm, 'stream_json')
        
        # Stream tokens in real-time
        content = ""
        chunk_count = 0
        
        try:
            # Use JSON streaming if available
            if has_json_stream:
                logger.debug("Using Gemini JSON streaming")
                for token in self.llm.stream_json(full_prompt):
                    chunk_count += 1
                    content += token
                    if token:
                        yield token
            else:
                # Fallback to regular streaming
                logger.debug("Using regular streaming")
                for chunk in self.llm.stream(full_prompt):
                    chunk_count += 1
                    # Extract token - handle different response formats
                    if hasattr(chunk, 'content'):
                        token = chunk.content
                    else:
                        token = str(chunk) if chunk else ""
                    
                    content += token
                    if token:
                        yield token
        
            logger.info(f"Gemini response: {chunk_count} chunks, {len(content)} characters")
            if content:
                logger.debug(f"Response preview: {content[:200]}...")
            else:
                logger.warning("Gemini returned empty content after streaming")
            
            # Parse JSON from response
            if has_json_stream:
                # Gemini with JSON mode returns valid JSON directly
                try:
                    ui_spec_json = json.loads(content)
                except json.JSONDecodeError as e:
                    logger.error(f"JSON parse error: {e}")
                    raise ValueError(f"Invalid JSON from Gemini: {e}")
            else:
                # Extract JSON from text response
                from .gpt_oss_output_parser import extract_json_from_response
                json_str = extract_json_from_response(content)
                
                if not json_str:
                    json_str = self._extract_json(content)
                    ui_spec_json = json_str
                else:
                    try:
                        ui_spec_json = json.loads(json_str)
                    except json.JSONDecodeError:
                        ui_spec_json = self._extract_json(content)
            
            # Validate and convert to UISpec
            ui_spec = UISpec.model_validate(ui_spec_json)
            logger.info(f"Successfully generated UI: {ui_spec.title}")
            
            # Cache the generated UI spec for future requests
            if self.cache:
                self.cache.set(request, ui_spec)
            
            yield ui_spec  # Yield the final UISpec
            
        except Exception as e:
            logger.error(f"Failed to generate UI with Gemini: {e}")
            logger.error(f"Response content: {content[:500] if content else 'empty'}")
            raise ValueError(f"Invalid UISpec generated: {e}")
    
    def _extract_json(self, text: str) -> Dict:
        """
        Extract JSON from LLM response (may have markdown formatting or extra text).
        
        Args:
            text: Raw LLM response text
            
        Returns:
            Parsed JSON dictionary
        """
        text = text.strip()
        
        # Remove markdown code blocks if present
        if "```" in text:
            # Extract content between code blocks
            if "```json" in text:
                start = text.find("```json") + 7
            elif "```" in text:
                start = text.find("```") + 3
            else:
                start = 0
            
            # Find the closing ```
            end = text.find("```", start)
            if end != -1:
                text = text[start:end].strip()
        
        # Find JSON object boundaries
        start = text.find("{")
        end = text.rfind("}")
        
        if start == -1 or end == -1:
            logger.error(f"No JSON braces found in: {text[:200]}...")
            raise ValueError("No JSON object found in response")
        
        json_str = text[start:end+1]
        
        # Try to parse JSON
        try:
            parsed = json.loads(json_str)
            logger.info(f"Successfully parsed JSON with {len(json_str)} characters")
            return parsed
        except json.JSONDecodeError as e:
            logger.error(f"JSON parse error at position {e.pos}: {e.msg}")
            logger.error(f"Error context: ...{json_str[max(0, e.pos-50):e.pos+50]}...")
            
            # Try to fix common issues
            # 1. Trailing commas
            fixed_str = json_str.replace(",]", "]").replace(",}", "}")
            try:
                parsed = json.loads(fixed_str)
                logger.info("Fixed JSON by removing trailing commas")
                return parsed
            except:
                pass
            
            logger.error(f"Full JSON attempt ({len(json_str)} chars): {json_str[:1000]}...")
            raise ValueError(f"Invalid JSON in response: {e.msg} at position {e.pos}")
    
    def _generate_calculator(self) -> UISpec:
        """Generate a calculator UI using generic ui.* tools."""
        # Display
        display = self.templates.input("display", value="0")
        display.props["readonly"] = True
        display.props["style"] = {"fontSize": "24px", "textAlign": "right"}
        
        # Layout - Using generic ui.append for ALL buttons (digits and operators)
        button_grid_children = [
            # Row 1
            self.templates.button("btn-7", "7", on_click="ui.append"),
            self.templates.button("btn-8", "8", on_click="ui.append"),
            self.templates.button("btn-9", "9", on_click="ui.append"),
            self.templates.button("btn-divide", "÷", on_click="ui.append"),
            
            # Row 2
            self.templates.button("btn-4", "4", on_click="ui.append"),
            self.templates.button("btn-5", "5", on_click="ui.append"),
            self.templates.button("btn-6", "6", on_click="ui.append"),
            self.templates.button("btn-multiply", "×", on_click="ui.append"),
            
            # Row 3
            self.templates.button("btn-1", "1", on_click="ui.append"),
            self.templates.button("btn-2", "2", on_click="ui.append"),
            self.templates.button("btn-3", "3", on_click="ui.append"),
            self.templates.button("btn-subtract", "−", on_click="ui.append"),
            
            # Row 4
            self.templates.button("btn-0", "0", on_click="ui.append"),
            self.templates.button("btn-clear", "C", on_click="ui.clear"),
            self.templates.button("btn-equals", "=", on_click="ui.compute"),
            self.templates.button("btn-add", "+", on_click="ui.append"),
        ]
        
        button_grid = self.templates.grid("button-grid", button_grid_children, columns=4, gap=8)
        
        return UISpec(
            title="Calculator",
            layout="vertical",
            components=[display, button_grid],
            style={"padding": "20px", "maxWidth": "300px"}
        )
    
    def _generate_counter(self) -> UISpec:
        """Generate a simple counter UI using generic ui.* tools."""
        count_display = self.templates.input("count", value="0")
        count_display.props["readonly"] = True
        count_display.props["style"] = {"fontSize": "48px", "fontWeight": "bold", "textAlign": "center"}
        
        # Use ui.append with +1 and -1 for increment/decrement
        increment = self.templates.button("btn-inc", "+1", on_click="ui.append")
        decrement = self.templates.button("btn-dec", "-1", on_click="ui.append")
        
        # Compute button to evaluate the expression
        compute = self.templates.button("btn-compute", "=", on_click="ui.compute")
        reset = self.templates.button("btn-reset", "Clear", on_click="ui.clear")
        
        buttons = self.templates.container("buttons", [increment, decrement, compute, reset], layout="horizontal", gap=12)
        
        return UISpec(
            title="Counter",
            layout="vertical",
            components=[count_display, buttons],
            style={"padding": "40px", "textAlign": "center"}
        )
    
    def _generate_todo_app(self) -> UISpec:
        """Generate a todo app UI."""
        # Header
        header = self.templates.text("header", "📝 Todo List", variant="h2")
        
        # Input + Add button
        task_input = self.templates.input("task-input", placeholder="Enter a new task...")
        add_button = self.templates.button("btn-add", "Add Task", on_click="ui.add_todo")
        input_row = self.templates.container("input-row", [task_input, add_button], layout="horizontal", gap=8)
        
        # Todo list container
        todo_list = self.templates.container("todo-list", [], layout="vertical", gap=8)
        
        return UISpec(
            title="Todo App",
            layout="vertical",
            components=[header, input_row, todo_list],
            style={"padding": "20px", "maxWidth": "500px"}
        )
    
    def _generate_placeholder(self, request: str) -> UISpec:
        """Generate a placeholder UI for unknown requests."""
        header = self.templates.text("header", f"🎨 Generated App", variant="h2")
        message = self.templates.text(
            "message",
            f"Request: {request}",
            variant="body"
        )
        info = self.templates.text(
            "info",
            "UI generation for this request is not yet implemented.",
            variant="caption"
        )
        
        return UISpec(
            title="Generated App",
            layout="vertical",
            components=[header, message, info],
            style={"padding": "40px"}
        )
    
    def spec_to_json(self, spec: UISpec) -> str:
        """Convert UISpec to JSON string."""
        return spec.model_dump_json(indent=2)
    
    def json_to_spec(self, json_str: str) -> UISpec:
        """Parse JSON string to UISpec."""
        return UISpec.model_validate_json(json_str)
    
    def _format_backend_services(self) -> str:
        """
        Format backend services for AI context.
        
        Returns:
            Formatted string describing backend services and their tools
        """
        if not self.backend_services:
            return ""
        
        lines = ["=== BACKEND SERVICES ==="]
        lines.append("\nThese services run on the Go backend and require service calls:")
        
        for service in self.backend_services:
            lines.append(f"\n{service.category.upper()} - {service.name}:")
            lines.append(f"  Description: {service.description}")
            lines.append(f"  Capabilities: {', '.join(service.capabilities)}")
            lines.append(f"  Tools:")
            
            for tool in service.tools:
                params_list = []
                for param in tool.parameters:
                    req_str = "(required)" if param.required else "(optional)"
                    params_list.append(f"{param.name}: {param.type} {req_str}")
                
                params_str = f"({', '.join(params_list)})" if params_list else "(no params)"
                lines.append(f"    - {tool.id}: {tool.description} {params_str}")
        
        return "\n".join(lines)


# ============================================================================
# Exports
# ============================================================================

__all__ = [
    "UIGeneratorAgent",
    "UISpec",
    "UIComponent",
    "ComponentTemplates",
]

# Note: ToolRegistry and ToolDefinition are now exported from tools.py

