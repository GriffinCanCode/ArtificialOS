"""Tests for gRPC handlers."""

import pytest
import time
from unittest.mock import MagicMock, AsyncMock

import ai_pb2

from src.handlers.ui import UIHandler
from src.handlers.chat import ChatHandler
from src.core import ValidationError


# ============================================================================
# UIHandler Tests
# ============================================================================

@pytest.mark.unit
def test_ui_handler_initialization(ui_generator):
    """Test UI handler initialization."""
    handler = UIHandler(ui_generator)
    assert handler.ui_generator is ui_generator


@pytest.mark.unit
def test_ui_handler_generate_success(ui_generator):
    """Test successful UI generation."""
    handler = UIHandler(ui_generator)
    
    request = ai_pb2.UIRequest(message="create a calculator")
    response = handler.generate(request)
    
    assert response.success is True
    assert len(response.ui_spec_json) > 0
    assert len(response.thoughts) > 0
    assert response.error == ""


@pytest.mark.unit
def test_ui_handler_generate_empty_message(ui_generator):
    """Test UI generation with empty message."""
    handler = UIHandler(ui_generator)
    
    request = ai_pb2.UIRequest(message="")
    response = handler.generate(request)
    
    assert response.success is False
    assert len(response.error) > 0


@pytest.mark.unit
def test_ui_handler_generate_validation_error(ui_generator):
    """Test UI generation with validation error."""
    handler = UIHandler(ui_generator)
    
    # Message too long
    request = ai_pb2.UIRequest(message="x" * 20000)
    response = handler.generate(request)
    
    assert response.success is False
    assert "Validation" in response.error


@pytest.mark.unit
def test_ui_handler_stream(ui_generator):
    """Test UI streaming."""
    handler = UIHandler(ui_generator)
    
    request = ai_pb2.UIRequest(message="calculator")
    tokens = list(handler.stream(request))
    
    # Should have start, tokens, and complete
    assert len(tokens) > 0
    
    # First token should be GENERATION_START
    assert tokens[0].type == ai_pb2.UIToken.GENERATION_START
    
    # Last token should be COMPLETE
    assert tokens[-1].type == ai_pb2.UIToken.COMPLETE


@pytest.mark.unit
def test_ui_handler_stream_with_thoughts(ui_generator):
    """Test UI streaming with thoughts."""
    handler = UIHandler(ui_generator)
    
    request = ai_pb2.UIRequest(message="todo app")
    tokens = list(handler.stream(request))
    
    # Should have thought tokens
    thought_tokens = [t for t in tokens if t.type == ai_pb2.UIToken.THOUGHT]
    assert len(thought_tokens) > 0


@pytest.mark.unit
def test_ui_handler_stream_error(ui_generator):
    """Test UI streaming with error."""
    handler = UIHandler(ui_generator)
    
    request = ai_pb2.UIRequest(message="")
    tokens = list(handler.stream(request))
    
    # Should have error token
    error_tokens = [t for t in tokens if t.type == ai_pb2.UIToken.ERROR]
    assert len(error_tokens) > 0


# ============================================================================
# ChatHandler Tests
# ============================================================================

