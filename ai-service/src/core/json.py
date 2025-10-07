"""Fast, type-safe JSON parsing with multiple backends."""

from typing import Any, TypeVar
import json
import sys

import msgspec
import orjson
from json_repair import repair_json

T = TypeVar("T")


class JSONParseError(Exception):
    """JSON parsing failed."""

    def __init__(self, message: str, original: Exception | None = None) -> None:
        super().__init__(message)
        self.original = original


def extract_json_boundaries(text: str) -> tuple[str, int, int] | None:
    """
    Extract JSON string and boundaries from text.

    Args:
        text: Text potentially containing JSON

    Returns:
        (extracted_text, start, end) or None if not found
    """
    working_text = text

    # Remove markdown code blocks
    if "```" in working_text:
        if "```json" in working_text:
            start_marker = working_text.find("```json") + 7
        else:
            start_marker = working_text.find("```") + 3

        end_marker = working_text.find("```", start_marker)
        if end_marker != -1:
            working_text = working_text[start_marker:end_marker].strip()

    # Find JSON object boundaries
    start = working_text.find("{")
    end = working_text.rfind("}")

    if start == -1 or end == -1:
        return None

    return (working_text, start, end + 1)


def extract_json(text: str, repair: bool = True) -> dict[str, Any]:
    """
    Extract and parse JSON from text with automatic extraction and multiple fallbacks.

    Args:
        text: Text containing JSON
        repair: Attempt to repair invalid JSON with json_repair

    Returns:
        Parsed JSON dictionary

    Raises:
        JSONParseError: If parsing fails
    """
    text = text.strip()

    # Extract JSON boundaries
    boundaries = extract_json_boundaries(text)
    if boundaries is None:
        raise JSONParseError("No JSON object found in text")

    extracted_text, start, end = boundaries
    json_str = extracted_text[start:end]

    # Try msgspec first (fastest)
    try:
        decoder = msgspec.json.Decoder()
        result = decoder.decode(json_str.encode("utf-8"))
        if not isinstance(result, dict):
            raise JSONParseError(f"Expected dict, got {type(result).__name__}")
        return result
    except msgspec.DecodeError as e:
        if not repair:
            raise JSONParseError(f"Invalid JSON: {e}", e)
        # Fall through to repair

    # Try standard library
    try:
        result = json.loads(json_str)
        if not isinstance(result, dict):
            raise JSONParseError(f"Expected dict, got {type(result).__name__}")
        return result
    except json.JSONDecodeError as e:
        if not repair:
            raise JSONParseError(f"JSON decode failed: {e}", e)

        # Last resort: try json_repair
        try:
            repaired = repair_json(json_str)
            result = json.loads(repaired)
            if not isinstance(result, dict):
                raise JSONParseError(f"Expected dict, got {type(result).__name__}")
            return result
        except Exception as repair_error:
            raise JSONParseError(f"JSON repair failed: {repair_error}", repair_error)


def safe_json_dumps(obj: Any, **kwargs: Any) -> str:
    """
    Encode object to JSON string using fastest available library.

    Args:
        obj: Object to encode
        **kwargs: Additional arguments (indent, etc.)

    Returns:
        JSON string
    """
    indent = kwargs.get("indent", 0)

    # Use orjson for compact output (fastest)
    if indent == 0:
        try:
            return orjson.dumps(obj).decode("utf-8")
        except (TypeError, ValueError):
            # Fallback for edge cases (e.g., integers outside 64-bit range)
            pass

    # Use msgspec for compact output (very fast)
    if indent == 0:
        try:
            encoder = msgspec.json.Encoder()
            return encoder.encode(obj).decode("utf-8")
        except (TypeError, ValueError):
            # Fallback for edge cases
            pass

    # Use stdlib for pretty-printed output or as fallback (most compatible)
    return json.dumps(obj, indent=indent if indent > 0 else None)


def validate_json_size(data: str, max_size: int, name: str = "JSON") -> None:
    """
    Validate JSON size to prevent DoS attacks.

    Args:
        data: JSON string to validate
        max_size: Maximum allowed size in bytes
        name: Name for error messages

    Raises:
        JSONParseError: If size exceeds limit
    """
    size = sys.getsizeof(data)
    if size > max_size:
        raise JSONParseError(f"{name} size {size} bytes exceeds maximum {max_size} bytes")


def validate_json_depth(obj: Any, max_depth: int = 20, current_depth: int = 0) -> None:
    """
    Validate JSON nesting depth to prevent stack overflow.

    Args:
        obj: Object to validate
        max_depth: Maximum allowed nesting depth
        current_depth: Current depth (internal)

    Raises:
        JSONParseError: If depth exceeds limit
    """
    if current_depth > max_depth:
        raise JSONParseError(f"JSON nesting depth {current_depth} exceeds maximum {max_depth}")

    if isinstance(obj, dict):
        for value in obj.values():
            validate_json_depth(value, max_depth, current_depth + 1)
    elif isinstance(obj, list):
        for item in obj:
            validate_json_depth(item, max_depth, current_depth + 1)
