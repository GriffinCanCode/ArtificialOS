"""Tests for agent modules."""

import pytest
from unittest.mock import MagicMock, AsyncMock

from src.agents.chat import ChatAgent, ChatHistory, ChatMessage
from src.agents.ui_generator import UIGenerator, Templates
from src.agents.tools import ToolRegistry, ToolDefinition
from src.agents.models import UISpec, UIComponent


# ============================================================================
# ChatAgent Tests
# ============================================================================

@pytest.mark.unit
def test_chat_message_creation():
    """Test chat message creation."""
    msg = ChatMessage(role="user", content="Hello", timestamp=1234567890)
    assert msg.role == "user"
    assert msg.content == "Hello"
    assert msg.timestamp == 1234567890


@pytest.mark.unit
def test_chat_message_validation():
    """Test chat message validation."""
    with pytest.raises(Exception):
        ChatMessage(role="invalid", content="test", timestamp=123)  # Invalid role
    
    with pytest.raises(Exception):
        ChatMessage(role="user", content="", timestamp=123)  # Empty content


@pytest.mark.unit
def test_chat_history_add():
    """Test adding messages to history."""
    history = ChatHistory()
    msg = ChatAgent.create_user_message("Hello")
    history.add(msg)
    
    assert len(history.messages) == 1
    assert history.messages[0].content == "Hello"


@pytest.mark.unit
def test_chat_history_limit():
    """Test history limit enforcement."""
    history = ChatHistory(max_history=3)
    
    for i in range(5):
        msg = ChatAgent.create_user_message(f"Message {i}")
        history.add(msg)
    
    assert len(history.messages) <= 3


@pytest.mark.unit
def test_chat_history_clear():
    """Test clearing non-system messages."""
    history = ChatHistory()
    history.add(ChatAgent.create_system_message("System"))
    history.add(ChatAgent.create_user_message("User"))
    
    history.clear()
    
    assert len(history.messages) == 1
    assert history.messages[0].role == "system"


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_agent_stream(chat_agent):
    """Test chat agent streaming."""
    tokens = []
    async for token in chat_agent.stream_response("Hello"):
        tokens.append(token)
    
    assert len(tokens) > 0
    assert "".join(tokens) == "test response"


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_agent_get_response(chat_agent):
    """Test chat agent non-streaming response."""
    response = await chat_agent.get_response("Hello")
    assert response == "test response"


# ============================================================================
# UIGenerator Tests
# ============================================================================

@pytest.mark.unit
def test_ui_generator_initialization(tool_registry):
    """Test UI generator initialization."""
    generator = UIGenerator(
        tool_registry=tool_registry,
        llm=None,
        backend_services=[],
        enable_cache=False
    )
    
    assert generator.tool_registry is tool_registry
    assert generator.use_llm is False


@pytest.mark.unit
def test_templates_button():
    """Test button template."""
    btn = Templates.button("submit", "Submit", "ui.submit", "primary")
    
    assert btn.type == "button"
    assert btn.id == "submit"
    assert btn.props["text"] == "Submit"
    assert btn.props["variant"] == "primary"
    assert btn.on_event["click"] == "ui.submit"


@pytest.mark.unit
def test_templates_input():
    """Test input template."""
    inp = Templates.input("name", placeholder="Enter name", value="John")
    
    assert inp.type == "input"
    assert inp.id == "name"
    assert inp.props["placeholder"] == "Enter name"
    assert inp.props["value"] == "John"


@pytest.mark.unit
def test_templates_text():
    """Test text template."""
    txt = Templates.text("header", "Welcome", "h1")
    
    assert txt.type == "text"
    assert txt.id == "header"
    assert txt.props["content"] == "Welcome"
    assert txt.props["variant"] == "h1"


@pytest.mark.unit
def test_templates_container():
    """Test container template."""
    children = [
        Templates.text("t1", "Text 1"),
        Templates.text("t2", "Text 2")
    ]
    container = Templates.container("main", children, layout="horizontal", gap=12)
    
    assert container.type == "container"
    assert container.id == "main"
    assert len(container.children) == 2
    assert container.props["layout"] == "horizontal"
    assert container.props["gap"] == 12


@pytest.mark.unit
def test_ui_generator_rule_based_calculator(ui_generator):
    """Test rule-based calculator generation."""
    spec = ui_generator.generate_ui("create a calculator")
    
    assert isinstance(spec, UISpec)
    assert spec.title == "Calculator"
    assert len(spec.components) > 0


