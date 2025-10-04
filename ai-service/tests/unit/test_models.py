"""Tests for model configuration and loading."""

import pytest
import os
from unittest.mock import MagicMock, patch

from src.models.config import (
    GeminiConfig, 
    GeminiModel, 
    ModelConfig, 
    ModelSize, 
    ModelBackend
)


# ============================================================================
# GeminiConfig Tests
# ============================================================================

@pytest.mark.unit
def test_gemini_config_defaults():
    """Test Gemini config defaults."""
    config = GeminiConfig(api_key="test-key")
    
    assert config.model_name == GeminiModel.FLASH_EXP.value
    assert config.temperature == 0.1
    assert config.max_tokens == 8192
    assert config.streaming is True


@pytest.mark.unit
def test_gemini_config_custom_values():
    """Test custom config values."""
    config = GeminiConfig(
        model_name=GeminiModel.PRO.value,
        api_key="test-key",
        temperature=0.7,
        max_tokens=2048,
        streaming=False
    )
    
    assert config.model_name == GeminiModel.PRO.value
    assert config.temperature == 0.7
    assert config.max_tokens == 2048
    assert config.streaming is False


@pytest.mark.unit
def test_gemini_config_api_key_from_env():
    """Test API key from environment."""
    with patch.dict(os.environ, {'GOOGLE_API_KEY': 'env-key'}):
        config = GeminiConfig()
        assert config.api_key == 'env-key'


@pytest.mark.unit
def test_gemini_config_validation_temperature():
    """Test temperature validation."""
    # Valid temperatures
    GeminiConfig(api_key="test", temperature=0.0)
    GeminiConfig(api_key="test", temperature=1.0)
    GeminiConfig(api_key="test", temperature=2.0)
    
    # Invalid temperatures
    with pytest.raises(Exception):
        GeminiConfig(api_key="test", temperature=-0.1)
    
    with pytest.raises(Exception):
        GeminiConfig(api_key="test", temperature=2.1)


@pytest.mark.unit
def test_gemini_config_validation_max_tokens():
    """Test max tokens validation."""
    # Valid
    GeminiConfig(api_key="test", max_tokens=1)
    GeminiConfig(api_key="test", max_tokens=8192)
    
    # Invalid
    with pytest.raises(Exception):
        GeminiConfig(api_key="test", max_tokens=0)
    
    with pytest.raises(Exception):
        GeminiConfig(api_key="test", max_tokens=10000)


@pytest.mark.unit
def test_gemini_config_is_flash_model():
    """Test is_flash_model property."""
    flash_config = GeminiConfig(
        api_key="test",
        model_name=GeminiModel.FLASH_EXP.value
    )
    pro_config = GeminiConfig(
        api_key="test",
        model_name=GeminiModel.PRO.value
    )
    
    assert flash_config.is_flash_model is True
    assert pro_config.is_flash_model is False


@pytest.mark.unit
def test_gemini_config_is_experimental():
    """Test is_experimental property."""
    exp_config = GeminiConfig(
        api_key="test",
        model_name=GeminiModel.FLASH_EXP.value
    )
    stable_config = GeminiConfig(
        api_key="test",
        model_name=GeminiModel.FLASH.value
    )
    
    assert exp_config.is_experimental is True
    assert stable_config.is_experimental is False


@pytest.mark.unit
def test_gemini_config_immutable():
    """Test config immutability."""
    config = GeminiConfig(api_key="test")
    
    # Should raise error on direct assignment
    with pytest.raises(Exception):
        config.temperature = 0.5


@pytest.mark.unit
def test_gemini_config_model_copy_with_updates():
    """Test creating updated config."""
    original = GeminiConfig(api_key="test", temperature=0.1)
    updated = original.model_copy_with_updates(temperature=0.7)
    
    assert original.temperature == 0.1  # Original unchanged
    assert updated.temperature == 0.7    # New config updated


@pytest.mark.unit
def test_gemini_config_json_mode():
    """Test JSON mode configuration."""
    config = GeminiConfig(
        api_key="test",
        json_mode=True,
        response_schema={"type": "object"}
    )
    
    assert config.json_mode is True
    assert config.response_schema is not None


# ============================================================================
# Legacy ModelConfig Tests
# ============================================================================

