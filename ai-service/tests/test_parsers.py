"""Parser tests with property-based testing."""

import pytest
import json
from hypothesis import given, strategies as st

from src.core import extract_json, safe_json_dumps, JSONParseError


def test_extract_json_clean():
    """Test extracting clean JSON."""
    text = '{"title": "Test", "count": 42}'
    result = extract_json(text)
    assert result == {"title": "Test", "count": 42}


def test_extract_json_with_markdown():
    """Test extracting JSON from markdown code blocks."""
    text = '''Here's the JSON:
```json
{"title": "Test", "count": 42}
```
Done!'''
    result = extract_json(text)
    assert result == {"title": "Test", "count": 42}


def test_extract_json_with_extra_text():
    """Test extracting JSON with surrounding text."""
    text = 'Some text before {"title": "Test", "count": 42} and after'
    result = extract_json(text)
    assert result == {"title": "Test", "count": 42}


def test_extract_json_invalid():
    """Test error on invalid JSON."""
    text = "This has no JSON"
    with pytest.raises(JSONParseError):
        extract_json(text, repair=False)


def test_safe_json_dumps():
    """Test JSON serialization."""
    obj = {"title": "Test", "items": [1, 2, 3]}
    result = safe_json_dumps(obj)
    assert json.loads(result) == obj


def test_safe_json_dumps_with_indent():
    """Test JSON serialization with indentation."""
    obj = {"title": "Test"}
    result = safe_json_dumps(obj, indent=2)
    assert json.loads(result) == obj
    assert "\n" in result  # Indented output has newlines


@given(st.dictionaries(st.text(min_size=1), st.integers()))
def test_json_roundtrip(data):
    """Property test: JSON serialization roundtrip."""
    json_str = safe_json_dumps(data)
    result = json.loads(json_str)
    assert result == data

