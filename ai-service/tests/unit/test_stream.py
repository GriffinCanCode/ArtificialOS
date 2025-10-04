"""Tests for stream utilities."""

import pytest
from src.core.stream import TokenBatcher, StreamCounter, batch_tokens, batch_tokens_sync


@pytest.mark.unit
def test_token_batcher_add():
    """Test adding tokens to batcher."""
    batcher = TokenBatcher(batch_size=10)
    
    # Add small token
    result = batcher.add("hello")
    assert result is None  # Not enough to flush
    
    # Add token that triggers flush
    result = batcher.add(" world!")
    assert result == "hello world!"


@pytest.mark.unit
def test_token_batcher_flush():
    """Test flushing remaining tokens."""
    batcher = TokenBatcher(batch_size=10)
    
    batcher.add("hi")
    batcher.add(" ")
    
    result = batcher.flush()
    assert result == "hi "
    
    # Second flush returns None
    result = batcher.flush()
    assert result is None


@pytest.mark.unit
def test_token_batcher_exact_batch():
    """Test exact batch size."""
    batcher = TokenBatcher(batch_size=5)
    
    result = batcher.add("12345")
    assert result == "12345"


@pytest.mark.unit
def test_token_batcher_oversized_token():
    """Test token larger than batch size."""
    batcher = TokenBatcher(batch_size=3)
    
    result = batcher.add("hello")
    assert result == "hello"


@pytest.mark.unit
def test_token_batcher_multiple_batches():
    """Test multiple batch flushes."""
    batcher = TokenBatcher(batch_size=5)
    
    results = []
    
    for token in ["12", "34", "56", "78", "90"]:
        result = batcher.add(token)
        if result:
            results.append(result)
    
    # Final flush
    final = batcher.flush()
    if final:
        results.append(final)
    
    assert len(results) > 0
    assert "".join(results) == "1234567890"


@pytest.mark.unit
def test_stream_counter_track():
    """Test stream counter tracking."""
    counter = StreamCounter()
    
    counter.track("hello")
    counter.track(" ")
    counter.track("world")
    
    assert counter.count == 3
    assert counter.chars == 11


@pytest.mark.unit
def test_stream_counter_reset():
    """Test counter reset."""
    counter = StreamCounter()
    
    counter.track("test")
    counter.track(" ")
    counter.track("data")
    
    count, chars = counter.reset()
    
    assert count == 3
    assert chars == 9
    assert counter.count == 0
    assert counter.chars == 0


@pytest.mark.unit
@pytest.mark.asyncio
async def test_batch_tokens_async():
    """Test async token batching."""
    async def token_stream():
        for token in ["a", "b", "c", "d", "e", "f"]:
            yield token
    
    batches = []
    async for batch in batch_tokens(token_stream(), batch_size=3):
        batches.append(batch)
    
    assert len(batches) == 2
    assert batches[0] == "abc"
    assert batches[1] == "def"


@pytest.mark.unit
@pytest.mark.asyncio
async def test_batch_tokens_async_with_remainder():
    """Test async batching with remainder."""
    async def token_stream():
        for token in ["1", "2", "3", "4", "5"]:
            yield token
    
    batches = []
    async for batch in batch_tokens(token_stream(), batch_size=2):
        batches.append(batch)
    
    assert len(batches) == 3
    assert batches[0] == "12"
    assert batches[1] == "34"
    assert batches[2] == "5"


@pytest.mark.unit
def test_batch_tokens_sync():
    """Test sync token batching."""
    def token_stream():
        for token in ["a", "b", "c", "d", "e", "f"]:
            yield token
    
    batches = list(batch_tokens_sync(token_stream(), batch_size=3))
    
    assert len(batches) == 2
    assert batches[0] == "abc"
    assert batches[1] == "def"


@pytest.mark.unit
def test_batch_tokens_sync_with_remainder():
    """Test sync batching with remainder."""
    def token_stream():
        for token in ["1", "2", "3", "4", "5"]:
            yield token
    
    batches = list(batch_tokens_sync(token_stream(), batch_size=2))
    
    assert len(batches) == 3
    assert batches[0] == "12"
    assert batches[1] == "34"
    assert batches[2] == "5"


@pytest.mark.unit
def test_batch_tokens_sync_empty_stream():
    """Test batching empty stream."""
    def empty_stream():
        return
        yield  # Unreachable
    
    batches = list(batch_tokens_sync(empty_stream(), batch_size=10))
    assert len(batches) == 0


@pytest.mark.unit
@pytest.mark.asyncio
async def test_batch_tokens_async_empty_stream():
    """Test async batching empty stream."""
    async def empty_stream():
        return
        yield  # Unreachable
    
    batches = []
    async for batch in batch_tokens(empty_stream(), batch_size=10):
        batches.append(batch)
    
    assert len(batches) == 0


@pytest.mark.unit
def test_batch_tokens_sync_variable_length():
    """Test batching with variable token lengths."""
    def token_stream():
        for token in ["hello", " ", "world", "!"]:
            yield token
    
    batches = list(batch_tokens_sync(token_stream(), batch_size=10))
    
    # Should combine into batches
    combined = "".join(batches)
    assert combined == "hello world!"


@pytest.mark.unit
@pytest.mark.benchmark
def test_token_batcher_performance(benchmark):
    """Benchmark token batcher performance."""
    def batch_tokens():
        batcher = TokenBatcher(batch_size=20)
        for i in range(1000):
            batcher.add(f"token{i}")
        batcher.flush()
    
    benchmark(batch_tokens)


@pytest.mark.unit
@pytest.mark.benchmark
def test_stream_counter_performance(benchmark):
    """Benchmark stream counter performance."""
    def count_tokens():
        counter = StreamCounter()
        for i in range(1000):
            counter.track(f"token{i}")
    
    benchmark(count_tokens)

