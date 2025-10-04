"""Tests for cache module."""

import time
import pytest
from hypothesis import given, strategies as st

from src.core.cache import LRUCache, Stats


def test_lru_basic():
    """Test basic cache operations."""
    cache = LRUCache[str](max_size=3)
    
    cache.set("a", "value_a")
    cache.set("b", "value_b")
    cache.set("c", "value_c")
    
    assert cache.get("a") == "value_a"
    assert cache.get("b") == "value_b"
    assert cache.get("c") == "value_c"
    assert len(cache) == 3


def test_lru_eviction():
    """Test LRU eviction on size limit."""
    cache = LRUCache[str](max_size=2)
    
    cache.set("a", "value_a")
    cache.set("b", "value_b")
    cache.set("c", "value_c")  # Should evict "a"
    
    assert cache.get("a") is None
    assert cache.get("b") == "value_b"
    assert cache.get("c") == "value_c"
    assert len(cache) == 2


def test_lru_order():
    """Test LRU ordering (most recently used stays)."""
    cache = LRUCache[str](max_size=2)
    
    cache.set("a", "value_a")
    cache.set("b", "value_b")
    
    # Access "a" to make it most recent
    _ = cache.get("a")
    
    # Add "c" - should evict "b" (least recent)
    cache.set("c", "value_c")
    
    assert cache.get("a") == "value_a"
    assert cache.get("b") is None
    assert cache.get("c") == "value_c"


def test_lru_ttl():
    """Test TTL expiration."""
    cache = LRUCache[str](max_size=10, ttl_seconds=1)
    
    cache.set("key", "value")
    assert cache.get("key") == "value"
    
    # Wait for expiration
    time.sleep(1.1)
    
    assert cache.get("key") is None


def test_lru_update():
    """Test updating existing entry."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("key", "value1")
    cache.set("key", "value2")
    
    assert cache.get("key") == "value2"
    assert len(cache) == 1


def test_lru_delete():
    """Test deletion."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("key", "value")
    assert cache.delete("key") is True
    assert cache.get("key") is None
    assert cache.delete("key") is False  # Already deleted


def test_lru_clear():
    """Test clearing cache."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("a", "value_a")
    cache.set("b", "value_b")
    
    cache.clear()
    
    assert len(cache) == 0
    assert cache.get("a") is None


def test_lru_contains():
    """Test __contains__."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("key", "value")
    
    assert "key" in cache
    assert "missing" not in cache


def test_stats_hit_miss():
    """Test statistics tracking."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("key", "value")
    
    _ = cache.get("key")  # Hit
    _ = cache.get("missing")  # Miss
    
    stats = cache.stats
    assert stats.hits == 1
    assert stats.misses == 1
    assert stats.hit_rate == 0.5


def test_stats_eviction():
    """Test eviction tracking."""
    cache = LRUCache[str](max_size=2)
    
    cache.set("a", "value_a")
    cache.set("b", "value_b")
    cache.set("c", "value_c")  # Eviction
    
    assert cache.stats.evictions == 1


def test_stats_to_dict():
    """Test stats export."""
    cache = LRUCache[str](max_size=10)
    
    cache.set("key", "value")
    _ = cache.get("key")
    
    stats_dict = cache.stats.to_dict()
    
    assert isinstance(stats_dict, dict)
    assert "hits" in stats_dict
    assert "misses" in stats_dict
    assert "hit_rate" in stats_dict


def test_invalid_max_size():
    """Test validation."""
    with pytest.raises(ValueError):
        LRUCache[str](max_size=0)
    
    with pytest.raises(ValueError):
        LRUCache[str](max_size=-1)


@given(st.lists(st.text(min_size=1, max_size=10), min_size=1, max_size=50))
def test_cache_preserves_values(keys):
    """Property test: cache preserves values correctly."""
    cache = LRUCache[str](max_size=100)
    
    # Set all values
    for key in keys:
        cache.set(key, f"value_{key}")
    
    # Verify all values (that fit in cache)
    recent_keys = keys[-100:]  # Only last 100 fit
    for key in recent_keys:
        assert cache.get(key) == f"value_{key}"


def test_type_safety():
    """Test generic type enforcement (static typing)."""
    # This is a compile-time test, but we can check runtime behavior
    cache_int = LRUCache[int](max_size=10)
    cache_int.set("key", 42)
    
    result = cache_int.get("key")
    assert isinstance(result, int)
    assert result == 42

