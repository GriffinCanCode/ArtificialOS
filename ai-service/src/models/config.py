"""
Model configuration with strong typing.
Centralized settings for LLM inference.
"""

from enum import Enum
from pathlib import Path
from typing import Optional

from pydantic import BaseModel, Field, field_validator


class ModelSize(str, Enum):
    """Available GPT-OSS model sizes."""
    
    SMALL = "gpt-oss-20b"
    LARGE = "gpt-oss-120b"


class ModelBackend(str, Enum):
    """Model inference backend."""
    
    LLAMA_CPP = "llama_cpp"  # Direct GGUF loading
    OLLAMA = "ollama"  # Ollama server


class ModelConfig(BaseModel):
    """Type-safe model configuration."""
    
    # Model selection
    backend: ModelBackend = Field(default=ModelBackend.OLLAMA)
    size: ModelSize = Field(default=ModelSize.SMALL)
    model_path: Optional[Path] = Field(default=None)
    ollama_base_url: str = Field(default="http://localhost:11434")
    
    # Context and generation
    context_length: int = Field(default=8192, ge=512, le=128000)
    max_tokens: int = Field(default=4096, ge=1, le=8192)
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    top_p: float = Field(default=0.95, ge=0.0, le=1.0)
    
    # Performance
    batch_size: int = Field(default=1024, ge=1, le=2048)  # Increased for better throughput
    gpu_layers: int = Field(default=-1, ge=-1, le=200)  # -1 = all layers on M4 Max GPU
    threads: Optional[int] = Field(default=None)
    
    # Streaming
    streaming: bool = Field(default=True)
    stream_chunk_size: int = Field(default=1)
    
    # Caching - DISABLED by default to prevent context pollution
    cache_prompt: bool = Field(default=False)
    seed: Optional[int] = Field(default=None)
    
    # Ollama-specific parameters
    keep_alive: str = Field(default="0")  # Don't keep model in memory between requests
    num_ctx: Optional[int] = Field(default=None)  # Context window size
    
    class Config:
        """Pydantic config."""
        
        frozen = True  # Immutable for thread safety
        use_enum_values = True
    
    @field_validator("model_path")
    @classmethod
    def validate_path(cls, v: Optional[Path]) -> Optional[Path]:
        """Ensure model path exists if provided."""
        if v is not None:
            v = Path(v).expanduser().resolve()
            if not v.exists():
                raise ValueError(f"Model path does not exist: {v}")
        return v
    
    @property
    def requires_gpu(self) -> bool:
        """Check if GPU acceleration is configured."""
        return self.gpu_layers > 0
    
    @property
    def model_name(self) -> str:
        """Get human-readable model name."""
        return self.size if isinstance(self.size, str) else self.size.value
    
    def model_copy_with_updates(self, **updates) -> "ModelConfig":
        """Create updated config (immutable pattern)."""
        data = self.model_dump()
        data.update(updates)
        return ModelConfig(**data)

