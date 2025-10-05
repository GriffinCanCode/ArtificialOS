"""Prompt Cache - lightweight wrapper around generic LRU cache."""

from functools import lru_cache

from core import LRUCache, get_logger

logger = get_logger(__name__)


class PromptCache:
    """
    Cache for LLM prompts using fast xxhash.

    Wrapper around core.LRUCache optimized for prompt caching.
    """

    def __init__(self, max_size: int = 50) -> None:
        """
        Initialize prompt cache.

        Args:
            max_size: Maximum number of cached prompts
        """
        self._cache: LRUCache[str] = LRUCache(
            max_size=max_size,
            ttl_seconds=None  # Prompts don't expire
        )
        logger.info("cache_init", max_size=max_size)

    def get(self, prompt: str) -> str | None:
        """
        Get cached prompt if available.

        Args:
            prompt: Prompt text

        Returns:
            Cached prompt or None if not found
        """
        result = self._cache.get(prompt)

        if result:
            logger.debug("cache_hit", hit_rate=f"{self.hit_rate:.1%}")
        else:
            logger.debug("cache_miss")

        return result

    def set(self, prompt: str) -> None:
        """
        Cache prompt.

        Args:
            prompt: Prompt text
        """
        self._cache.set(prompt, prompt)
        logger.debug("cached", size=len(self._cache))

    @property
    def hit_rate(self) -> float:
        """Calculate cache hit rate."""
        return self._cache.stats.hit_rate

    @property
    def stats(self):
        """Get cache statistics."""
        return self._cache.stats.to_dict()


@lru_cache(maxsize=10)
def get_system_prompt_template(prompt_type: str = "ui_generation") -> str:
    """
    Get cached system prompt template.

    Uses functools.lru_cache for compile-time templates.

    Args:
        prompt_type: Type of prompt (ui_generation, chat, etc.)

    Returns:
        System prompt template string
    """
    logger.debug("build_template", type=prompt_type)
    return ""  # Placeholder


__all__ = ["PromptCache", "get_system_prompt_template"]

