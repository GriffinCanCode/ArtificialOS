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
from langchain_core.tools import tool
from langchain_core.messages import HumanMessage, SystemMessage

logger = logging.getLogger(__name__)


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
# Tool Registry Schema
# ============================================================================

class ToolDefinition(BaseModel):
    """Definition of a callable tool."""
    id: str = Field(..., description="Unique tool identifier")
    name: str = Field(..., description="Human-readable name")
    description: str = Field(..., description="What the tool does")
    parameters: Dict[str, Any] = Field(default_factory=dict, description="Parameter schema")
    category: str = Field(default="general", description="Tool category (compute, ui, system, etc.)")


class ToolRegistry:
    """
    Registry of available tools that the AI can call.
    Tools are small, focused functions accessible via IPC.
    """
    
    def __init__(self):
        self.tools: Dict[str, ToolDefinition] = {}
        self._initialize_builtin_tools()
    
    def _initialize_builtin_tools(self):
        """Initialize built-in tools for common operations."""
        
        # Calculator tools
        self.register_tool(ToolDefinition(
            id="calc.add",
            name="Add",
            description="Add two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.subtract",
            name="Subtract",
            description="Subtract two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.multiply",
            name="Multiply",
            description="Multiply two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        self.register_tool(ToolDefinition(
            id="calc.divide",
            name="Divide",
            description="Divide two numbers",
            parameters={"a": "number", "b": "number"},
            category="compute"
        ))
        
        # UI state tools
        self.register_tool(ToolDefinition(
            id="ui.set_state",
            name="Set State",
            description="Update UI state variable",
            parameters={"key": "string", "value": "any"},
            category="ui"
        ))
        
        self.register_tool(ToolDefinition(
            id="ui.get_state",
            name="Get State",
            description="Get UI state variable",
            parameters={"key": "string"},
            category="ui"
        ))
        
        # System tools
        self.register_tool(ToolDefinition(
            id="system.alert",
            name="Alert",
            description="Show alert dialog",
            parameters={"message": "string"},
            category="system"
        ))
        
        self.register_tool(ToolDefinition(
            id="system.log",
            name="Log",
            description="Log message to console",
            parameters={"message": "string", "level": "string"},
            category="system"
        ))
        
        # App management tools
        self.register_tool(ToolDefinition(
            id="app.spawn",
            name="Spawn App",
            description="Create and launch a new app from natural language request",
            parameters={"request": "string"},
            category="app"
        ))
        
        self.register_tool(ToolDefinition(
            id="app.close",
            name="Close App",
            description="Close the current app",
            parameters={},
            category="app"
        ))
        
        self.register_tool(ToolDefinition(
            id="app.list",
            name="List Apps",
            description="List all running apps",
            parameters={},
            category="app"
        ))
        
        # Storage tools
        self.register_tool(ToolDefinition(
            id="storage.set",
            name="Set Storage",
            description="Store data in local storage",
            parameters={"key": "string", "value": "any"},
            category="storage"
        ))
        
        self.register_tool(ToolDefinition(
            id="storage.get",
            name="Get Storage",
            description="Retrieve data from local storage",
            parameters={"key": "string"},
            category="storage"
        ))
        
        self.register_tool(ToolDefinition(
            id="storage.remove",
            name="Remove Storage",
            description="Remove data from local storage",
            parameters={"key": "string"},
            category="storage"
        ))
        
        # Network tools
        self.register_tool(ToolDefinition(
            id="http.get",
            name="HTTP GET",
            description="Fetch data from a URL",
            parameters={"url": "string"},
            category="network"
        ))
        
        self.register_tool(ToolDefinition(
            id="http.post",
            name="HTTP POST",
            description="Send data to a URL",
            parameters={"url": "string", "data": "any"},
            category="network"
        ))
        
        # Timer tools
        self.register_tool(ToolDefinition(
            id="timer.set",
            name="Set Timer",
            description="Execute action after delay",
            parameters={"delay": "number", "action": "string"},
            category="timer"
        ))
        
        self.register_tool(ToolDefinition(
            id="timer.interval",
            name="Set Interval",
            description="Execute action repeatedly",
            parameters={"interval": "number", "action": "string"},
            category="timer"
        ))
        
        self.register_tool(ToolDefinition(
            id="timer.clear",
            name="Clear Timer",
            description="Stop a timer or interval",
            parameters={"timer_id": "string"},
            category="timer"
        ))
        
    def register_tool(self, tool: ToolDefinition):
        """Register a new tool."""
        self.tools[tool.id] = tool
        logger.info(f"Registered tool: {tool.id} ({tool.name})")
    
    def get_tool(self, tool_id: str) -> Optional[ToolDefinition]:
        """Get tool by ID."""
        return self.tools.get(tool_id)
    
    def list_tools(self, category: Optional[str] = None) -> List[ToolDefinition]:
        """List all tools, optionally filtered by category."""
        tools = list(self.tools.values())
        if category:
            tools = [t for t in tools if t.category == category]
        return tools
    
    def get_tools_description(self) -> str:
        """Get formatted description of all tools for AI context."""
        lines = ["Available Tools:"]
        for category in ["compute", "ui", "system", "app", "storage", "network", "timer"]:
            category_tools = self.list_tools(category)
            if category_tools:
                lines.append(f"\n{category.upper()}:")
                for tool in category_tools:
                    params = ", ".join(f"{k}: {v}" for k, v in tool.parameters.items())
                    params_str = f"({params})" if params else "(no params)"
                    lines.append(f"  - {tool.id}: {tool.description} {params_str}")
        return "\n".join(lines)


