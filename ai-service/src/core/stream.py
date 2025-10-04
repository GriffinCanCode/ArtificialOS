"""Efficient token streaming with batching."""

from typing import AsyncIterator, Iterator, TypeVar
from collections.abc import AsyncGenerator, Generator
from dataclasses import dataclass, field

T = TypeVar('T')


@dataclass
class TokenBatcher:
    """Batches tokens for efficient streaming."""
    
    batch_size: int = 20
    _buffer: str = field(default="", init=False, repr=False)
    
    def add(self, token: str) -> str | None:
        """Add token to buffer, return batch if ready."""
        self._buffer += token
        if len(self._buffer) >= self.batch_size:
            batch_result = self._buffer
            self._buffer = ""
            return batch_result
        return None
    
    def flush(self) -> str | None:
        """Return remaining buffer contents."""
        if self._buffer:
            batch_result = self._buffer
            self._buffer = ""
            return batch_result
        return None


@dataclass
class StreamCounter:
    """Track token statistics."""
    
    count: int = 0
    chars: int = 0
    
    def track(self, token: str) -> None:
        """Record token."""
        self.count += 1
        self.chars += len(token)
    
    def reset(self) -> tuple[int, int]:
        """Reset and return counts."""
        result = (self.count, self.chars)
        self.count = 0
        self.chars = 0
        return result


async def batch_tokens(stream: AsyncIterator[str], batch_size: int = 20) -> AsyncGenerator[str, None]:
    """
    Batch tokens from async stream.
    
    Args:
        stream: Async token iterator
        batch_size: Tokens per batch
        
    Yields:
        Batched tokens
    """
    batcher = TokenBatcher(batch_size=batch_size)
    
    async for token in stream:
        if batch_result := batcher.add(token):
            yield batch_result
    
    # Flush remaining
    if final := batcher.flush():
        yield final


def batch_tokens_sync(stream: Iterator[str], batch_size: int = 20) -> Generator[str, None, None]:
    """
    Batch tokens from sync stream.
    
    Args:
        stream: Sync token iterator
        batch_size: Tokens per batch
        
    Yields:
        Batched tokens
    """
    batcher = TokenBatcher(batch_size=batch_size)
    
    for token in stream:
        if batch_result := batcher.add(token):
            yield batch_result
    
    # Flush remaining
    if final := batcher.flush():
        yield final