@pytest.mark.unit
def test_ui_generator_rule_based_counter(ui_generator):
    """Test rule-based counter generation."""
    spec = ui_generator.generate_ui("make a counter")
    
    assert isinstance(spec, UISpec)
    assert spec.title == "Counter"
    assert len(spec.components) > 0


@pytest.mark.unit
def test_ui_generator_rule_based_todo(ui_generator):
    """Test rule-based todo generation."""
    spec = ui_generator.generate_ui("build a todo app")
    
    assert isinstance(spec, UISpec)
    assert spec.title == "Todo App"
    assert len(spec.components) > 0


@pytest.mark.unit
def test_ui_generator_stream(ui_generator):
    """Test UI generation streaming."""
    items = list(ui_generator.generate_ui_stream("calculator"))
    
    # Should have both string tokens and final UISpec
    has_strings = any(isinstance(item, str) for item in items)
    has_spec = any(isinstance(item, UISpec) for item in items)
    
    assert has_strings
    assert has_spec


# ============================================================================
# ToolRegistry Tests
# ============================================================================

@pytest.mark.unit
def test_tool_registry_initialization():
    """Test tool registry initialization."""
    registry = ToolRegistry()
    
    # Should have registered built-in tools
    assert len(registry.tools) > 0
    
    # Should have multiple categories
    categories = registry.get_categories()
    assert "ui" in categories
    assert "app" in categories


@pytest.mark.unit
def test_tool_registry_get_tool(tool_registry):
    """Test getting tool by ID."""
    tool = tool_registry.get_tool("ui.set")
    
    assert tool is not None
    assert tool.id == "ui.set"
    assert tool.category == "ui"


@pytest.mark.unit
def test_tool_registry_list_tools(tool_registry):
    """Test listing all tools."""
    all_tools = tool_registry.list_tools()
    assert len(all_tools) > 0
    
    ui_tools = tool_registry.list_tools(category="ui")
    assert len(ui_tools) > 0
    assert all(t.category == "ui" for t in ui_tools)


@pytest.mark.unit
def test_tool_registry_register_tool(tool_registry):
    """Test registering custom tool."""
    custom_tool = ToolDefinition(
        id="custom.action",
        name="Custom Action",
        description="Does something custom",
        parameters={"param": "string"},
        category="custom"
    )
    
    tool_registry.register_tool(custom_tool)
    
    retrieved = tool_registry.get_tool("custom.action")
    assert retrieved is not None
    assert retrieved.name == "Custom Action"


@pytest.mark.unit
def test_tool_registry_get_tools_description(tool_registry):
    """Test getting tools description for AI context."""
    description = tool_registry.get_tools_description()
    
    assert isinstance(description, str)
    assert "ui.set" in description
    assert "app.spawn" in description


# ============================================================================
# UISpec/UIComponent Models Tests
# ============================================================================

@pytest.mark.unit
def test_ui_component_creation():
    """Test UI component model."""
    comp = UIComponent(
        type="button",
        id="submit",
        props={"text": "Submit", "variant": "primary"},
        on_event={"click": "ui.submit"}
    )
    
    assert comp.type == "button"
    assert comp.id == "submit"
    assert comp.props["text"] == "Submit"


@pytest.mark.unit
def test_ui_component_with_children():
    """Test nested UI components."""
    child = UIComponent(type="text", id="label", props={"content": "Label"})
    parent = UIComponent(
        type="container",
        id="wrapper",
        props={},
        children=[child]
    )
    
    assert len(parent.children) == 1
    assert parent.children[0].type == "text"


@pytest.mark.unit
def test_ui_spec_creation(sample_ui_spec):
    """Test UI spec model."""
    spec = UISpec(**sample_ui_spec)
    
    assert spec.type == "app"
    assert spec.title == "Test App"
    assert spec.layout == "vertical"
    assert len(spec.components) == 2


@pytest.mark.unit
def test_ui_spec_serialization(sample_ui_spec):
    """Test UI spec serialization."""
    spec = UISpec(**sample_ui_spec)
    data = spec.model_dump()
    
    assert data["title"] == "Test App"
    assert len(data["components"]) == 2
    
    # Can reconstruct from dict
    spec2 = UISpec(**data)
    assert spec2.title == spec.title

