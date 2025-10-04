"""
UI Spec Caching System
Caches commonly generated UIs to reduce LLM inference time.
"""

import hashlib
import json
import logging
import time
from typing import Dict, Optional, Tuple
from functools import lru_cache

from .ui_generator import UISpec

logger = logging.getLogger(__name__)


class UICache:
    """
    LRU cache for UI specifications.
    Caches based on normalized request hash to avoid repeated LLM inference.
    """
    
    def __init__(self, max_size: int = 100, ttl_seconds: int = 3600):
        """
        Initialize UI cache.
        
        Args:
            max_size: Maximum number of cached UI specs
            ttl_seconds: Time-to-live for cache entries (default 1 hour)
        """
        self.max_size = max_size
        self.ttl_seconds = ttl_seconds
        self._cache: Dict[str, Tuple[UISpec, float]] = {}
        self._access_times: Dict[str, float] = {}
        self._hits = 0
        self._misses = 0
        logger.info(f"UICache initialized: max_size={max_size}, ttl={ttl_seconds}s")
    
    def _normalize_request(self, request: str) -> str:
        """
        Normalize request for better cache hit rate.
        
        Args:
            request: Raw user request
            
        Returns:
            Normalized request string
        """
        # Convert to lowercase
        normalized = request.lower().strip()
        
        # Remove extra whitespace
        normalized = ' '.join(normalized.split())
        
        # Common synonyms/variations (expand as needed)
        replacements = {
            'create': 'make',
            'build': 'make',
            'generate': 'make',
            'calculator': 'calc',
            'to-do': 'todo',
            'to do': 'todo',
        }
        
        for old, new in replacements.items():
            normalized = normalized.replace(old, new)
        
        return normalized
    
    def _compute_hash(self, request: str) -> str:
        """
        Compute hash for request.
        
        Args:
            request: User request
            
        Returns:
            Hash string
        """
        normalized = self._normalize_request(request)
        return hashlib.sha256(normalized.encode()).hexdigest()[:16]
    
    def get(self, request: str) -> Optional[UISpec]:
        """
        Get cached UI spec if available and not expired.
        
        Args:
            request: User request
            
        Returns:
            Cached UISpec or None if not found/expired
        """
        cache_key = self._compute_hash(request)
        
        if cache_key not in self._cache:
            self._misses += 1
            logger.debug(f"Cache miss for: {request[:50]}...")
            return None
        
        ui_spec, timestamp = self._cache[cache_key]
        
        # Check if expired
        if time.time() - timestamp > self.ttl_seconds:
            logger.debug(f"Cache expired for: {request[:50]}...")
            del self._cache[cache_key]
            del self._access_times[cache_key]
            self._misses += 1
            return None
        
        # Update access time
        self._access_times[cache_key] = time.time()
        self._hits += 1
        
        logger.info(f"Cache hit for: {request[:50]}... (hit rate: {self.hit_rate:.1%})")
        return ui_spec
    
    def set(self, request: str, ui_spec: UISpec) -> None:
        """
        Cache UI spec for request.
        
        Args:
            request: User request
            ui_spec: Generated UI specification
        """
        cache_key = self._compute_hash(request)
        
        # Evict oldest if at capacity
        if len(self._cache) >= self.max_size:
            self._evict_lru()
        
        # Store with current timestamp
        self._cache[cache_key] = (ui_spec, time.time())
        self._access_times[cache_key] = time.time()
        
        logger.info(f"Cached UI spec for: {request[:50]}... (cache size: {len(self._cache)})")
    
    def _evict_lru(self) -> None:
        """Evict least recently used entry."""
        if not self._access_times:
            return
        
        # Find LRU entry
        lru_key = min(self._access_times.items(), key=lambda x: x[1])[0]
        
        # Remove from cache
        del self._cache[lru_key]
        del self._access_times[lru_key]
        
        logger.debug(f"Evicted LRU entry: {lru_key}")
    
    def clear(self) -> None:
        """Clear all cached entries."""
        self._cache.clear()
        self._access_times.clear()
        logger.info("Cache cleared")
    
    @property
    def hit_rate(self) -> float:
        """Calculate cache hit rate."""
        total = self._hits + self._misses
        return self._hits / total if total > 0 else 0.0
    
    @property
    def stats(self) -> Dict[str, any]:
        """Get cache statistics."""
        return {
            'size': len(self._cache),
            'max_size': self.max_size,
            'hits': self._hits,
            'misses': self._misses,
            'hit_rate': self.hit_rate,
            'ttl_seconds': self.ttl_seconds,
        }


# Memoize common component templates for faster access
@lru_cache(maxsize=256)
def get_cached_component_template(component_type: str, variant: str = "default") -> str:
    """
    Get cached component template structure.
    Used to speed up common component generation.
    
    Args:
        component_type: Type of component (button, input, text, etc.)
        variant: Component variant
        
    Returns:
        JSON template string
    """
    templates = {
        'button': '{"type":"button","id":"","props":{"text":"","variant":"default","size":"medium"},"children":[],"on_event":null}',
        'input': '{"type":"input","id":"","props":{"placeholder":"","value":"","type":"text","readonly":false},"children":[],"on_event":null}',
        'text': '{"type":"text","id":"","props":{"content":"","variant":"body"},"children":[],"on_event":null}',
        'container': '{"type":"container","id":"","props":{"layout":"vertical","gap":8},"children":[],"on_event":null}',
        'grid': '{"type":"grid","id":"","props":{"columns":3,"gap":8},"children":[],"on_event":null}',
    }
    
    return templates.get(component_type, '{}')


__all__ = ['UICache', 'get_cached_component_template']

