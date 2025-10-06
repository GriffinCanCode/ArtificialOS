"""
Metrics Collection
Prometheus metrics for AI service performance tracking
"""

import time
from contextlib import contextmanager
from typing import Callable, Optional

from prometheus_client import Counter, Gauge, Histogram, Summary, generate_latest


class MetricsCollector:
    """
    Collects and exposes Prometheus metrics for the AI service.
    """

    def __init__(self) -> None:
        # UI Generation metrics
        self.ui_requests_total = Counter(
            "ai_ui_requests_total",
            "Total number of UI generation requests",
            ["status"],
        )
        self.ui_duration = Histogram(
            "ai_ui_duration_seconds",
            "UI generation duration in seconds",
            ["method"],
            buckets=[0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0],
        )
        self.ui_tokens = Summary(
            "ai_ui_tokens",
            "Number of tokens in UI generation",
            ["direction"],
        )

        # Chat metrics
        self.chat_requests_total = Counter(
            "ai_chat_requests_total",
            "Total number of chat requests",
            ["status"],
        )
        self.chat_duration = Histogram(
            "ai_chat_duration_seconds",
            "Chat response duration in seconds",
            buckets=[0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0],
        )
        self.chat_tokens = Summary(
            "ai_chat_tokens",
            "Number of tokens in chat response",
            ["direction"],
        )

        # LLM metrics
        self.llm_calls_total = Counter(
            "ai_llm_calls_total",
            "Total number of LLM API calls",
            ["model", "status"],
        )
        self.llm_duration = Histogram(
            "ai_llm_duration_seconds",
            "LLM API call duration in seconds",
            ["model"],
            buckets=[0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0],
        )
        self.llm_tokens = Summary(
            "ai_llm_tokens",
            "Number of tokens in LLM call",
            ["model", "type"],
        )

        # Cache metrics
        self.cache_hits = Counter(
            "ai_cache_hits_total",
            "Total number of cache hits",
            ["cache_type"],
        )
        self.cache_misses = Counter(
            "ai_cache_misses_total",
            "Total number of cache misses",
            ["cache_type"],
        )
        self.cache_size = Gauge(
            "ai_cache_size_bytes",
            "Current cache size in bytes",
            ["cache_type"],
        )

        # gRPC metrics
        self.grpc_requests_total = Counter(
            "ai_grpc_requests_total",
            "Total number of gRPC requests",
            ["method", "status"],
        )
        self.grpc_duration = Histogram(
            "ai_grpc_duration_seconds",
            "gRPC request duration in seconds",
            ["method"],
            buckets=[0.01, 0.05, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
        )

        # Stream metrics
        self.stream_messages = Counter(
            "ai_stream_messages_total",
            "Total number of stream messages",
            ["type"],
        )
        self.stream_errors = Counter(
            "ai_stream_errors_total",
            "Total number of stream errors",
            ["type"],
        )

        # Error metrics
        self.errors_total = Counter(
            "ai_errors_total",
            "Total number of errors",
            ["error_type", "component"],
        )

        # System metrics
        self.uptime = Gauge(
            "ai_uptime_seconds",
            "Service uptime in seconds",
        )
        self.start_time = time.time()

    def record_ui_request(self, status: str, duration: float, method: str = "generate") -> None:
        """Record a UI generation request."""
        self.ui_requests_total.labels(status=status).inc()
        self.ui_duration.labels(method=method).observe(duration)

    def record_ui_tokens(self, input_tokens: int, output_tokens: int) -> None:
        """Record UI generation token counts."""
        self.ui_tokens.labels(direction="input").observe(input_tokens)
        self.ui_tokens.labels(direction="output").observe(output_tokens)

    def record_chat_request(self, status: str, duration: float) -> None:
        """Record a chat request."""
        self.chat_requests_total.labels(status=status).inc()
        self.chat_duration.observe(duration)

    def record_chat_tokens(self, input_tokens: int, output_tokens: int) -> None:
        """Record chat token counts."""
        self.chat_tokens.labels(direction="input").observe(input_tokens)
        self.chat_tokens.labels(direction="output").observe(output_tokens)

    def record_llm_call(self, model: str, status: str, duration: float) -> None:
        """Record an LLM API call."""
        self.llm_calls_total.labels(model=model, status=status).inc()
        self.llm_duration.labels(model=model).observe(duration)

    def record_llm_tokens(
        self, model: str, input_tokens: int, output_tokens: int
    ) -> None:
        """Record LLM token counts."""
        self.llm_tokens.labels(model=model, type="input").observe(input_tokens)
        self.llm_tokens.labels(model=model, type="output").observe(output_tokens)

    def record_cache_hit(self, cache_type: str) -> None:
        """Record a cache hit."""
        self.cache_hits.labels(cache_type=cache_type).inc()

    def record_cache_miss(self, cache_type: str) -> None:
        """Record a cache miss."""
        self.cache_misses.labels(cache_type=cache_type).inc()

    def set_cache_size(self, cache_type: str, size_bytes: int) -> None:
        """Set the cache size."""
        self.cache_size.labels(cache_type=cache_type).set(size_bytes)

    def record_grpc_request(self, method: str, status: str, duration: float) -> None:
        """Record a gRPC request."""
        self.grpc_requests_total.labels(method=method, status=status).inc()
        self.grpc_duration.labels(method=method).observe(duration)

    def record_stream_message(self, msg_type: str) -> None:
        """Record a stream message."""
        self.stream_messages.labels(type=msg_type).inc()

    def record_stream_error(self, error_type: str) -> None:
        """Record a stream error."""
        self.stream_errors.labels(type=error_type).inc()

    def record_error(self, error_type: str, component: str) -> None:
        """Record an error."""
        self.errors_total.labels(error_type=error_type, component=component).inc()

    def update_uptime(self) -> None:
        """Update the uptime metric."""
        self.uptime.set(time.time() - self.start_time)

    @contextmanager
    def measure_duration(self, callback: Callable[[float], None]):
        """Context manager to measure operation duration."""
        start = time.time()
        try:
            yield
        finally:
            duration = time.time() - start
            callback(duration)

    def get_metrics(self) -> bytes:
        """Get metrics in Prometheus format."""
        self.update_uptime()
        return generate_latest()


# Global metrics collector instance
metrics_collector = MetricsCollector()
