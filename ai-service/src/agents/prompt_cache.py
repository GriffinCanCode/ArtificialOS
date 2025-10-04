"""Prompt Cache - Optimized caching with strong typing."""

import hashlib
from typing import Dict, Optional, Any
from functools import lru_cache

from core import get_logger

logger = get_logger(__name__)


class PromptCache:
    """
    Cache for LLM prompt embeddings and common prompt patterns.
    Helps reduce context loading time for similar requests.
    """
    
    def __init__(self, max_size: int = 50):
        """
        Initialize prompt cache.
        
        Args:
            max_size: Maximum number of cached prompts
        """
        self.max_size = max_size
        self._cache: Dict[str, str] = {}
        self._hits = 0
        self._misses = 0
        logger.info("cache_init", max_size=max_size)
    
    def _compute_hash(self, prompt: str) -> str:
        """
        Compute hash for prompt.
        
        Args:
            prompt: Prompt text
            
        Returns:
            Hash string
        """
        return hashlib.sha256(prompt.encode()).hexdigest()[:16]
    
    def get(self, prompt: str) -> Optional[str]:
        """
        Get cached prompt if available.
        
        Args:
            prompt: Prompt text
            
        Returns:
            Cached prompt or None if not found
        """
        cache_key = self._compute_hash(prompt)
        
        if cache_key in self._cache:
            self._hits += 1
            logger.debug("cache_hit", hit_rate=f"{self.hit_rate:.1%}")
            return self._cache[cache_key]
        
        self._misses += 1
        logger.debug("cache_miss")
        return None
    
    def set(self, prompt: str) -> None:
        """
        Cache prompt.
        
        Args:
            prompt: Prompt text
        """
        cache_key = self._compute_hash(prompt)
        
        # Evict oldest if at capacity
        if len(self._cache) >= self.max_size:
            # Simple FIFO eviction
            first_key = next(iter(self._cache))
            del self._cache[first_key]
        
        self._cache[cache_key] = prompt
        logger.debug("cached", size=len(self._cache))
    
    @property
    def hit_rate(self) -> float:
        """Calculate cache hit rate."""
        total = self._hits + self._misses
        return self._hits / total if total > 0 else 0.0
    
    @property
    def stats(self) -> Dict[str, Any]:
        """Get cache statistics."""
        return {
            'size': len(self._cache),
            'max_size': self.max_size,
            'hits': self._hits,
            'misses': self._misses,
            'hit_rate': self.hit_rate,
        }


@lru_cache(maxsize=10)
def get_system_prompt_template(prompt_type: str = "ui_generation") -> str:
    """
    Get cached system prompt template.
    LRU cache ensures we don't rebuild common prompts repeatedly.
    
    Args:
        prompt_type: Type of prompt (ui_generation, chat, etc.)
        
    Returns:
        System prompt template string
    """
    # This would be populated with actual system prompts
    # The LRU cache ensures we only build these once
    logger.debug("build_template", type=prompt_type)
    return ""  # Placeholder


__all__ = ['PromptCache', 'get_system_prompt_template']

