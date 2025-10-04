"""
Streaming Utilities
Efficient token streaming with batching and backpressure.
"""

from typing import AsyncGenerator, Generator, TypeVar, Union
from collections.abc import AsyncIterator, Iterator

T = TypeVar('T')


class TokenBatcher:
    """Batches tokens for efficient streaming."""
    
    def __init__(self, batch_size: int = 20):
        """
        Initialize token batcher.
        
        Args:
            batch_size: Number of characters per batch
        """
        self.batch_size = batch_size
        self.buffer = ""
    
    def add(self, token: str) -> str | None:
        """
        Add token to buffer, return batch if ready.
        
        Args:
            token: Token to add
            
        Returns:
            Batch if ready, None otherwise
        """
        self.buffer += token
        if len(self.buffer) >= self.batch_size:
            batch = self.buffer
            self.buffer = ""
            return batch
        return None
    
    def flush(self) -> str | None:
        """
        Flush remaining buffer.
        
        Returns:
            Remaining tokens or None
        """
        if self.buffer:
            batch = self.buffer
            self.buffer = ""
            return batch
        return None


async def batch_tokens(
    stream: AsyncIterator[str],
    batch_size: int = 20
) -> AsyncGenerator[str, None]:
    """
    Batch tokens from async stream for efficient transmission.
    
    Args:
        stream: Async token stream
        batch_size: Tokens per batch
        
    Yields:
        Batched tokens
    """
    batcher = TokenBatcher(batch_size)
    
    async for token in stream:
        batch = batcher.add(token)
        if batch:
            yield batch
    
    # Flush remaining
    final = batcher.flush()
    if final:
        yield final


def batch_tokens_sync(
    stream: Iterator[str],
    batch_size: int = 20
) -> Generator[str, None, None]:
    """
    Batch tokens from sync stream for efficient transmission.
    
    Args:
        stream: Sync token stream
        batch_size: Tokens per batch
        
    Yields:
        Batched tokens
    """
    batcher = TokenBatcher(batch_size)
    
    for token in stream:
        batch = batcher.add(token)
        if batch:
            yield batch
    
    # Flush remaining
    final = batcher.flush()
    if final:
        yield final


class StreamCounter:
    """Counts tokens in stream without consuming."""
    
    def __init__(self):
        self.count = 0
        self.chars = 0
    
    def track(self, token: str) -> None:
        """Track token statistics."""
        self.count += 1
        self.chars += len(token)
    
    def reset(self) -> tuple[int, int]:
        """Reset and return counts."""
        count, chars = self.count, self.chars
        self.count = 0
        self.chars = 0
        return count, chars

