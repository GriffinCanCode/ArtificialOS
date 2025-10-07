"""
Distributed Tracing
Lightweight tracing for AI service operations with gRPC integration.
"""

import contextvars
import functools
import logging
import time
import uuid
from collections.abc import Callable
from contextlib import asynccontextmanager, contextmanager
from dataclasses import dataclass, field
from typing import Any, AsyncGenerator, TypeVar

import structlog

logger: structlog.BoundLogger | None = None


def _get_logger() -> structlog.BoundLogger:
    """Get or create logger instance."""
    global logger
    if logger is None:
        logger = structlog.get_logger(__name__)
    return logger

F = TypeVar("F", bound=Callable[..., Any])

# Context variables for trace propagation
_trace_id: contextvars.ContextVar[str] = contextvars.ContextVar("trace_id", default="")
_span_id: contextvars.ContextVar[str] = contextvars.ContextVar("span_id", default="")


@dataclass
class Span:
    """Represents a single traced operation."""

    trace_id: str
    span_id: str
    parent_id: str
    name: str
    service: str
    start_time: float
    end_time: float = 0.0
    duration: float = 0.0
    tags: dict[str, str] = field(default_factory=dict)
    logs: list[dict[str, Any]] = field(default_factory=list)
    error: Exception | None = None
    status_code: int = 200

    def finish(self) -> None:
        """Mark span as complete."""
        self.end_time = time.time()
        self.duration = self.end_time - self.start_time

    def set_tag(self, key: str, value: str) -> None:
        """Add a tag to the span."""
        self.tags[key] = value

    def set_error(self, error: Exception) -> None:
        """Record an error in the span."""
        self.error = error
        self.status_code = 500

    def set_status(self, code: int) -> None:
        """Set the status code."""
        self.status_code = code

    def log(self, message: str, **fields: Any) -> None:
        """Add a log entry to the span."""
        self.logs.append({"timestamp": time.time(), "message": message, "fields": fields})


class Tracer:
    """Manages distributed tracing."""

    def __init__(self, service: str) -> None:
        self.service = service

    def start_span(self, name: str, **tags: str) -> Span:
        """Create a new span."""
        trace_id = _trace_id.get() or str(uuid.uuid4())
        parent_id = _span_id.get()
        span_id = str(uuid.uuid4())

        span = Span(
            trace_id=trace_id,
            span_id=span_id,
            parent_id=parent_id,
            name=name,
            service=self.service,
            start_time=time.time(),
            tags=tags,
        )

        # Set span in context
        _trace_id.set(trace_id)
        _span_id.set(span_id)

        return span

    def submit(self, span: Span) -> None:
        """Process completed span."""
        log = _get_logger()
        fields = {
            "trace_id": span.trace_id,
            "span_id": span.span_id,
            "operation": span.name,
            "duration_ms": span.duration * 1000,
            "service": span.service,
            "status_code": span.status_code,
            **span.tags,
        }

        if span.parent_id:
            fields["parent_id"] = span.parent_id

        if span.error:
            log.error("span_completed_with_error", error=str(span.error), **fields)
        else:
            if span.duration > 1.0:
                log.warning("span_completed_slow", **fields)
            else:
                log.info("span_completed", **fields)


# Global tracer instance
_tracer: Tracer | None = None


def init_tracer(service: str) -> Tracer:
    """Initialize global tracer."""
    global _tracer
    _tracer = Tracer(service)
    return _tracer


def get_tracer() -> Tracer:
    """Get global tracer instance."""
    if _tracer is None:
        raise RuntimeError("Tracer not initialized. Call init_tracer() first.")
    return _tracer


@contextmanager
def trace_operation(operation: str, **kwargs: Any):
    """Context manager for tracing operations."""
    if _tracer is None:
        yield
        return

    span = _tracer.start_span(operation, **{k: str(v) for k, v in kwargs.items()})
    try:
        yield span
    except Exception as e:
        span.set_error(e)
        raise
    finally:
        span.finish()
        _tracer.submit(span)


@asynccontextmanager
async def trace_operation_async(operation: str, **kwargs: Any) -> AsyncGenerator[Span, None]:
    """Async context manager for tracing operations."""
    if _tracer is None:
        yield None  # type: ignore
        return

    span = _tracer.start_span(operation, **{k: str(v) for k, v in kwargs.items()})
    try:
        yield span
    except Exception as e:
        span.set_error(e)
        raise
    finally:
        span.finish()
        _tracer.submit(span)


def trace_grpc_call(method: str) -> Callable[[F], F]:
    """Decorator for tracing gRPC calls."""

    def decorator(func: F) -> F:
        @functools.wraps(func)
        async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
            async with trace_operation_async("grpc_call", method=method):
                return await func(*args, **kwargs)

        @functools.wraps(func)
        def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
            with trace_operation("grpc_call", method=method):
                return func(*args, **kwargs)

        if functools.iscoroutinefunction(func):
            return async_wrapper  # type: ignore
        return sync_wrapper  # type: ignore

    return decorator


def extract_trace_context(metadata: dict[str, list[str]]) -> tuple[str, str]:
    """Extract trace context from gRPC metadata."""
    trace_id = metadata.get("x-trace-id", [""])[0] if metadata else ""
    span_id = metadata.get("x-span-id", [""])[0] if metadata else ""
    return trace_id, span_id


def inject_trace_context(metadata: dict[str, str]) -> None:
    """Inject trace context into gRPC metadata."""
    trace_id = _trace_id.get()
    span_id = _span_id.get()

    if trace_id:
        metadata["x-trace-id"] = trace_id
    if span_id:
        metadata["x-span-id"] = span_id


def get_trace_id() -> str:
    """Get current trace ID from context."""
    return _trace_id.get()


def get_span_id() -> str:
    """Get current span ID from context."""
    return _span_id.get()


def set_trace_context(trace_id: str, span_id: str) -> None:
    """Set trace context for distributed tracing."""
    if trace_id:
        _trace_id.set(trace_id)
    if span_id:
        _span_id.set(span_id)
