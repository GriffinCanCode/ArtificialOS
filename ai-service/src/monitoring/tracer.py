"""
Distributed Tracing
Structured tracing for AI service operations
"""

import functools
import time
from contextlib import contextmanager
from typing import Any, Callable, Optional, TypeVar

from core.logging_config import get_logger

logger = get_logger(__name__)

F = TypeVar("F", bound=Callable[..., Any])


@contextmanager
def trace_operation(operation: str, **kwargs: Any):
    """
    Context manager for tracing operations with structured logging.

    Args:
        operation: Name of the operation
        **kwargs: Additional context to log
    """
    start = time.time()
    logger.info(f"operation_start: {operation}", **kwargs)

    try:
        yield
    except Exception as e:
        duration = time.time() - start
        logger.error(
            f"operation_error: {operation}",
            error=str(e),
            duration_ms=duration * 1000,
            **kwargs,
        )
        raise
    else:
        duration = time.time() - start
        if duration > 1.0:
            logger.warning(
                f"operation_slow: {operation}",
                duration_ms=duration * 1000,
                **kwargs,
            )
        else:
            logger.info(
                f"operation_end: {operation}",
                duration_ms=duration * 1000,
                **kwargs,
            )


def trace_grpc_call(method: str) -> Callable[[F], F]:
    """
    Decorator for tracing gRPC calls.

    Args:
        method: gRPC method name

    Returns:
        Decorated function
    """

    def decorator(func: F) -> F:
        @functools.wraps(func)
        async def wrapper(*args: Any, **kwargs: Any) -> Any:
            with trace_operation("grpc_call", method=method):
                return await func(*args, **kwargs)

        return wrapper  # type: ignore

    return decorator


def trace_function(func: F) -> F:
    """
    Decorator for tracing function calls.

    Args:
        func: Function to trace

    Returns:
        Decorated function
    """

    @functools.wraps(func)
    async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
        with trace_operation("function_call", function=func.__name__):
            return await func(*args, **kwargs)

    @functools.wraps(func)
    def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
        with trace_operation("function_call", function=func.__name__):
            return func(*args, **kwargs)

    if functools.iscoroutinefunction(func):
        return async_wrapper  # type: ignore
    return sync_wrapper  # type: ignore


class Span:
    """
    Span for operation tracing.
    """

    def __init__(self, name: str, **kwargs: Any):
        self.name = name
        self.context = kwargs
        self.start_time = time.time()

    def record(self, key: str, value: Any) -> None:
        """Record additional context."""
        self.context[key] = value

    def finish(self, status: str = "success") -> None:
        """Finish the span."""
        duration = time.time() - self.start_time
        logger.info(
            f"span_finish: {self.name}",
            status=status,
            duration_ms=duration * 1000,
            **self.context,
        )

    def __enter__(self) -> "Span":
        logger.info(f"span_start: {self.name}", **self.context)
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        if exc_type is not None:
            self.finish(status="error")
        else:
            self.finish(status="success")


def create_span(name: str, **kwargs: Any) -> Span:
    """
    Create a new span for tracing.

    Args:
        name: Span name
        **kwargs: Additional context

    Returns:
        Span instance
    """
    return Span(name, **kwargs)
