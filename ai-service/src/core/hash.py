"""Fast hashing for non-cryptographic use cases.

Provides xxhash for high-performance cache keys and SHA256 for compatibility.
All functions are strongly typed and designed for testability.
"""

from typing import Any, Protocol
from enum import Enum
import hashlib

try:
    import xxhash
    HAS_XXHASH = True
except ImportError:
    HAS_XXHASH = False


class Algorithm(str, Enum):
    """Supported hash algorithms."""
    
    XXHASH64 = "xxhash64"  # Fast, non-cryptographic (cache keys)
    SHA256 = "sha256"      # Secure, compatible with Go backend


class Hasher(Protocol):
    """Protocol for hash implementations."""
    
    def digest(self, data: bytes) -> str:
        """Compute hex digest of data."""
        ...


class XXHasher:
    """Ultra-fast non-cryptographic hasher."""
    
    def digest(self, data: bytes) -> str:
        """Compute xxhash64 hex digest."""
        return xxhash.xxh64(data).hexdigest()


class SHA256Hasher:
    """Secure cryptographic hasher."""
    
    def digest(self, data: bytes) -> str:
        """Compute SHA256 hex digest."""
        return hashlib.sha256(data).hexdigest()


def create_hasher(algorithm: Algorithm = Algorithm.XXHASH64) -> Hasher:
    """
    Create hasher instance.
    
    Args:
        algorithm: Hash algorithm to use
        
    Returns:
        Hasher instance
        
    Raises:
        ValueError: If xxhash not available when requested
    """
    if algorithm == Algorithm.XXHASH64:
        if not HAS_XXHASH:
            raise ValueError("xxhash not installed, use SHA256 or install xxhash")
        return XXHasher()
    elif algorithm == Algorithm.SHA256:
        return SHA256Hasher()
    else:
        raise ValueError(f"Unknown algorithm: {algorithm}")


def hash_string(
    text: str,
    algorithm: Algorithm = Algorithm.XXHASH64,
    truncate: int | None = None
) -> str:
    """
    Hash string to hex digest.
    
    Args:
        text: String to hash
        algorithm: Hash algorithm (default: xxhash64 for speed)
        truncate: Optional length to truncate digest (e.g., 16 for cache keys)
        
    Returns:
        Hex digest string
        
    Examples:
        >>> hash_string("test")  # Fast cache key
        'a4f6c9...'
        >>> hash_string("test", Algorithm.SHA256, truncate=16)  # Compatible
        'a4f6c9e3f4c3b2a1'
    """
    hasher = create_hasher(algorithm)
    digest = hasher.digest(text.encode("utf-8"))
    
    if truncate:
        return digest[:truncate]
    return digest


def hash_bytes(
    data: bytes,
    algorithm: Algorithm = Algorithm.XXHASH64,
    truncate: int | None = None
) -> str:
    """
    Hash bytes to hex digest.
    
    Args:
        data: Bytes to hash
        algorithm: Hash algorithm
        truncate: Optional length to truncate digest
        
    Returns:
        Hex digest string
    """
    hasher = create_hasher(algorithm)
    digest = hasher.digest(data)
    
    if truncate:
        return digest[:truncate]
    return digest


def hash_fields(*fields: str, algorithm: Algorithm = Algorithm.XXHASH64) -> str:
    """
    Hash multiple fields together (deterministic).
    
    Args:
        *fields: Fields to combine and hash
        algorithm: Hash algorithm
        
    Returns:
        Hex digest of combined fields
        
    Examples:
        >>> hash_fields("app", "user-123", "v1.0")
        'b4f3c2...'
    """
    combined = "\x00".join(fields)  # Null byte separator
    return hash_string(combined, algorithm)


__all__ = [
    "Algorithm",
    "Hasher",
    "create_hasher",
    "hash_string",
    "hash_bytes",
    "hash_fields",
]

