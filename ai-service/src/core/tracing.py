"""
Distributed Tracing
Lightweight tracing for AI service operations.
"""

import contextvars
import time
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from core import get_logger

logger = get_logger(__name__)

# Context variables for trace propagation
_trace_id: contextvars.ContextVar[str] = contextvars.ContextVar(
    "trace_id", default=""
)
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
        self.logs.append(
            {"timestamp": time.time(), "message": message, "fields": fields}
        )


class Tracer:
    """Manages distributed tracing."""

    def __init__(self, service: str) -> None:
        self.service = service

    def start_span(self, name: str) -> Span:
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
        )

        # Set span in context
        _trace_id.set(trace_id)
        _span_id.set(span_id)

        return span

    def submit(self, span: Span) -> None:
        """Process completed span."""
        fields = {
            "trace_id": span.trace_id,
            "span_id": span.span_id,
            "operation": span.name,
            "duration_ms": span.duration * 1000,
            "service": span.service,
            "status_code": span.status_code,
        }

        if span.parent_id:
            fields["parent_id"] = span.parent_id

        if span.error:
            logger.error("span_completed_with_error", error=str(span.error), **fields)
        else:
            logger.info("span_completed", **fields)


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
    """Set trace context."""
    if trace_id:
        _trace_id.set(trace_id)
    if span_id:
        _span_id.set(span_id)


def traced(name: str | None = None) -> Callable:
    """Decorator for automatic span creation."""

    def decorator(func: Callable) -> Callable:
        span_name = name or f"{func.__module__}.{func.__name__}"

        def wrapper(*args: Any, **kwargs: Any) -> Any:
            from core import get_container

            container = get_container()
            if not hasattr(container, "tracer"):
                # Tracing not initialized
                return func(*args, **kwargs)

            tracer = container.tracer
            span = tracer.start_span(span_name)

            try:
                result = func(*args, **kwargs)
                span.set_status(200)
                return result
            except Exception as e:
                span.set_error(e)
                raise
            finally:
                span.finish()
                tracer.submit(span)

        return wrapper

    return decorator
