"""Pytest configuration and fixtures."""

import pytest
from unittest.mock import MagicMock

from src.core import create_container
from src.models.config import GeminiConfig


@pytest.fixture
def mock_gemini_model():
    """Mock Gemini model for testing."""
    mock = MagicMock()
    mock.stream.return_value = iter(["test", " ", "response"])
    mock.invoke.return_value = "test response"
    return mock


@pytest.fixture
def settings():
    """Test settings."""
    from src.core import get_settings
    return get_settings()


@pytest.fixture
def di_container():
    """Dependency injection container for testing."""
    return create_container()

