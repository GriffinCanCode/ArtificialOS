"""Pytest configuration and fixtures."""

import os
import pytest
import asyncio
from unittest.mock import MagicMock, AsyncMock, Mock
from typing import AsyncGenerator, Generator

import httpx
import respx

from src.core import create_container, get_settings
from src.models.config import GeminiConfig
from src.agents.tools import ToolRegistry
from src.agents.ui_generator import UIGenerator
from src.agents.chat import ChatAgent, ChatHistory


# ============================================================================
# Pytest Hooks
# ============================================================================

def pytest_configure(config):
    """Configure pytest with environment variables."""
    # Set test environment variables
    os.environ['AI_GRPC_PORT'] = '50053'  # Different port for tests
    os.environ['AI_LOG_LEVEL'] = 'DEBUG'
    os.environ['AI_ENABLE_CACHE'] = 'false'  # Disable cache in tests
    os.environ['GOOGLE_API_KEY'] = 'test-api-key'  # Mock API key


# ============================================================================
# Core Fixtures
# ============================================================================

@pytest.fixture
def settings():
    """Test settings."""
    return get_settings()


@pytest.fixture
def di_container():
    """Dependency injection container for testing."""
    return create_container()


@pytest.fixture
def tool_registry():
    """Tool registry fixture."""
    return ToolRegistry()


# ============================================================================
# Model Fixtures
# ============================================================================

@pytest.fixture
def gemini_config():
    """Gemini config for testing."""
    return GeminiConfig(
        model_name="gemini-2.0-flash-exp",
        api_key="test-api-key",
        temperature=0.1,
        max_tokens=1024,
        streaming=True
    )


@pytest.fixture
def mock_gemini_model():
    """Mock Gemini model for testing."""
    mock = MagicMock()
    mock.stream.return_value = iter(["test", " ", "response"])
    mock.invoke.return_value = "test response"
    mock.config = MagicMock()
    mock.config.model_name = "gemini-2.0-flash-exp"
    
    # Async methods
    async def astream_mock(prompt):
        for token in ["test", " ", "response"]:
            yield token
    
    async def ainvoke_mock(prompt):
        return "test response"
    
    mock.astream = astream_mock
    mock.ainvoke = ainvoke_mock
    
    return mock


# ============================================================================
# Agent Fixtures
# ============================================================================

@pytest.fixture
def ui_generator(tool_registry, mock_gemini_model):
    """UI generator with mocked LLM."""
    return UIGenerator(
        tool_registry=tool_registry,
        llm=mock_gemini_model,
        backend_services=[],
        enable_cache=False
    )


@pytest.fixture
def chat_agent(mock_gemini_model):
    """Chat agent with mocked LLM."""
    return ChatAgent(llm=mock_gemini_model)


@pytest.fixture
def chat_history():
    """Empty chat history."""
    return ChatHistory()


# ============================================================================
# HTTP/Network Fixtures
# ============================================================================

@pytest.fixture
def mock_httpx_client():
    """Mock httpx client."""
    with respx.mock:
        yield respx


@pytest.fixture
def mock_backend_services():
    """Mock backend service discovery response."""
    return {
        "services": [
            {
                "id": "storage",
                "name": "Storage Service",
                "description": "Key-value storage",
                "category": "data",
                "capabilities": ["persistent", "encrypted"],
                "tools": [
                    {
                        "id": "storage.get",
                        "name": "Get Value",
                        "description": "Get value from storage",
                        "parameters": [
                            {"name": "key", "type": "string", "description": "Storage key", "required": True}
                        ],
                        "returns": "any"
                    },
                    {
                        "id": "storage.set",
                        "name": "Set Value",
                        "description": "Set value in storage",
                        "parameters": [
                            {"name": "key", "type": "string", "description": "Storage key", "required": True},
                            {"name": "value", "type": "any", "description": "Value to store", "required": True}
                        ],
                        "returns": "void"
                    }
                ]
            }
        ]
    }


# ============================================================================
# gRPC Fixtures
# ============================================================================

@pytest.fixture
def mock_grpc_context():
    """Mock gRPC context."""
    context = MagicMock()
    context.set_code = MagicMock()
    context.set_details = MagicMock()
    return context


# ============================================================================
# Data Fixtures
# ============================================================================

@pytest.fixture
def sample_ui_spec():
    """Sample UI specification."""
    return {
        "type": "app",
        "title": "Test App",
        "layout": "vertical",
        "components": [
            {
                "type": "text",
                "id": "header",
                "props": {"content": "Hello World", "variant": "h1"}
            },
            {
                "type": "button",
                "id": "submit",
                "props": {"text": "Submit", "variant": "primary"},
                "on_event": {"click": "ui.submit"}
            }
        ]
    }


@pytest.fixture
def sample_blueprint():
    """Sample Blueprint JSON."""
    return """{
  "app": {
    "id": "test-app",
    "name": "Test App",
    "icon": "🧪",
    "category": "utilities",
    "tags": ["test"],
    "permissions": ["STANDARD"]
  },
  "services": [
    {
      "storage": ["get", "set"]
    }
  ],
  "ui": {
    "title": "Test App",
    "layout": "vertical",
    "components": [
      {
        "text#header": {
          "content": "Test Header",
          "variant": "h1"
        }
      },
      {
        "button#submit": {
          "text": "Submit",
          "variant": "primary",
          "@click": "ui.submit"
        }
      }
    ]
  }
}"""


# ============================================================================
# Event Loop Fixtures (for async tests)
# ============================================================================

@pytest.fixture(scope="session")
def event_loop_policy():
    """Set event loop policy for tests."""
    return asyncio.get_event_loop_policy()


# ============================================================================
# Cleanup Fixtures
# ============================================================================

@pytest.fixture(autouse=True)
def reset_environment():
    """Reset environment after each test."""
    yield
    # Cleanup after test
    pass

