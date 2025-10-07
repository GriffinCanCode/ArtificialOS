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
            max_size=max_size, ttl_seconds=None  # Prompts don't expire
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

    if prompt_type == "ui_generation":
        from agents.prompt import get_ui_generation_prompt
        # Return base UI generation prompt without tools/context
        return get_ui_generation_prompt("", "")

    elif prompt_type == "chat":
        return """You are a helpful AI assistant.
Provide clear, accurate, and concise responses to user questions.
Be professional, friendly, and informative."""

    elif prompt_type == "code_generation":
        return """You are an expert code generation assistant.
Generate clean, efficient, and well-documented code based on user requirements.
Follow best practices and modern conventions for the specified language."""

    elif prompt_type == "blueprint_generation":
        from agents.prompt import BLUEPRINT_DOCUMENTATION
        return BLUEPRINT_DOCUMENTATION

    else:
        logger.warn("unknown_prompt_type", type=prompt_type)
        return ""


__all__ = ["PromptCache", "get_system_prompt_template"]
