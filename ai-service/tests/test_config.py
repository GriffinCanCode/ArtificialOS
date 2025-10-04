"""Configuration tests."""

import pytest
from src.core import get_settings


def test_settings_defaults():
    """Test default settings load correctly."""
    settings = get_settings()
    
    assert settings.grpc_port == 50052
    assert settings.grpc_workers == 10
    assert settings.backend_url == "http://localhost:8000"
    assert settings.log_level == "INFO"
    assert settings.enable_cache is True


def test_settings_validation():
    """Test settings validation."""
    from src.core.config import Settings
    
    # Valid settings
    settings = Settings(gemini_temperature=0.5)
    assert settings.gemini_temperature == 0.5
    
    # Invalid temperature (too high)
    with pytest.raises(Exception):
        Settings(gemini_temperature=3.0)
    
    # Invalid temperature (negative)
    with pytest.raises(Exception):
        Settings(gemini_temperature=-0.1)