# ============================================================================
# Function Calling Tools for LLM UI Generation
# ============================================================================

@tool
def create_button(id: str, text: str, on_click: Optional[str] = None, variant: str = "default", size: str = "medium") -> Dict:
    """
    Create a button component.
    
    Args:
        id: Unique identifier for the button
        text: Button label text
        on_click: Tool ID to call when clicked (e.g., "calc.add", "ui.set_state")
        variant: Button style variant (default, primary, danger)
        size: Button size (small, medium, large)
    
    Returns:
        Button component specification
    """
    return {
        "type": "button",
        "id": id,
        "props": {"text": text, "variant": variant, "size": size},
        "on_event": {"click": on_click} if on_click else None
    }


@tool
def create_input(id: str, placeholder: str = "", value: str = "", input_type: str = "text", readonly: bool = False) -> Dict:
    """
    Create an input field component.
    
    Args:
        id: Unique identifier for the input
        placeholder: Placeholder text
        value: Initial value
        input_type: Input type (text, number, password, email)
        readonly: Whether input is readonly
    
    Returns:
        Input component specification
    """
    return {
        "type": "input",
        "id": id,
        "props": {
            "placeholder": placeholder,
            "value": value,
            "type": input_type,
            "readonly": readonly
        }
    }


@tool
def create_text(id: str, content: str, variant: str = "body", style: Optional[Dict] = None) -> Dict:
    """
    Create a text/label component.
    
    Args:
        id: Unique identifier
        content: Text content to display
        variant: Text style (h1, h2, h3, body, caption)
        style: Optional inline styles (fontSize, fontWeight, color, etc.)
    
    Returns:
        Text component specification
    """
    component = {
        "type": "text",
        "id": id,
        "props": {"content": content, "variant": variant}
    }
    if style:
        component["props"]["style"] = style
    return component


@tool
def create_container(id: str, layout: str = "vertical", gap: int = 8) -> Dict:
    """
    Create a container for organizing child components.
    
    Args:
        id: Unique identifier
        layout: Layout direction (vertical, horizontal)
        gap: Spacing between children in pixels
    
    Returns:
        Container component specification (add children separately)
    """
    return {
        "type": "container",
        "id": id,
        "props": {"layout": layout, "gap": gap},
        "children": []
    }


@tool
def create_grid(id: str, columns: int = 3, gap: int = 8) -> Dict:
    """
    Create a grid layout for arranging components in a grid.
    
    Args:
        id: Unique identifier
        columns: Number of columns
        gap: Spacing between items in pixels
    
    Returns:
        Grid component specification (add children separately)
    """
    return {
        "type": "grid",
        "id": id,
        "props": {"columns": columns, "gap": gap},
        "children": []
    }


