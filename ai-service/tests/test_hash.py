"""Tests for hash module."""

import pytest
from hypothesis import given, strategies as st

from src.core.hash import (
    Algorithm,
    hash_string,
    hash_bytes,
    hash_fields,
    create_hasher,
)


def test_hash_string_xxhash():
    """Test xxhash string hashing."""
    result = hash_string("test", Algorithm.XXHASH64)
    assert isinstance(result, str)
    assert len(result) == 16  # xxhash64 produces 16 hex chars
    
    # Same input = same hash
    assert hash_string("test", Algorithm.XXHASH64) == result


def test_hash_string_sha256():
    """Test SHA256 string hashing."""
    result = hash_string("test", Algorithm.SHA256)
    assert isinstance(result, str)
    assert len(result) == 64  # SHA256 produces 64 hex chars
    
    # Same input = same hash
    assert hash_string("test", Algorithm.SHA256) == result


def test_hash_string_truncate():
    """Test hash truncation."""
    full = hash_string("test", Algorithm.SHA256)
    truncated = hash_string("test", Algorithm.SHA256, truncate=16)
    
    assert len(truncated) == 16
    assert full.startswith(truncated)


def test_hash_bytes():
    """Test byte hashing."""
    data = b"test data"
    result = hash_bytes(data, Algorithm.XXHASH64)
    
    assert isinstance(result, str)
    assert len(result) == 16


def test_hash_fields():
    """Test multi-field hashing."""
    result = hash_fields("field1", "field2", "field3")
    
    assert isinstance(result, str)
    
    # Order matters
    result2 = hash_fields("field3", "field2", "field1")
    assert result != result2
    
    # Deterministic
    result3 = hash_fields("field1", "field2", "field3")
    assert result == result3


def test_create_hasher_xxhash():
    """Test hasher creation."""
    hasher = create_hasher(Algorithm.XXHASH64)
    digest = hasher.digest(b"test")
    
    assert isinstance(digest, str)
    assert len(digest) == 16


def test_create_hasher_sha256():
    """Test SHA256 hasher creation."""
    hasher = create_hasher(Algorithm.SHA256)
    digest = hasher.digest(b"test")
    
    assert isinstance(digest, str)
    assert len(digest) == 64


def test_create_hasher_invalid():
    """Test invalid algorithm."""
    with pytest.raises(ValueError):
        create_hasher("invalid")  # type: ignore


@given(st.text(min_size=1, max_size=1000))
def test_hash_deterministic(text):
    """Property test: hashing is deterministic."""
    hash1 = hash_string(text, Algorithm.XXHASH64)
    hash2 = hash_string(text, Algorithm.XXHASH64)
    assert hash1 == hash2


@given(st.text(min_size=1), st.text(min_size=1))
def test_hash_unique(text1, text2):
    """Property test: different inputs produce different hashes."""
    if text1 != text2:
        hash1 = hash_string(text1, Algorithm.XXHASH64)
        hash2 = hash_string(text2, Algorithm.XXHASH64)
        # Collision is possible but extremely rare
        # This is a probabilistic test
        assert hash1 != hash2 or len(text1) > 100

