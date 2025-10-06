"""UI Generator Agent - Fast JSON with error handling."""

from typing import Any
from collections.abc import Iterator
from langchain_core.language_models import BaseLLM

from core import get_logger, extract_json, safe_json_dumps, JSONParseError
from blueprint import parse_blueprint
from .prompt import get_ui_generation_prompt
from .tools import ToolRegistry
from .ui_cache import UICache
from .prompts import PromptBuilder
from .models import BlueprintComponent, Blueprint


logger = get_logger(__name__)


class Templates:
    """Component templates."""

    @staticmethod
    def button(id: str, text: str, on_click: str | None = None, variant: str = "default") -> BlueprintComponent:
        return BlueprintComponent(
            type="button",
            id=id,
            props={"text": text, "variant": variant},
            on_event={"click": on_click} if on_click else None
        )

    @staticmethod
    def input(id: str, placeholder: str = "", value: str = "", readonly: bool = False) -> BlueprintComponent:
        return BlueprintComponent(
            type="input",
            id=id,
            props={"placeholder": placeholder, "value": value, "readonly": readonly}
        )

    @staticmethod
    def text(id: str, content: str, variant: str = "body") -> BlueprintComponent:
        return BlueprintComponent(type="text", id=id, props={"content": content, "variant": variant})

    @staticmethod
    def container(id: str, children: list[BlueprintComponent], layout: str = "vertical", gap: int = 0) -> BlueprintComponent:
        return BlueprintComponent(
            type="container",
            id=id,
            props={"layout": layout, "gap": gap},
            children=children
        )

    @staticmethod
    def grid(id: str, children: list[BlueprintComponent], columns: int = 3, gap: int = 0) -> BlueprintComponent:
        return BlueprintComponent(
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
        llm: BaseLLM | None = None,
        backend_services: list[Any] | None = None,
        enable_cache: bool = True
    ) -> None:
        self.tool_registry = tool_registry
        self.templates = Templates()
        self.llm = llm
        self.use_llm = llm is not None
        self.backend_services = backend_services or []
        self.cache = UICache(max_size=100, ttl_seconds=3600) if enable_cache else None

        logger.info("initialized", mode="llm" if self.use_llm else "rule-based")

    def generate_ui(self, request: str) -> Blueprint:
        """Generate Blueprint (non-streaming)."""
        for item in self.generate_ui_stream(request):
            if isinstance(item, Blueprint):
                return item
        raise ValueError("No Blueprint generated")

    def generate_ui_stream(self, request: str) -> Iterator[str | dict | Blueprint]:
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

    def _generate_llm(self, request: str) -> Iterator[str | Blueprint]:
        """Generate with LLM - stream JSON in small chunks for incremental component rendering."""
        if not self.llm:
            raise ValueError("LLM not configured")

        tools_desc = self.tool_registry.get_tools_description()
        if self.backend_services:
            tools_desc += self._format_backend_services()

        prompt = get_ui_generation_prompt(tools_desc, "")
        prompt += f"\n\n=== REQUEST ===\n{request}\n\nGenerate Blueprint:"

        logger.info("llm_generate")

        # Stream LLM tokens in real-time while accumulating
        content = ""
        started_json = False
        buffer = ""  # Buffer for chunking
        CHUNK_SIZE = 50  # Send ~50 chars at a time for smooth component rendering

        for token in self.llm.stream(prompt):
            token_str = token.content if hasattr(token, 'content') else str(token)
            content += token_str

            # Start streaming once we see the opening brace
            if not started_json and "{" in content:
                started_json = True
                # Extract clean JSON (skip any markdown before the {)
                json_start = content.find("{")
                clean_content = content[json_start:]
                buffer = clean_content
            elif started_json:
                # Add to buffer
                buffer += token_str

            # Yield buffer in chunks for smooth incremental updates
            while len(buffer) >= CHUNK_SIZE:
                chunk = buffer[:CHUNK_SIZE]
                buffer = buffer[CHUNK_SIZE:]
                yield chunk

        # Yield any remaining buffer
        if buffer and started_json:
            yield buffer

        # Clean up complete response for parsing
        logger.info("llm_complete", content_length=len(content))

        json_content = content.strip()
        if json_content.startswith("```"):
            # Find the end of the first line (could be ```json or just ```)
            first_newline = json_content.find("\n")
            if first_newline != -1:
                json_content = json_content[first_newline + 1:]
            # Remove the closing ```
            if json_content.endswith("```"):
                json_content = json_content[:-3].strip()

        # Remove leading --- if present (legacy YAML document separator)
        if json_content.startswith("---"):
            json_content = json_content[3:].strip()

        # Remove any additional --- separators (multiple documents)
        if "\n---" in json_content:
            json_content = json_content.split("\n---")[0].strip()

        logger.info("llm_cleaned_json", json_length=len(json_content), preview=json_content[:200])

        # Parse complete Blueprint JSON to Blueprint
        logger.info("llm_parsing")
        try:

            # Parse Blueprint JSON to package dict
            package_dict = parse_blueprint(json_content)
            # Extract UI spec from package
            ui_spec_dict = package_dict.get("ui_spec", {})
            ui_spec = Blueprint.model_validate(ui_spec_dict)

            logger.info("llm_parse_success", components=len(ui_spec.components))
        except Exception as e:
            logger.error("llm_parse_failed", error=str(e), content_preview=content[:500])
            raise

        if self.cache:
            self.cache.set(request, ui_spec)

        yield ui_spec

    def _generate_rules(self, request: str) -> Iterator[str | Blueprint]:
        """Rule-based generation - outputs Blueprint JSON format."""
        request_lower = request.lower()

        # Use pattern matching for cleaner app type detection
        match request_lower:
            case s if "calculator" in s:
                blueprint_json = self._build_calculator_blueprint()
                title = "Calculator"
            case s if "counter" in s:
                blueprint_json = self._build_counter_blueprint()
                title = "Counter"
            case s if "todo" in s:
                blueprint_json = self._build_todo_blueprint()
                title = "Todo App"
            case _:
                blueprint_json = self._build_placeholder_blueprint(request)
                title = "Generated App"

        # Stream Blueprint JSON
        for i in range(0, len(blueprint_json), 50):
            yield blueprint_json[i:i+50]

        # Parse to Blueprint for caching
        try:
            package_dict = parse_blueprint(blueprint_json)
            ui_spec = Blueprint.model_validate(package_dict.get("ui_spec", {}))
            if self.cache:
                self.cache.set(request, ui_spec)
            yield ui_spec
        except Exception as e:
            logger.warning("rule_based_parse_failed", error=str(e))
            # Still yield a minimal Blueprint
            ui_spec = Blueprint(title=title, components=[])
            yield ui_spec

    def _build_calculator_blueprint(self) -> str:
        """Generate Calculator Blueprint JSON - explicit format for streaming."""
        return """{
  "app": {
    "id": "calculator",
    "name": "Calculator",
    "version": "1.0.0",
    "author": "system",
    "permissions": ["STANDARD"]
  },
  "services": [],
  "ui": {
    "title": "Calculator",
    "layout": "vertical",
    "components": [
      {"type": "input", "id": "display", "props": {"value": "0", "readonly": true}},
      {
        "type": "grid",
        "id": "buttons",
        "props": {"columns": 4, "gap": 8},
        "children": [
          {"type": "button", "id": "7", "props": {"text": "7"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "8", "props": {"text": "8"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "9", "props": {"text": "9"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "div", "props": {"text": "÷"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "4", "props": {"text": "4"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "5", "props": {"text": "5"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "6", "props": {"text": "6"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "mul", "props": {"text": "×"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "1", "props": {"text": "1"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "2", "props": {"text": "2"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "3", "props": {"text": "3"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "sub", "props": {"text": "−"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "0", "props": {"text": "0"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "clear", "props": {"text": "C"}, "on_event": {"click": "ui.clear"}},
          {"type": "button", "id": "equals", "props": {"text": "="}, "on_event": {"click": "ui.compute"}},
          {"type": "button", "id": "add", "props": {"text": "+"}, "on_event": {"click": "ui.append"}}
        ]
      }
    ]
  }
}"""

    def _build_counter_blueprint(self) -> str:
        """Generate Counter Blueprint JSON - explicit format for streaming."""
        return """{
  "app": {
    "id": "counter",
    "name": "Counter",
    "version": "1.0.0",
    "author": "system",
    "permissions": ["STANDARD"]
  },
  "services": [],
  "ui": {
    "title": "Counter",
    "layout": "vertical",
    "components": [
      {"type": "input", "id": "count", "props": {"value": "0", "readonly": true}},
      {
        "type": "row",
        "id": "controls",
        "props": {"gap": 12},
        "children": [
          {"type": "button", "id": "inc", "props": {"text": "+1"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "dec", "props": {"text": "-1"}, "on_event": {"click": "ui.append"}},
          {"type": "button", "id": "compute", "props": {"text": "="}, "on_event": {"click": "ui.compute"}},
          {"type": "button", "id": "reset", "props": {"text": "Clear"}, "on_event": {"click": "ui.clear"}}
        ]
      }
    ]
  }
}"""

    def _build_todo_blueprint(self) -> str:
        """Generate Todo Blueprint JSON - explicit format for streaming."""
        return """{
  "app": {
    "id": "todo",
    "name": "Todo App",
    "version": "1.0.0",
    "author": "system",
    "permissions": ["STANDARD"]
  },
  "services": [],
            "ui": {
                "title": "Todo App",
                "layout": "vertical",
                "components": [
                    {"type": "text", "id": "header", "props": {"content": "Todo List", "variant": "h2"}},
      {
        "type": "row",
        "id": "input",
        "props": {"gap": 8},
        "children": [
          {"type": "input", "id": "task-input", "props": {"placeholder": "New task..."}},
          {"type": "button", "id": "add", "props": {"text": "Add"}, "on_event": {"click": "ui.list.add"}}
        ]
      },
      {
        "type": "col",
        "id": "list",
        "props": {"gap": 8}
      }
    ]
  }
}"""

    def _build_placeholder_blueprint(self, request: str) -> str:
        """Generate placeholder Blueprint JSON - explicit format for streaming."""
        import json
        return json.dumps({
            "app": {
                "id": "generated-app",
                "name": "Generated App",
                "version": "1.0.0",
                "author": "system",
                "permissions": ["STANDARD"]
            },
            "services": [],
            "ui": {
                "title": "Generated App",
                "layout": "vertical",
                "components": [
                    {"type": "text", "id": "header", "props": {"content": "Generated App", "variant": "h2"}},
                    {"type": "text", "id": "msg", "props": {"content": f"Request: {request}", "variant": "body"}}
                ]
            }
        }, indent=2)

    # Legacy methods kept for compatibility
    def _build_calculator(self) -> Blueprint:
        """Build calculator UI (legacy)."""
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
        return Blueprint(title="Calculator", components=[display, grid])

    def _build_counter(self) -> Blueprint:
        """Build counter UI (legacy)."""
        display = self.templates.input("count", value="0", readonly=True)
        inc = self.templates.button("inc", "+1", "ui.append")
        dec = self.templates.button("dec", "-1", "ui.append")
        compute = self.templates.button("compute", "=", "ui.compute")
        reset = self.templates.button("reset", "Clear", "ui.clear")
        buttons = self.templates.container("controls", [inc, dec, compute, reset], layout="horizontal", gap=12)
        return Blueprint(title="Counter", components=[display, buttons])

    def _build_todo(self) -> Blueprint:
        """Build todo UI (legacy)."""
        header = self.templates.text("header", "Todo List", variant="h2")
        task_input = self.templates.input("task-input", placeholder="New task...")
        add_btn = self.templates.button("add", "Add", "ui.add_todo")
        input_row = self.templates.container("input", [task_input, add_btn], layout="horizontal", gap=8)
        todo_list = self.templates.container("list", [], layout="vertical", gap=8)
        return Blueprint(title="Todo App", components=[header, input_row, todo_list])

    def _build_placeholder(self, request: str) -> Blueprint:
        """Build placeholder UI (legacy)."""
        header = self.templates.text("header", "Generated App", variant="h2")
        message = self.templates.text("msg", f"Request: {request}", variant="body")
        return Blueprint(title="Generated App", components=[header, message])

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