@pytest.mark.unit
def test_chat_handler_initialization():
    """Test chat handler initialization."""
    from src.models.loader import ModelLoader
    handler = ChatHandler(ModelLoader)
    assert handler.model_loader is ModelLoader


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_handler_stream(mocker):
    """Test chat streaming."""
    from src.models.loader import ModelLoader
    
    # Mock model loader
    mock_model = MagicMock()
    
    async def astream_mock(prompt):
        for token in ["hello", " ", "world"]:
            yield token
    
    mock_model.astream = astream_mock
    
    mocker.patch.object(ModelLoader, 'load', return_value=mock_model)
    
    handler = ChatHandler(ModelLoader)
    
    request = ai_pb2.ChatRequest(
        message="Hello",
        history=[]
    )
    
    tokens = []
    async for token in handler.stream(request):
        tokens.append(token)
    
    assert len(tokens) > 0
    
    # Should have start, content tokens, and complete
    assert tokens[0].type == ai_pb2.ChatToken.GENERATION_START
    assert tokens[-1].type == ai_pb2.ChatToken.COMPLETE


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_handler_stream_with_history(mocker):
    """Test chat streaming with history."""
    from src.models.loader import ModelLoader
    
    mock_model = MagicMock()
    
    async def astream_mock(prompt):
        yield "response"
    
    mock_model.astream = astream_mock
    
    mocker.patch.object(ModelLoader, 'load', return_value=mock_model)
    
    handler = ChatHandler(ModelLoader)
    
    history = [
        ai_pb2.ChatMessage(role="user", content="Hi", timestamp=int(time.time())),
        ai_pb2.ChatMessage(role="assistant", content="Hello", timestamp=int(time.time()))
    ]
    
    request = ai_pb2.ChatRequest(
        message="How are you?",
        history=history
    )
    
    tokens = []
    async for token in handler.stream(request):
        tokens.append(token)
    
    assert len(tokens) > 0


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_handler_stream_validation_error(mocker):
    """Test chat streaming with validation error."""
    from src.models.loader import ModelLoader
    
    handler = ChatHandler(ModelLoader)
    
    request = ai_pb2.ChatRequest(
        message="",  # Empty message
        history=[]
    )
    
    tokens = []
    async for token in handler.stream(request):
        tokens.append(token)
    
    # Should have error token
    error_tokens = [t for t in tokens if t.type == ai_pb2.ChatToken.ERROR]
    assert len(error_tokens) > 0


@pytest.mark.unit
@pytest.mark.asyncio
async def test_chat_handler_stream_exception(mocker):
    """Test chat streaming with exception."""
    from src.models.loader import ModelLoader
    
    # Mock model loader to raise exception
    mocker.patch.object(
        ModelLoader, 
        'load', 
        side_effect=Exception("Model load failed")
    )
    
    handler = ChatHandler(ModelLoader)
    
    request = ai_pb2.ChatRequest(
        message="Hello",
        history=[]
    )
    
    tokens = []
    async for token in handler.stream(request):
        tokens.append(token)
    
    # Should have error token
    error_tokens = [t for t in tokens if t.type == ai_pb2.ChatToken.ERROR]
    assert len(error_tokens) > 0
    assert "Model load failed" in error_tokens[0].content


@pytest.mark.integration
def test_ui_handler_full_workflow(ui_generator):
    """Test full UI generation workflow."""
    handler = UIHandler(ui_generator)
    
    # Non-streaming
    request = ai_pb2.UIRequest(message="calculator")
    response = handler.generate(request)
    assert response.success is True
    
    # Streaming
    tokens = list(handler.stream(request))
    assert len(tokens) > 0


@pytest.mark.integration
@pytest.mark.asyncio
async def test_chat_handler_full_workflow(mocker):
    """Test full chat workflow."""
    from src.models.loader import ModelLoader
    
    mock_model = MagicMock()
    
    async def astream_mock(prompt):
        for token in ["test", " ", "response"]:
            yield token
    
    mock_model.astream = astream_mock
    
    mocker.patch.object(ModelLoader, 'load', return_value=mock_model)
    
    handler = ChatHandler(ModelLoader)
    
    # First message
    request1 = ai_pb2.ChatRequest(message="Hello", history=[])
    tokens1 = []
    async for token in handler.stream(request1):
        tokens1.append(token)
    
    # Second message with history
    history = [ai_pb2.ChatMessage(role="user", content="Hello", timestamp=int(time.time()))]
    request2 = ai_pb2.ChatRequest(message="How are you?", history=history)
    tokens2 = []
    async for token in handler.stream(request2):
        tokens2.append(token)
    
    assert len(tokens1) > 0
    assert len(tokens2) > 0

