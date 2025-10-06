"""Core utilities and infrastructure."""

from .config import Settings, get_settings
from .validate import (
    ValidationError,
    UIGenerationRequest,
    ChatRequest,
    BlueprintValidator,
    validate_json_size,
    validate_json_depth,
    validate_ui_spec,
)
from .logging_config import configure_logging, get_logger, LogContext
from .stream import TokenBatcher, StreamCounter, batch_tokens, batch_tokens_sync
from .json import (
    extract_json,
    safe_json_dumps,
    JSONParseError,
    validate_json_size,
    validate_json_depth,
)
from .hash import Algorithm, hash_string, hash_bytes, hash_fields
from .cache import LRUCache, Stats


def create_container(backend_url: str = "http://localhost:8000"):
    """Create dependency injection container (lazy import to avoid circular deps)."""
    from .container import create_container as _create_container

    return _create_container(backend_url)


__all__ = [
    # Config
    "Settings",
    "get_settings",
    # Validation
    "ValidationError",
    "UIGenerationRequest",
    "ChatRequest",
    "BlueprintValidator",
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
    # Hashing
    "Algorithm",
    "hash_string",
    "hash_bytes",
    "hash_fields",
    # Caching
    "LRUCache",
    "Stats",
]
