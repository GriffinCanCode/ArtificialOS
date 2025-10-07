"""Input validation with strong typing and multiple backends."""

from typing import Any

from pydantic import BaseModel, Field, field_validator, ConfigDict


# Validation limits
MAX_UI_SPEC_SIZE = 512 * 1024  # 512KB
MAX_CONTEXT_SIZE = 64 * 1024  # 64KB
MAX_JSON_DEPTH = 20
MAX_MESSAGE_LENGTH = 10_000
MAX_HISTORY_LENGTH = 50


class ValidationError(Exception):
    """Validation failed."""

    pass


class RequestValidator(BaseModel):
    """Base validator with strict configuration."""

    model_config = ConfigDict(
        strict=True, validate_assignment=True, extra="forbid", frozen=True  # Immutable by default
    )


class UIGenerationRequest(RequestValidator):
    """Validated UI generation request."""

    message: str = Field(min_length=1, max_length=MAX_MESSAGE_LENGTH)
    context: dict[str, Any] = Field(default_factory=dict)

    @field_validator("message")
    @classmethod
    def validate_message(cls, v: str) -> str:
        """Ensure message is non-empty after stripping."""
        stripped = v.strip()
        if not stripped:
            raise ValueError("Message cannot be empty")
        return stripped


class ChatRequest(RequestValidator):
    """Validated chat request."""

    message: str = Field(min_length=1, max_length=MAX_MESSAGE_LENGTH)
    history_count: int = Field(default=0, ge=0, le=MAX_HISTORY_LENGTH)

    @field_validator("message")
    @classmethod
    def validate_message(cls, v: str) -> str:
        """Ensure message is non-empty after stripping."""
        stripped = v.strip()
        if not stripped:
            raise ValueError("Message cannot be empty")
        return stripped


def validate_json_size(data: str, max_size: int, name: str = "JSON") -> None:
    """
    Validate JSON size to prevent DoS attacks.

    Args:
        data: JSON string to validate
        max_size: Maximum allowed size in bytes
        name: Name for error messages

    Raises:
        ValidationError: If size exceeds limit
    """
    import sys

    size = sys.getsizeof(data)
    if size > max_size:
        raise ValidationError(f"{name} size {size} bytes exceeds maximum {max_size} bytes")


def validate_json_depth(obj: Any, max_depth: int = MAX_JSON_DEPTH, current_depth: int = 0) -> None:
    """
    Validate JSON nesting depth to prevent stack overflow.

    Args:
        obj: Object to validate
        max_depth: Maximum allowed nesting depth
        current_depth: Current depth (internal)

    Raises:
        ValidationError: If depth exceeds limit
    """
    if current_depth > max_depth:
        raise ValidationError(f"JSON nesting depth {current_depth} exceeds maximum {max_depth}")

    if isinstance(obj, dict):
        for value in obj.values():
            validate_json_depth(value, max_depth, current_depth + 1)
    elif isinstance(obj, list):
        for item in obj:
            validate_json_depth(item, max_depth, current_depth + 1)


class BlueprintValidator:
    """Validates generated UI specifications."""

    @staticmethod
    def validate(spec_dict: dict[str, Any], spec_json: str) -> None:
        """
        Validate UI spec comprehensively.

        Args:
            spec_dict: Parsed UI spec dictionary
            spec_json: JSON string representation

        Raises:
            ValidationError: If validation fails
        """
        # Size validation
        validate_json_size(spec_json, MAX_UI_SPEC_SIZE, "UI spec")

        # Depth validation
        validate_json_depth(spec_dict)

        # Structure validation
        if "title" not in spec_dict:
            raise ValidationError("UI spec missing required 'title' field")

        if "components" not in spec_dict:
            raise ValidationError("UI spec missing required 'components' field")

        if not isinstance(spec_dict["components"], list):
            raise ValidationError("UI spec 'components' must be a list")
