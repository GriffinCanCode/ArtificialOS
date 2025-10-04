"""Core utilities and infrastructure."""

from .config import Settings, get_settings
from .validation import (
    ValidationError,
    UIGenerationRequest,
    ChatRequest,
    UISpecValidator,
    validate_json_size,
    validate_json_depth
)
from .logging_config import configure_logging, get_logger, LogContext
from .streaming import TokenBatcher, batch_tokens, batch_tokens_sync, StreamCounter
from .parsers import extract_json, safe_json_dumps, JSONParseError
from .container import create_container

__all__ = [
    # Config
    "Settings",
    "get_settings",
    # Validation
    "ValidationError",
    "UIGenerationRequest",
    "ChatRequest",
    "UISpecValidator",
    "validate_json_size",
    "validate_json_depth",
    # Logging
    "configure_logging",
    "get_logger",
    "LogContext",
    # Streaming
    "TokenBatcher",
    "batch_tokens",
    "batch_tokens_sync",
    "StreamCounter",
    # Parsing
    "extract_json",
    "safe_json_dumps",
    "JSONParseError",
    # DI
    "create_container",
]

