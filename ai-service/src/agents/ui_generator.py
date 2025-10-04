"""UI Generator Agent - Fast JSON with error handling."""

from typing import Any, List, Optional, Iterator
from langchain_core.language_models import BaseLLM

from core import get_logger, extract_json, safe_json_dumps, JSONParseError
from .prompt import get_ui_generation_prompt
from .tools import ToolRegistry
from .ui_cache import UICache
from .prompts import PromptBuilder
from .models import UIComponent, UISpec


logger = get_logger(__name__)


class Templates:
    """Component templates."""
    
    @staticmethod
    def button(id: str, text: str, on_click: str | None = None, variant: str = "default") -> UIComponent:
        return UIComponent(
            type="button",
            id=id,
            props={"text": text, "variant": variant},
            on_event={"click": on_click} if on_click else None
        )
    
    @staticmethod
    def input(id: str, placeholder: str = "", value: str = "", readonly: bool = False) -> UIComponent:
        return UIComponent(
            type="input",
            id=id,
            props={"placeholder": placeholder, "value": value, "readonly": readonly}
        )
    
    @staticmethod
    def text(id: str, content: str, variant: str = "body") -> UIComponent:
        return UIComponent(type="text", id=id, props={"content": content, "variant": variant})
    
    @staticmethod
    def container(id: str, children: List[UIComponent], layout: str = "vertical", gap: int = 0) -> UIComponent:
        return UIComponent(
            type="container",
            id=id,
            props={"layout": layout, "gap": gap},
            children=children
        )
    
    @staticmethod
    def grid(id: str, children: List[UIComponent], columns: int = 3, gap: int = 0) -> UIComponent:
        return UIComponent(
            type="grid",
            id=id,
            props={"columns": columns, "gap": gap},
            children=children
        )