# Tool list for LLM binding
UI_COMPONENT_TOOLS = [
    create_button,
    create_input,
    create_text,
    create_container,
    create_grid,
]


# ============================================================================
# Component Templates
# ============================================================================

class ComponentTemplates:
    """Pre-built component templates for common UI patterns."""
    
    @staticmethod
    def button(id: str, text: str, on_click: Optional[str] = None) -> UIComponent:
        """Create a button component."""
        return UIComponent(
            type="button",
            id=id,
            props={"text": text},
            on_event={"click": on_click} if on_click else None
        )
    
    @staticmethod
    def input(id: str, placeholder: str = "", value: str = "") -> UIComponent:
        """Create an input component."""
        return UIComponent(
            type="input",
            id=id,
            props={
                "placeholder": placeholder,
                "value": value,
                "type": "text"
            }
        )
    
    @staticmethod
    def text(id: str, content: str, variant: str = "body") -> UIComponent:
        """Create a text component."""
        return UIComponent(
            type="text",
            id=id,
            props={"content": content, "variant": variant}
        )
    
    @staticmethod
    def container(
        id: str,
        children: List[UIComponent],
        layout: str = "vertical",
        gap: int = 8
    ) -> UIComponent:
        """Create a container component."""
        return UIComponent(
            type="container",
            id=id,
            props={"layout": layout, "gap": gap},
            children=children
        )
    
    @staticmethod
    def grid(
        id: str,
        children: List[UIComponent],
        columns: int = 3,
        gap: int = 8
    ) -> UIComponent:
        """Create a grid component."""
        return UIComponent(
            type="grid",
            id=id,
            props={"columns": columns, "gap": gap},
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
    
    SYSTEM_PROMPT = """You are a UI generation expert. Your ONLY job is to output valid JSON for UI specifications.

CRITICAL RULES:
1. Output ONLY valid JSON - no explanations, no markdown, no extra text
2. EVERY component MUST have: "id", "type", "props", "children", "on_event"
3. Keep JSON concise - avoid deeply nested structures
4. For calculator: maximum 16 buttons (4x4 grid)

{tools_description}

EXACT COMPONENT SCHEMA:

Button:
{{
  "type": "button",
  "id": "unique-id",
  "props": {{"text": "Label", "variant": "default", "size": "medium"}},
  "children": [],
  "on_event": {{"click": "tool.name"}}
}}

Input:
{{
  "type": "input",
  "id": "unique-id",
  "props": {{"placeholder": "...", "value": "...", "type": "text", "readonly": false}},
  "children": [],
  "on_event": null
}}

Text:
{{
  "type": "text",
  "id": "unique-id",
  "props": {{"content": "text here", "variant": "body"}},
  "children": [],
  "on_event": null
}}

Container:
{{
  "type": "container",
  "id": "unique-id",
  "props": {{"layout": "vertical", "gap": 8}},
  "children": [/* nested components here */],
  "on_event": null
}}

Grid:
{{
  "type": "grid",
  "id": "unique-id",
  "props": {{"columns": 3, "gap": 8}},
  "children": [/* grid items here */],
  "on_event": null
}}

COMPLETE EXAMPLE - Calculator:
{{
  "type": "app",
  "title": "Calculator",
  "layout": "vertical",
  "components": [
    {{
      "type": "input",
      "id": "display",
      "props": {{"value": "0", "readonly": true}},
      "children": [],
      "on_event": null
    }},
    {{
      "type": "grid",
      "id": "button-grid",
      "props": {{"columns": 4, "gap": 8}},
      "children": [
        {{"type": "button", "id": "btn-7", "props": {{"text": "7"}}, "children": [], "on_event": {{"click": "calc.add"}}}},
        {{"type": "button", "id": "btn-8", "props": {{"text": "8"}}, "children": [], "on_event": {{"click": "calc.add"}}}},
        {{"type": "button", "id": "btn-plus", "props": {{"text": "+"}}, "children": [], "on_event": {{"click": "calc.add"}}}},
        {{"type": "button", "id": "btn-equals", "props": {{"text": "="}}, "children": [], "on_event": {{"click": "calc.add"}}}}
      ],
      "on_event": null
    }}
  ],
  "style": {{}}
}}

INSTRUCTIONS:
1. Keep it SIMPLE - fewer components means valid JSON
2. Return COMPLETE JSON - don't truncate
3. Use short IDs: "btn-1" not "button-number-1"
4. Minimal props: only value, text, readonly, type, columns, gap
5. Output ONLY the JSON object - nothing else

REQUIRED ROOT STRUCTURE:
{{
  "type": "app",
  "title": "App Name",
  "layout": "vertical",
  "components": [/* max 10 components */],
  "style": {{}}
}}"""
    
    def __init__(
        self,
        tool_registry: Optional[ToolRegistry] = None,
        llm: Optional[BaseLLM] = None,
        service_registry: Optional[Any] = None,
        context_builder: Optional[Any] = None
    ):
        """
        Initialize the UI generator agent.
        
        Args:
            tool_registry: Registry of available tools
            llm: Optional LLM for generating UIs (if None, uses rule-based generation)
            service_registry: Service registry for BaaS
            context_builder: Context builder for intelligent prompts
        """
        self.tool_registry = tool_registry or ToolRegistry()
        self.templates = ComponentTemplates()
        self.llm = llm
        self.use_llm = llm is not None
        self.service_registry = service_registry
        self.context_builder = context_builder
        
        if self.use_llm:
            logger.info("UIGeneratorAgent: LLM-based generation enabled")
        else:
            logger.info("UIGeneratorAgent: Using rule-based generation")
    
    def get_system_prompt(self, context_str: str = "") -> str:
        """Get system prompt with current tool descriptions and service context."""
        base_prompt = self.SYSTEM_PROMPT.format(
            tools_description=self.tool_registry.get_tools_description()
        )
        
        if context_str:
            return f"{base_prompt}\n\n{context_str}"
        
        return base_prompt
    
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
            UISpec: Final UI specification
        """
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
                # Fall through to rule-based generation
        
        # Rule-based generation (fallback) - yield the JSON then the spec
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
        
        # Yield the JSON as a single token for rule-based
        yield json.dumps(ui_spec.model_dump())
        yield ui_spec
    
    def _generate_with_llm_stream(self, request: str):
        """
        Generate UI using LLM with structured output.
        
        Args:
            request: User's natural language request
            stream_callback: Optional callback(token) for real-time streaming
            
        Returns:
            UISpec: Generated UI specification
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
        
        # Create a single combined prompt (works better with Ollama)
        system_prompt = self.get_system_prompt(context_str)
        
        # Combine system and user prompts into one
        full_prompt = f"""{system_prompt}

===== USER REQUEST =====

Generate a UI for: "{request}"

Requirements:
1. Analyze what components are needed
2. Design the layout and arrangement
3. Add interactivity by binding appropriate tools
4. If services are available, bind components to service tools
5. Return ONLY the JSON specification (no extra text)

Output the complete UI specification as valid JSON now:"""
        
        logger.info(f"Sending prompt to LLM for: {request}")
        logger.debug(f"Full prompt length: {len(full_prompt)} characters")
        
        # CRITICAL: Get a fresh LLM instance for each request to prevent cache pollution
        # Import here to avoid circular dependency
        from models import ModelLoader, ModelConfig
        from models.config import ModelBackend, ModelSize
        
        llm = ModelLoader.load(ModelConfig(
            backend=ModelBackend.OLLAMA,
            size=ModelSize.SMALL,
            streaming=True,
            cache_prompt=False,  # Disable caching to prevent context pollution
            keep_alive="0"  # Unload immediately after request
        ))
        
        logger.debug("Using fresh LLM instance for streaming (cache_prompt=False)")
        
        # Stream tokens in real-time
        content = ""
        chunk_count = 0
        
        for chunk in llm.stream(full_prompt):
            chunk_count += 1
            if hasattr(chunk, 'content'):
                token = chunk.content
                content += token
                # Yield token immediately for real-time streaming
                if token:
                    yield token
        
        logger.info(f"LLM response: {chunk_count} chunks, {len(content)} characters")
        if content:
            logger.debug(f"Response preview: {content[:200]}...")
        else:
            logger.warning("LLM returned empty content after streaming")
        
        # Parse JSON from response
        ui_spec_json = self._extract_json(content)
        
        # Validate and convert to UISpec
        try:
            ui_spec = UISpec.model_validate(ui_spec_json)
            logger.info(f"Successfully generated UI: {ui_spec.title}")
            yield ui_spec  # Yield the final UISpec
        except Exception as e:
            logger.error(f"Failed to validate UISpec: {e}")
            logger.error(f"Response content: {content}")
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
        """Generate a calculator UI."""
        # Display
        display = self.templates.input("display", value="0")
        display.props["readonly"] = True
        display.props["style"] = {"fontSize": "24px", "textAlign": "right"}
        
        # Number buttons (0-9)
        number_buttons = []
        for num in ["7", "8", "9", "4", "5", "6", "1", "2", "3", "0"]:
            number_buttons.append(
                self.templates.button(f"btn-{num}", num, on_click="calc.append_digit")
            )
        
        # Operation buttons
        ops = [
            ("÷", "divide"),
            ("×", "multiply"),
            ("−", "subtract"),
            ("+", "add"),
        ]
        for symbol, op in ops:
            number_buttons.insert(
                number_buttons.index(
                    self.templates.button(f"btn-{['9','6','3'][ops.index((symbol, op))]}", ['9','6','3'][ops.index((symbol, op))], on_click="calc.append_digit")
                ) + 1 if ops.index((symbol, op)) < 3 else len(number_buttons),
                self.templates.button(f"btn-{op}", symbol, on_click=f"calc.{op}")
            )
        
        # Special buttons
        clear = self.templates.button("btn-clear", "C", on_click="calc.clear")
        equals = self.templates.button("btn-equals", "=", on_click="calc.evaluate")
        
        # Layout
        button_grid_children = [
            self.templates.button("btn-7", "7", on_click="calc.append_digit"),
            self.templates.button("btn-8", "8", on_click="calc.append_digit"),
            self.templates.button("btn-9", "9", on_click="calc.append_digit"),
            self.templates.button("btn-divide", "÷", on_click="calc.divide"),
            
            self.templates.button("btn-4", "4", on_click="calc.append_digit"),
            self.templates.button("btn-5", "5", on_click="calc.append_digit"),
            self.templates.button("btn-6", "6", on_click="calc.append_digit"),
            self.templates.button("btn-multiply", "×", on_click="calc.multiply"),
            
            self.templates.button("btn-1", "1", on_click="calc.append_digit"),
            self.templates.button("btn-2", "2", on_click="calc.append_digit"),
            self.templates.button("btn-3", "3", on_click="calc.append_digit"),
            self.templates.button("btn-subtract", "−", on_click="calc.subtract"),
            
            self.templates.button("btn-0", "0", on_click="calc.append_digit"),
            clear,
            equals,
            self.templates.button("btn-add", "+", on_click="calc.add"),
        ]
        
        button_grid = self.templates.grid("button-grid", button_grid_children, columns=4, gap=8)
        
        return UISpec(
            title="Calculator",
            layout="vertical",
            components=[display, button_grid],
            style={"padding": "20px", "maxWidth": "300px"}
        )
    
    def _generate_counter(self) -> UISpec:
        """Generate a simple counter UI."""
        count_display = self.templates.text("count", "0", variant="h1")
        count_display.props["style"] = {"fontSize": "48px", "fontWeight": "bold"}
        
        increment = self.templates.button("btn-inc", "Increment", on_click="calc.add")
        decrement = self.templates.button("btn-dec", "Decrement", on_click="calc.subtract")
        reset = self.templates.button("btn-reset", "Reset", on_click="calc.clear")
        
        buttons = self.templates.container("buttons", [increment, decrement, reset], layout="horizontal", gap=12)
        
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


# ============================================================================
# Exports
# ============================================================================

__all__ = [
    "UIGeneratorAgent",
    "UISpec",
    "UIComponent",
    "ToolRegistry",
    "ToolDefinition",
    "ComponentTemplates",
]

