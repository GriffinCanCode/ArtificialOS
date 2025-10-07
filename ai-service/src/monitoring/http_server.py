"""
HTTP Metrics Server
Simple HTTP server to expose metrics in JSON format alongside Prometheus
"""

import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import Dict, Any

from monitoring.metrics import metrics_collector
from core import get_logger

logger = get_logger(__name__)


class MetricsHandler(BaseHTTPRequestHandler):
    """HTTP handler for metrics endpoints."""

    def log_message(self, format: str, *args) -> None:
        """Suppress default HTTP logging."""
        pass

    def do_GET(self) -> None:
        """Handle GET requests."""
        if self.path == "/metrics/json":
            self.send_json_metrics()
        elif self.path == "/metrics":
            self.send_prometheus_metrics()
        elif self.path == "/health":
            self.send_health()
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")

    def send_json_metrics(self) -> None:
        """Send metrics in JSON format."""
        try:
            metrics_collector.update_uptime()

            # Collect metrics snapshot
            metrics: Dict[str, Any] = {
                "status": "operational",
                "uptime_seconds": metrics_collector.uptime._value._value,
            }

            # Add counter metrics
            for metric_name in [
                "ui_requests_total",
                "chat_requests_total",
                "llm_calls_total",
                "cache_hits",
                "cache_misses",
                "grpc_requests_total",
                "stream_messages",
                "stream_errors",
                "errors_total",
            ]:
                if hasattr(metrics_collector, metric_name):
                    counter = getattr(metrics_collector, metric_name)
                    try:
                        # Get all label combinations
                        for sample in counter.collect()[0].samples:
                            label_key = "_".join(f"{k}_{v}" for k, v in sample.labels.items() if v)
                            metric_key = f"{metric_name}_{label_key}" if label_key else metric_name
                            metrics[metric_key] = sample.value
                    except (AttributeError, IndexError):
                        pass

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(metrics).encode())

        except Exception as e:
            logger.error("metrics_json_error", error=str(e))
            self.send_response(500)
            self.end_headers()
            self.wfile.write(json.dumps({"error": str(e)}).encode())

    def send_prometheus_metrics(self) -> None:
        """Send metrics in Prometheus format."""
        try:
            metrics_data = metrics_collector.get_metrics()
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; version=0.0.4")
            self.end_headers()
            self.wfile.write(metrics_data)
        except Exception as e:
            logger.error("metrics_prometheus_error", error=str(e))
            self.send_response(500)
            self.end_headers()

    def send_health(self) -> None:
        """Send health check response."""
        health = {
            "status": "healthy",
            "service": "ai-service",
            "uptime_seconds": metrics_collector.uptime._value._value,
        }
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(health).encode())


def start_metrics_server(port: int = 50053, host: str = "0.0.0.0") -> None:
    """Start the metrics HTTP server in a background thread."""
    server = HTTPServer((host, port), MetricsHandler)
    logger.info("metrics_server_starting", host=host, port=port)

    def serve():
        try:
            server.serve_forever()
        except Exception as e:
            logger.error("metrics_server_error", error=str(e))

    thread = threading.Thread(target=serve, daemon=True)
    thread.start()
    logger.info("metrics_server_started", host=host, port=port)