class UIGenerator:
    """Generates UI specifications from natural language."""
    
    def __init__(
        self,
        tool_registry: ToolRegistry,
        llm: Optional[BaseLLM] = None,
        backend_services: List[Any] = None,
        enable_cache: bool = True
    ):
        self.tool_registry = tool_registry
        self.templates = Templates()
        self.llm = llm
        self.use_llm = llm is not None
        self.backend_services = backend_services or []
        self.cache = UICache(max_size=100, ttl_seconds=3600) if enable_cache else None
        
        logger.info("initialized", mode="llm" if self.use_llm else "rule-based")
    
    def generate_ui(self, request: str) -> UISpec:
        """Generate UI spec (non-streaming)."""
        for item in self.generate_ui_stream(request):
            if isinstance(item, UISpec):
                return item
        raise ValueError("No UI spec generated")
    
    def generate_ui_stream(self, request: str) -> Iterator[str | dict | UISpec]:
        """Stream UI generation."""
        # Check cache
        if self.cache:
            cached = self.cache.get(request)
            if cached:
                logger.info("cache_hit")
                json_str = safe_json_dumps(cached.model_dump(), indent=2)
                for i in range(0, len(json_str), 100):
                    yield json_str[i:i+100]
                yield cached
                return
        
        # Try LLM generation
        if self.use_llm:
            try:
                yield from self._generate_llm(request)
                return
            except Exception as e:
                logger.warning("llm_failed", error=str(e))
                yield {"reset": True}
        
        # Fallback to rule-based
        yield from self._generate_rules(request)
    
    def _generate_llm(self, request: str) -> Iterator[str | UISpec]:
        """Generate with LLM."""
        tools_desc = self.tool_registry.get_tools_description()
        if self.backend_services:
            tools_desc += self._format_backend_services()
        
        prompt = get_ui_generation_prompt(tools_desc, "")
        prompt += f"\n\n=== REQUEST ===\n{request}\n\nGenerate JSON:"
        
        logger.info("llm_generate")
        
        content = ""
        for token in self.llm.stream(prompt):
            token_str = token.content if hasattr(token, 'content') else str(token)
            content += token_str
            if token_str:
                yield token_str
        
        # Parse response with msgspec cascade (fast)
        try:
            spec_dict = extract_json(content, repair=True)
            ui_spec = UISpec.model_validate(spec_dict)
        except JSONParseError as e:
            logger.error("json_parse_failed", error=str(e))
            raise
        except Exception as e:
            logger.error("validation_failed", error=str(e))
            raise
        
        if self.cache:
            self.cache.set(request, ui_spec)
        
        yield ui_spec
    
    def _generate_rules(self, request: str) -> Iterator[str | UISpec]:
        """Rule-based generation."""
        request_lower = request.lower()
        
        if "calculator" in request_lower:
            ui_spec = self._build_calculator()
        elif "counter" in request_lower:
            ui_spec = self._build_counter()
        elif "todo" in request_lower:
            ui_spec = self._build_todo()
        else:
            ui_spec = self._build_placeholder(request)
        
        # Stream JSON (uses orjson for speed)
        json_str = safe_json_dumps(ui_spec.model_dump(), indent=2)
        for i in range(0, len(json_str), 50):
            yield json_str[i:i+50]
        
        if self.cache:
            self.cache.set(request, ui_spec)
        
        yield ui_spec
    
    def _build_calculator(self) -> UISpec:
        """Build calculator UI."""
        display = self.templates.input("display", value="0", readonly=True)
        buttons = [
            self.templates.button("7", "7", "ui.append"),
            self.templates.button("8", "8", "ui.append"),
            self.templates.button("9", "9", "ui.append"),
            self.templates.button("div", "÷", "ui.append"),
            self.templates.button("4", "4", "ui.append"),
            self.templates.button("5", "5", "ui.append"),
            self.templates.button("6", "6", "ui.append"),
            self.templates.button("mul", "×", "ui.append"),
            self.templates.button("1", "1", "ui.append"),
            self.templates.button("2", "2", "ui.append"),
            self.templates.button("3", "3", "ui.append"),
            self.templates.button("sub", "−", "ui.append"),
            self.templates.button("0", "0", "ui.append"),
            self.templates.button("clear", "C", "ui.clear"),
            self.templates.button("equals", "=", "ui.compute"),
            self.templates.button("add", "+", "ui.append"),
        ]
        grid = self.templates.grid("buttons", buttons, columns=4, gap=8)
        return UISpec(title="Calculator", components=[display, grid])
    
    def _build_counter(self) -> UISpec:
        """Build counter UI."""
        display = self.templates.input("count", value="0", readonly=True)
        inc = self.templates.button("inc", "+1", "ui.append")
        dec = self.templates.button("dec", "-1", "ui.append")
        compute = self.templates.button("compute", "=", "ui.compute")
        reset = self.templates.button("reset", "Clear", "ui.clear")
        buttons = self.templates.container("controls", [inc, dec, compute, reset], layout="horizontal", gap=12)
        return UISpec(title="Counter", components=[display, buttons])
    
    def _build_todo(self) -> UISpec:
        """Build todo UI."""
        header = self.templates.text("header", "📝 Todo List", variant="h2")
        task_input = self.templates.input("task-input", placeholder="New task...")
        add_btn = self.templates.button("add", "Add", "ui.add_todo")
        input_row = self.templates.container("input", [task_input, add_btn], layout="horizontal", gap=8)
        todo_list = self.templates.container("list", [], layout="vertical", gap=8)
        return UISpec(title="Todo App", components=[header, input_row, todo_list])
    
    def _build_placeholder(self, request: str) -> UISpec:
        """Build placeholder UI."""
        header = self.templates.text("header", "Generated App", variant="h2")
        message = self.templates.text("msg", f"Request: {request}", variant="body")
        return UISpec(title="Generated App", components=[header, message])
    
    def _format_backend_services(self) -> str:
        """Format backend services for prompt."""
        if not self.backend_services:
            return ""
        lines = ["\n=== BACKEND SERVICES ==="]
        for svc in self.backend_services:
            lines.append(f"\n{svc.name}:")
            for tool in svc.tools:
                params = ", ".join(f"{p.name}:{p.type}" for p in tool.parameters)
                lines.append(f"  - {tool.id}({params}): {tool.description}")
        return "\n".join(lines)


# Alias for backwards compatibility
UIGeneratorAgent = UIGenerator