@pytest.mark.unit
def test_model_config_defaults():
    """Test legacy ModelConfig defaults."""
    config = ModelConfig()
    
    assert config.backend == ModelBackend.GEMINI
    assert config.size == ModelSize.SMALL
    assert config.temperature == 0.1
    assert config.streaming is True


@pytest.mark.unit
def test_model_config_to_gemini_config():
    """Test conversion from legacy to new config."""
    legacy = ModelConfig(
        size=ModelSize.LARGE,
        temperature=0.7,
        max_tokens=2048
    )
    
    gemini = legacy.to_gemini_config()
    
    assert isinstance(gemini, GeminiConfig)
    assert gemini.model_name == ModelSize.LARGE.value
    assert gemini.temperature == 0.7
    assert gemini.max_tokens == 2048


# ============================================================================
# GeminiModel Enum Tests
# ============================================================================

@pytest.mark.unit
def test_gemini_model_enum_values():
    """Test GeminiModel enum values."""
    assert GeminiModel.FLASH_EXP.value == "gemini-2.0-flash-exp"
    assert GeminiModel.FLASH.value == "gemini-1.5-flash"
    assert GeminiModel.PRO.value == "gemini-1.5-pro"
    assert GeminiModel.FLASH_8B.value == "gemini-1.5-flash-8b"


@pytest.mark.unit
def test_gemini_config_with_all_models():
    """Test config with all model variants."""
    for model in GeminiModel:
        config = GeminiConfig(api_key="test", model_name=model.value)
        assert config.model_name == model.value


# ============================================================================
# Model Loader Tests (mocked)
# ============================================================================

@pytest.mark.unit
def test_model_loader_singleton():
    """Test ModelLoader singleton pattern."""
    from src.models.loader import ModelLoader
    
    # Clear singleton
    ModelLoader._instance = None
    
    config = GeminiConfig(api_key="test-key")
    
    # Mock genai.configure to avoid actual API calls
    with patch('src.models.loader.genai.configure'):
        with patch('src.models.loader.genai.GenerativeModel'):
            model1 = ModelLoader.load(config)
            model2 = ModelLoader.load(config)
            
            # Should return same instance
            assert ModelLoader._instance is not None


@pytest.mark.unit
def test_model_loader_unload():
    """Test ModelLoader unload."""
    from src.models.loader import ModelLoader
    
    ModelLoader._instance = MagicMock()
    ModelLoader.unload()
    
    assert ModelLoader._instance is None


# ============================================================================
# Integration Tests (with mocked Gemini API)
# ============================================================================

@pytest.mark.integration
def test_gemini_model_initialization(gemini_config):
    """Test GeminiModel initialization."""
    with patch('src.models.loader.genai.configure') as mock_configure:
        with patch('src.models.loader.genai.GenerativeModel') as mock_model:
            from src.models.loader import GeminiModel
            
            model = GeminiModel(gemini_config)
            
            # Should configure API
            mock_configure.assert_called_once_with(api_key=gemini_config.api_key)
            
            # Should create model
            mock_model.assert_called_once()


@pytest.mark.integration
def test_gemini_model_stream(gemini_config):
    """Test GeminiModel streaming."""
    with patch('src.models.loader.genai.configure'):
        with patch('src.models.loader.genai.GenerativeModel') as mock_model_class:
            from src.models.loader import GeminiModel
            
            # Mock streaming response
            mock_chunk1 = MagicMock()
            mock_chunk1.text = "hello"
            mock_chunk2 = MagicMock()
            mock_chunk2.text = " world"
            
            mock_model_instance = MagicMock()
            mock_model_instance.generate_content.return_value = [mock_chunk1, mock_chunk2]
            mock_model_class.return_value = mock_model_instance
            
            model = GeminiModel(gemini_config)
            tokens = list(model.stream("test prompt"))
            
            assert tokens == ["hello", " world"]


@pytest.mark.integration
def test_gemini_model_invoke(gemini_config):
    """Test GeminiModel non-streaming."""
    with patch('src.models.loader.genai.configure'):
        with patch('src.models.loader.genai.GenerativeModel') as mock_model_class:
            from src.models.loader import GeminiModel
            
            # Mock response
            mock_response = MagicMock()
            mock_response.text = "response text"
            
            mock_model_instance = MagicMock()
            mock_model_instance.generate_content.return_value = mock_response
            mock_model_class.return_value = mock_model_instance
            
            model = GeminiModel(gemini_config)
            result = model.invoke("test prompt")
            
            assert result == "response text"

