"""
Models package - Gemini API integration.
Unified interface for AI model loading and configuration.
"""

from .config import GeminiConfig, GeminiModel as GeminiModelEnum, ModelConfig, ModelSize, ModelBackend
from .loader import ModelLoader, GeminiModel, ModelLoadError

__all__ = [
    # Primary exports (Gemini)
    "GeminiConfig",
    "GeminiModel",
    "ModelLoader",
    "ModelLoadError",
    
    # Legacy exports (for backwards compatibility)
    "ModelConfig",
    "ModelSize",
    "ModelBackend",
    "GeminiModelEnum",
]
