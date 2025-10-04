"""Validation tests."""

import pytest
from hypothesis import given, strategies as st

from src.core import (
    UIGenerationRequest,
    ChatRequest,
    ValidationError,
    validate_json_size,
    validate_json_depth,
    JSONParseError
)


def test_ui_generation_request_valid():
    """Test valid UI generation request."""
    req = UIGenerationRequest(message="Create a calculator")
    assert req.message == "Create a calculator"


def test_ui_generation_request_empty():
    """Test empty message validation."""
    with pytest.raises(Exception):
        UIGenerationRequest(message="")


def test_ui_generation_request_whitespace():
    """Test whitespace-only message validation."""
    with pytest.raises(Exception):
        UIGenerationRequest(message="   ")


def test_chat_request_valid():
    """Test valid chat request."""
    req = ChatRequest(message="Hello", history_count=5)
    assert req.message == "Hello"
    assert req.history_count == 5


def test_chat_request_history_limit():
    """Test chat history limit validation."""
    with pytest.raises(Exception):
        ChatRequest(message="Hello", history_count=100)


def test_validate_json_size():
    """Test JSON size validation."""
    small_data = '{"test": "data"}'
    validate_json_size(small_data, 1000)  # Should pass
    
    large_data = "x" * 1_000_000
    with pytest.raises(JSONParseError):
        validate_json_size(large_data, 1000)


def test_validate_json_depth():
    """Test JSON depth validation."""
    shallow = {"a": {"b": {"c": 1}}}
    validate_json_depth(shallow, max_depth=5)  # Should pass
    
    # Create deeply nested structure
    deep = {"level": 1}
    current = deep
    for i in range(25):
        current["nested"] = {"level": i + 2}
        current = current["nested"]
    
    with pytest.raises(JSONParseError):
        validate_json_depth(deep, max_depth=20)


@given(st.text(min_size=1, max_size=1000))
def test_message_validation_property(message):
    """Property test: Any non-empty string should be valid."""
    if message.strip():  # Only test non-whitespace strings
        req = UIGenerationRequest(message=message)
        assert req.message == message.strip()

