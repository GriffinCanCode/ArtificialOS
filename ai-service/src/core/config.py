"""Configuration Management."""

import os
from functools import lru_cache
from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()


class Settings(BaseSettings):
    """Application settings from environment."""

    model_config = SettingsConfigDict(
        env_prefix="AI_",
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",  # Ignore extra environment variables
    )

    # Server
    grpc_port: int = Field(default=50052, description="gRPC server port")
    grpc_workers: int = Field(default=10, description="gRPC thread pool size")

    # Backend
    backend_url: str = Field(default="http://localhost:8000", description="Backend service URL")
    backend_timeout: float = Field(default=5.0, description="Backend request timeout")

    # Model
    gemini_api_key: str = Field(
        default_factory=lambda: os.getenv("GEMINI_API_KEY", ""), description="Gemini API key"
    )
    gemini_model: str = Field(default="gemini-2.0-flash-exp", description="Gemini model name")
    gemini_temperature: float = Field(default=0.1, ge=0.0, le=2.0, description="Model temperature")
    gemini_max_tokens: int = Field(default=4096, gt=0, description="Max output tokens")

    # Logging
    log_level: str = Field(default="INFO", description="Log level")
    json_logs: bool = Field(default=False, description="Use JSON log format")

    # Caching
    enable_cache: bool = Field(default=True, description="Enable UI spec caching")
    cache_size: int = Field(default=100, gt=0, description="Cache max size")
    cache_ttl: int = Field(default=3600, gt=0, description="Cache TTL (seconds)")

    # Streaming
    stream_batch_size: int = Field(default=20, gt=0, description="Token batch size for streaming")

    # Validation
    max_message_length: int = Field(default=10_000, gt=0, description="Max message length")
    max_history_length: int = Field(default=50, gt=0, description="Max chat history length")


@lru_cache
def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()
