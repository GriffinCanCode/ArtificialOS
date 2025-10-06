"""Fast, type-safe JSON parsing with multiple backends."""

from typing import Any, TypeVar
import json
import sys

try:
    import msgspec

    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False

try:
    import orjson

    HAS_ORJSON = True
except ImportError:
    HAS_ORJSON = False

try:
    from json_repair import repair_json

    HAS_JSON_REPAIR = True
except ImportError:
    HAS_JSON_REPAIR = False

from returns.result import Result, Success, Failure

T = TypeVar("T")


class JSONParseError(Exception):
    """JSON parsing failed."""

    def __init__(self, message: str, original: Exception | None = None) -> None:
        super().__init__(message)
        self.original = original


class ParseError(JSONParseError):
    """Alias for backwards compatibility."""

    pass


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
    Extract and parse JSON from text (original function for backwards compatibility).

    Args:
        text: Text containing JSON
        repair: Attempt to repair invalid JSON

    Returns:
        Parsed JSON dictionary

    Raises:
        JSONParseError: If parsing fails
    """
    result = parse(text, strict=not repair)
    if isinstance(result, Success):
        return result.unwrap()
    else:
        raise result.failure()


def parse(
    text: str, *, strict: bool = True, repair: bool = False
) -> Result[dict[str, Any], ParseError]:
    """
    Parse JSON from text with automatic extraction and multiple fallbacks.

    Args:
        text: Text containing JSON
        strict: Whether to enforce strict JSON (no trailing commas, etc.)
        repair: Attempt to repair invalid JSON with json_repair

    Returns:
        Result containing parsed dict or error
    """
    text = text.strip()

    # Extract JSON boundaries
    boundaries = extract_json_boundaries(text)
    if boundaries is None:
        return Failure(ParseError("No JSON object found in text"))

    extracted_text, start, end = boundaries
    json_str = extracted_text[start:end]

    # Try msgspec first (fastest)
    if HAS_MSGSPEC and strict:
        try:
            decoder = msgspec.json.Decoder()
            result = decoder.decode(json_str.encode("utf-8"))
            if not isinstance(result, dict):
                return Failure(ParseError(f"Expected dict, got {type(result).__name__}"))
            return Success(result)
        except msgspec.DecodeError as e:
            if strict and not repair:
                return Failure(ParseError(f"Invalid JSON: {e}", e))
            # Fall through to next parser

    # Try standard library
    try:
        result = json.loads(json_str)
        if not isinstance(result, dict):
            return Failure(ParseError(f"Expected dict, got {type(result).__name__}"))
        return Success(result)
    except json.JSONDecodeError as e:
        if not repair or not HAS_JSON_REPAIR:
            return Failure(ParseError(f"JSON decode failed: {e}", e))

        # Last resort: try json_repair
        try:
            repaired = repair_json(json_str)
            result = json.loads(repaired)
            if not isinstance(result, dict):
                return Failure(ParseError(f"Expected dict, got {type(result).__name__}"))
            return Success(result)
        except Exception as repair_error:
            return Failure(ParseError(f"JSON repair failed: {repair_error}", repair_error))


def safe_json_dumps(obj: Any, **kwargs: Any) -> str:
    """
    Safely serialize object to JSON (original function for backwards compatibility).

    Args:
        obj: Object to serialize
        **kwargs: Additional arguments (indent, etc.)

    Returns:
        JSON string
    """
    return encode(obj, indent=kwargs.get("indent", 0))


def encode(obj: Any, *, indent: int = 0) -> str:
    """
    Encode object to JSON string using fastest available library.

    Args:
        obj: Object to encode
        indent: Indentation level (0 = compact, 2+ = pretty)

    Returns:
        JSON string
    """
    # Use orjson for compact output (fastest)
    if HAS_ORJSON and indent == 0:
        try:
            return orjson.dumps(obj).decode("utf-8")
        except (TypeError, ValueError):
            # Fallback for edge cases (e.g., integers outside 64-bit range)
            pass

    # Use msgspec for compact output (very fast)
    if HAS_MSGSPEC and indent == 0:
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
    Validate JSON size (original function for backwards compatibility).

    Args:
        data: JSON string to validate
        max_size: Maximum allowed size in bytes
        name: Name for error messages

    Raises:
        JSONParseError: If size exceeds limit
    """
    result = validate_size(data, max_size)
    if isinstance(result, Failure):
        raise JSONParseError(f"{name} size exceeds limit")


def validate_size(data: str, max_bytes: int) -> Result[None, ParseError]:
    """
    Validate JSON size to prevent DoS.

    Args:
        data: JSON string
        max_bytes: Maximum size in bytes

    Returns:
        Result indicating success or failure
    """
    size = sys.getsizeof(data)
    if size > max_bytes:
        return Failure(ParseError(f"Size {size} bytes exceeds maximum {max_bytes} bytes"))
    return Success(None)


def validate_json_depth(obj: Any, max_depth: int = 20, current_depth: int = 0) -> None:
    """
    Validate JSON nesting depth (original function for backwards compatibility).

    Args:
        obj: Object to validate
        max_depth: Maximum allowed nesting depth
        current_depth: Current depth (internal)

    Raises:
        JSONParseError: If depth exceeds limit
    """
    result = validate_depth(obj, max_depth, current_depth)
    if isinstance(result, Failure):
        raise JSONParseError(f"JSON nesting depth exceeds maximum {max_depth}")


def validate_depth(obj: Any, max_depth: int = 20, _depth: int = 0) -> Result[None, ParseError]:
    """
    Validate JSON nesting depth to prevent stack overflow.

    Args:
        obj: Object to validate
        max_depth: Maximum nesting depth
        _depth: Internal depth counter

    Returns:
        Result indicating success or failure
    """
    if _depth > max_depth:
        return Failure(ParseError(f"Nesting depth {_depth} exceeds maximum {max_depth}"))

    if isinstance(obj, dict):
        for value in obj.values():
            result = validate_depth(value, max_depth, _depth + 1)
            if isinstance(result, Failure):
                return result
    elif isinstance(obj, list):
        for item in obj:
            result = validate_depth(item, max_depth, _depth + 1)
            if isinstance(result, Failure):
                return result

    return Success(None)
