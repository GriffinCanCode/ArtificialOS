"""UI Specification Cache - lightweight wrapper around generic LRU cache."""

from core import LRUCache
from .models import Blueprint


class UICache:
    """
    Type-safe LRU cache for UI specifications.

    Wrapper around core.LRUCache with Blueprint type enforcement.
    """

    def __init__(self, max_size: int = 100, ttl_seconds: int = 3600) -> None:
        """
        Initialize UI cache.

        Args:
            max_size: Maximum cached specs
            ttl_seconds: Time-to-live in seconds
        """
        self._cache: LRUCache[Blueprint] = LRUCache(
            max_size=max_size,
            ttl_seconds=ttl_seconds
        )

    def get(self, key: str) -> Blueprint | None:
        """Get cached UI spec if valid."""
        return self._cache.get(key)

    def set(self, key: str, spec: Blueprint) -> None:
        """Cache UI spec."""
        self._cache.set(key, spec)

    def clear(self) -> None:
        """Clear cache."""
        self._cache.clear()

    @property
    def stats(self):
        """Get cache statistics."""
        return self._cache.stats


__all__ = ["UICache"]
