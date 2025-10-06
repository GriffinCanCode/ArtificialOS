"""
Model configuration with strong typing.
Centralized settings for Gemini API.
"""

from enum import Enum
import os

from pydantic import BaseModel, Field


class GeminiModel(str, Enum):
    """Available Gemini model variants."""

    FLASH_EXP = "gemini-2.0-flash-exp"  # Latest experimental Flash (recommended)
    FLASH = "gemini-1.5-flash"  # Stable Flash version
    PRO = "gemini-1.5-pro"  # Pro version (more capable, higher cost)
    FLASH_8B = "gemini-1.5-flash-8b"  # Ultra-fast, lowest cost


class GeminiConfig(BaseModel):
    """Type-safe Gemini API configuration."""

    # Model selection
    model_name: str = Field(default=GeminiModel.FLASH_EXP.value)
    api_key: str | None = Field(default=None)

    # Generation parameters
    temperature: float = Field(default=0.1, ge=0.0, le=2.0)
    max_tokens: int = Field(default=8192, ge=1, le=8192)
    top_p: float = Field(default=0.95, ge=0.0, le=1.0)
    top_k: int = Field(default=40, ge=1, le=100)

    # Streaming
    streaming: bool = Field(default=True)

    # JSON mode
    json_mode: bool = Field(default=False)
    response_schema: dict | None = Field(default=None)

    class Config:
        """Pydantic config."""

        frozen = True  # Immutable for thread safety
        use_enum_values = True

    def __init__(self, **data):
        """Initialize config with API key from environment if not provided."""
        if "api_key" not in data or data["api_key"] is None:
            data["api_key"] = os.getenv("GOOGLE_API_KEY")
        super().__init__(**data)

    @property
    def is_flash_model(self) -> bool:
        """Check if using a Flash model variant."""
        return "flash" in self.model_name.lower()

    @property
    def is_experimental(self) -> bool:
        """Check if using experimental model."""
        return "exp" in self.model_name.lower()

    def model_copy_with_updates(self, **updates) -> "GeminiConfig":
        """Create updated config (immutable pattern)."""
        data = self.model_dump()
        data.update(updates)
        return GeminiConfig(**data)


# Legacy ModelSize and ModelBackend kept for backwards compatibility
# (will be removed in future versions)
class ModelSize(str, Enum):
    """Deprecated: Use GeminiModel instead."""

    SMALL = "gemini-2.0-flash-exp"
    LARGE = "gemini-1.5-pro"


class ModelBackend(str, Enum):
    """Deprecated: All models now use Gemini API."""

    GEMINI = "gemini"
    TRANSFORMERS = "gemini"  # Redirect to Gemini
    LLAMA_CPP = "gemini"  # Redirect to Gemini


class ModelConfig(BaseModel):
    """
    Deprecated: Use GeminiConfig instead.
    Kept for backwards compatibility during migration.
    """

    backend: ModelBackend = Field(default=ModelBackend.GEMINI)
    size: ModelSize = Field(default=ModelSize.SMALL)
    temperature: float = Field(default=0.1, ge=0.0, le=2.0)
    max_tokens: int = Field(default=8192, ge=1, le=8192)
    streaming: bool = Field(default=True)

    class Config:
        frozen = True
        use_enum_values = True

    def to_gemini_config(self) -> GeminiConfig:
        """Convert legacy ModelConfig to GeminiConfig."""
        return GeminiConfig(
            model_name=self.size if isinstance(self.size, str) else self.size.value,
            temperature=self.temperature,
            max_tokens=self.max_tokens,
            streaming=self.streaming,
        )
