"""JSON Parsing Utilities."""

import json
from typing import Any, Dict

try:
    from json_repair import repair_json
    HAS_JSON_REPAIR = True
except ImportError:
    HAS_JSON_REPAIR = False

try:
    import orjson
    HAS_ORJSON = True
except ImportError:
    HAS_ORJSON = False


class JSONParseError(Exception):
    """JSON parsing failed."""
    pass


def extract_json(text: str, repair: bool = True) -> Dict[str, Any]:
    """
    Extract and parse JSON from text with automatic repair.
    
    Args:
        text: Text containing JSON
        repair: Attempt to repair invalid JSON
        
    Returns:
        Parsed JSON dictionary
        
    Raises:
        JSONParseError: If parsing fails
    """
    text = text.strip()
    
    # Remove markdown code blocks
    if "```" in text:
        if "```json" in text:
            start = text.find("```json") + 7
        elif "```" in text:
            start = text.find("```") + 3
        else:
            start = 0
        
        end = text.find("```", start)
        if end != -1:
            text = text[start:end].strip()
    
    # Find JSON boundaries
    start = text.find("{")
    end = text.rfind("}")
    
    if start == -1 or end == -1:
        raise JSONParseError(f"No JSON braces found in text: {text[:200]}...")
    
    json_str = text[start:end+1]
    
    # Try direct parse first
    try:
        return json.loads(json_str)
    except json.JSONDecodeError as e:
        if not repair or not HAS_JSON_REPAIR:
            raise JSONParseError(f"JSON parse error: {e}") from e
        
        # Attempt repair with json-repair library
        try:
            repaired = repair_json(json_str)
            return json.loads(repaired)
        except Exception as repair_error:
            raise JSONParseError(
                f"JSON repair failed: {repair_error}"
            ) from repair_error


def safe_json_dumps(obj: Any, **kwargs: Any) -> str:
    """
    Safely serialize object to JSON with fast library if available.
    
    Args:
        obj: Object to serialize
        **kwargs: Additional arguments
        
    Returns:
        JSON string
    """
    if HAS_ORJSON:
        # orjson is 2-3x faster than standard json
        # Note: orjson doesn't support indent directly, we handle it
        indent = kwargs.get('indent')
        if indent:
            # For indented output, use standard json (for readability)
            return json.dumps(obj, **kwargs)
        else:
            # For compact output, use orjson (for speed)
            return orjson.dumps(obj).decode('utf-8')
    else:
        # Fallback to standard library
        return json.dumps(obj, **kwargs)
