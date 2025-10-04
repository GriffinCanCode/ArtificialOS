"""Core utilities and infrastructure."""

from .config import Settings, get_settings
from .validate import (
    ValidationError,
    UIGenerationRequest,
    ChatRequest,
    UISpecValidator,
    validate_json_size,
    validate_json_depth,
    validate_ui_spec
)
from .logging_config import configure_logging, get_logger, LogContext
from .stream import TokenBatcher, StreamCounter, batch_tokens, batch_tokens_sync
from .json import (
    extract_json,
    safe_json_dumps,
    JSONParseError,
    validate_json_size,
    validate_json_depth
)
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
    "validate_ui_spec",
    # Logging
    "configure_logging",
    "get_logger",
    "LogContext",
    # Streaming
    "TokenBatcher",
    "StreamCounter",
    "batch_tokens",
    "batch_tokens_sync",
    # JSON
    "extract_json",
    "safe_json_dumps",
    "JSONParseError",
    "validate_json_size",
    "validate_json_depth",
    # DI
    "create_container",
]

