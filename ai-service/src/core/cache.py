"""Generic LRU cache with TTL and statistics.

Type-safe, testable, and highly extensible cache implementation.
Supports both in-memory caching with automatic eviction and TTL expiration.
"""

import time
from typing import Generic, TypeVar, Any
from collections import OrderedDict
from dataclasses import dataclass, field

from .hash import hash_string, Algorithm

T = TypeVar("T")


@dataclass
class Stats:
    """Cache statistics."""

    size: int = 0
    max_size: int = 0
    hits: int = 0
    misses: int = 0
    evictions: int = 0

    @property
    def hit_rate(self) -> float:
        """Calculate hit rate (0.0 to 1.0)."""
        total = self.hits + self.misses
        return self.hits / total if total > 0 else 0.0

    def to_dict(self) -> dict[str, Any]:
        """Export as dictionary."""
        return {
            "size": self.size,
            "max_size": self.max_size,
            "hits": self.hits,
            "misses": self.misses,
            "evictions": self.evictions,
            "hit_rate": self.hit_rate,
        }


class LRUCache(Generic[T]):
    """
    LRU cache with TTL support and statistics tracking.

    Features:
    - Type-safe generic implementation
    - Optional TTL expiration
    - Automatic eviction on size limit
    - Hit/miss statistics
    - Fast O(1) operations

    Examples:
        >>> cache = LRUCache[str](max_size=100, ttl_seconds=3600)
        >>> cache.set("key", "value")
        >>> cache.get("key")
        'value'
        >>> cache.stats.hit_rate
        1.0
    """

    def __init__(
        self,
        max_size: int = 100,
        ttl_seconds: int | None = None,
        hash_algorithm: Algorithm = Algorithm.XXHASH64,
    ):
        """
        Initialize LRU cache.

        Args:
            max_size: Maximum number of entries
            ttl_seconds: Time-to-live in seconds (None = no expiration)
            hash_algorithm: Algorithm for computing cache keys
        """
        if max_size <= 0:
            raise ValueError("max_size must be positive")

        self.max_size = max_size
        self.ttl_seconds = ttl_seconds
        self.hash_algorithm = hash_algorithm

        self._cache: OrderedDict[str, tuple[T, float]] = OrderedDict()
        self._stats = Stats(max_size=max_size)

    def _compute_key(self, key: str) -> str:
        """Compute cache key from input string."""
        return hash_string(key, self.hash_algorithm, truncate=16)

    def _is_expired(self, timestamp: float) -> bool:
        """Check if timestamp is expired."""
        if self.ttl_seconds is None:
            return False
        return time.time() - timestamp >= self.ttl_seconds

    def get(self, key: str) -> T | None:
        """
        Get cached value if available and not expired.

        Args:
            key: Cache key

        Returns:
            Cached value or None if not found/expired
        """
        cache_key = self._compute_key(key)

        if cache_key in self._cache:
            value, timestamp = self._cache[cache_key]

            if self._is_expired(timestamp):
                # Expired - remove it
                del self._cache[cache_key]
                self._stats.size = len(self._cache)
                self._stats.misses += 1
                return None

            # Valid hit - move to end (most recently used)
            self._cache.move_to_end(cache_key)
            self._stats.hits += 1
            return value

        self._stats.misses += 1
        return None

    def set(self, key: str, value: T) -> None:
        """
        Cache value with current timestamp.

        Args:
            key: Cache key
            value: Value to cache
        """
        cache_key = self._compute_key(key)

        # Update existing entry
        if cache_key in self._cache:
            del self._cache[cache_key]

        # Add new entry
        self._cache[cache_key] = (value, time.time())

        # Enforce size limit (FIFO eviction)
        if len(self._cache) > self.max_size:
            self._cache.popitem(last=False)
            self._stats.evictions += 1

        self._stats.size = len(self._cache)

    def delete(self, key: str) -> bool:
        """
        Delete entry from cache.

        Args:
            key: Cache key

        Returns:
            True if deleted, False if not found
        """
        cache_key = self._compute_key(key)

        if cache_key in self._cache:
            del self._cache[cache_key]
            self._stats.size = len(self._cache)
            return True
        return False

    def clear(self) -> None:
        """Clear entire cache."""
        self._cache.clear()
        self._stats.size = 0

    @property
    def stats(self) -> Stats:
        """Get cache statistics."""
        return self._stats

    def __len__(self) -> int:
        """Return number of cached entries."""
        return len(self._cache)

    def __contains__(self, key: str) -> bool:
        """Check if key exists (doesn't update LRU order)."""
        cache_key = self._compute_key(key)
        return cache_key in self._cache


__all__ = ["LRUCache", "Stats"]
