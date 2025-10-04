"""UI Specification Cache."""

import time
from typing import Optional, Dict
from collections import OrderedDict

from .models import UISpec


class UICache:
    """LRU cache for UI specifications."""
    
    def __init__(self, max_size: int = 100, ttl_seconds: int = 3600):
        self.max_size = max_size
        self.ttl_seconds = ttl_seconds
        self.cache: OrderedDict[str, tuple[UISpec, float]] = OrderedDict()
    
    def get(self, key: str) -> Optional[UISpec]:
        """Get cached UI spec if valid."""
        if key in self.cache:
            spec, timestamp = self.cache[key]
            if time.time() - timestamp < self.ttl_seconds:
                # Move to end (LRU)
                self.cache.move_to_end(key)
                return spec
            else:
                # Expired
                del self.cache[key]
        return None
    
    def set(self, key: str, spec: UISpec) -> None:
        """Cache UI spec."""
        if key in self.cache:
            del self.cache[key]
        self.cache[key] = (spec, time.time())
        
        # Enforce size limit
        if len(self.cache) > self.max_size:
            self.cache.popitem(last=False)
    
    def clear(self) -> None:
        """Clear cache."""
        self.cache.clear()
