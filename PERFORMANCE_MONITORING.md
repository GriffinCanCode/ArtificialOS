# Performance Monitoring Setup

This document describes the complete performance monitoring implementation across all layers of AgentOS.

## Overview

AgentOS uses a comprehensive, modular performance monitoring system that tracks metrics across four layers:
- **Kernel (Rust)**: Custom metrics collector with Prometheus-compatible export
- **Backend (Go)**: Prometheus client_golang with automatic HTTP middleware
- **AI Service (Python)**: prometheus-client with structured tracing
- **UI (TypeScript)**: web-vitals + custom metrics collector

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                 Metrics Collection                  │
├─────────────────────────────────────────────────────┤
│  Kernel (Rust)     │  Backend (Go)                  │
│  - MetricsCollector│  - prometheus client_golang    │
│  - Counters        │  - HTTP middleware             │
│  - Gauges          │  - Service timers              │
│  - Histograms      │                                │
├────────────────────┼────────────────────────────────┤
│  AI Service (Py)   │  UI (TypeScript)               │
│  - prometheus-cli  │  - web-vitals                  │
│  - Tracing         │  - Custom collector            │
│  - gRPC metrics    │  - Tool execution timing       │
└────────────────────┴────────────────────────────────┘
```

## Implementation Details

### 1. Kernel (Rust)

**Location**: `kernel/src/monitoring/`

**Components**:
- `metrics.rs`: MetricsCollector with counters, gauges, histograms
- `tracer.rs`: Structured tracing for syscalls and operations
- `api/metrics.rs`: Prometheus format exporter

**Integration**:
```rust
// In syscall executor
let _span = span_syscall(&format!("{:?}", syscall), pid);
metrics.inc_counter(&format!("syscall_{:?}", syscall), 1.0);
```

**Key Metrics**:
- `kernel_syscall_*`: Individual syscall counters
- `kernel_syscall_permission_denied`: Permission check failures
- `kernel_uptime_seconds`: System uptime

### 2. Backend (Go)

**Location**: `backend/internal/infrastructure/monitoring/`

**Components**:
- `metrics.go`: Prometheus metrics definitions
- `middleware.go`: Gin middleware for automatic HTTP tracking
- `helpers.go`: Prometheus export formatting

**Integration**:
```go
// Automatic via middleware
router.Use(monitoring.Middleware(metrics))

// Manual tracking in handlers
defer h.metrics.TrackAppOperation("list")()
```

**Key Metrics**:
- `backend_http_requests_total`: HTTP request counts
- `backend_http_request_duration_seconds`: Request latency (histogram)
- `backend_apps_active`: Active application count
- `backend_service_calls_total`: Service call counts
- `backend_grpc_calls_total`: gRPC operation counts

**Endpoint**: `GET /metrics` (Prometheus format)

### 3. AI Service (Python)

**Location**: `ai-service/src/monitoring/`

**Components**:
- `metrics.py`: Prometheus metrics collector
- `tracer.py`: Structured tracing with context managers

**Integration**:
```python
# In handlers
with trace_operation("ui_generation", message=request.message[:50]):
    # ... operation ...
    metrics_collector.record_ui_request("success", duration, "generate")
```

**Key Metrics**:
- `ai_ui_requests_total`: UI generation requests
- `ai_ui_duration_seconds`: Generation latency (histogram)
- `ai_chat_requests_total`: Chat requests
- `ai_llm_calls_total`: LLM API calls
- `ai_cache_hits_total`: Cache performance

### 4. UI (TypeScript/React)

**Location**: `ui/src/core/monitoring/`

**Components**:
- `metrics.ts`: Custom metrics collector (Prometheus-compatible)
- `vitals.ts`: Core Web Vitals tracking
- `types.ts`: Type definitions

**Integration**:
```typescript
// Tool execution
const timer = new Timer("tool_execution", { tool: toolId });
// ... execute tool ...
timer.stop();

// Component rendering
metricsCollector.incCounter("component_renders_total", 1, { component: "Button" });
```

**Key Metrics**:
- `ui_tool_executions_total`: Tool execution counts
- `ui_tool_execution_duration`: Tool latency (histogram)
- `ui_webvitals_cls`: Cumulative Layout Shift
- `ui_webvitals_fid`: First Input Delay
- `ui_webvitals_lcp`: Largest Contentful Paint

## Metrics Endpoints

| Service | Port | Endpoint | Format |
|---------|------|----------|--------|
| Kernel | 50051 | gRPC API | JSON/Prometheus |
| Backend | 8000 | `/metrics` | Prometheus |
| AI Service | 50052 | gRPC API | Prometheus |
| UI | 5173 | Client-side | JSON |

## Prometheus Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'agentos-backend'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Add kernel and AI service as needed
```

Start Prometheus:
```bash
prometheus --config.file=prometheus.yml
```

## Grafana Dashboards

### Backend Dashboard

```json
{
  "dashboard": {
    "title": "AgentOS Backend",
    "panels": [
      {
        "title": "HTTP Request Rate",
        "targets": [
          {
            "expr": "rate(backend_http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Request Latency (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, backend_http_request_duration_seconds_bucket)"
          }
        ]
      }
    ]
  }
}
```

## Best Practices

### 1. Set Alerts

Configure alerting for critical metrics:
```yaml
groups:
  - name: agentos
    rules:
      - alert: HighLatency
        expr: histogram_quantile(0.95, backend_http_request_duration_seconds_bucket) > 1
        for: 5m
        annotations:
          summary: "High p95 latency detected"
```

### 2. Dashboard Organization

- **System Health**: Uptime, error rates, active connections
- **Performance**: Latency percentiles (p50, p95, p99)
- **Throughput**: Requests per second, operations per second
- **Resources**: Memory usage, CPU usage, cache performance

### 3. Metric Naming

Follow Prometheus conventions:
- Use base unit (seconds, bytes, not milliseconds)
- Suffix with unit (`_seconds`, `_bytes`, `_total`)
- Use consistent labeling

### 4. Cardinality Management

- Avoid high-cardinality labels (user IDs, timestamps)
- Use bounded label values
- Aggregate before recording when possible

## Testing Metrics

### Backend
```bash
curl http://localhost:8000/metrics
```

### Check specific metric:
```bash
curl http://localhost:8000/metrics | grep backend_http_requests_total
```

### UI (Browser Console)
```javascript
// Get all metrics
console.log(metricsCollector.getMetricsJSON());

// Get specific stats
console.log(metricsCollector.getStats("tool_execution"));
```

## Troubleshooting

### Missing Metrics

1. **Kernel**: Check that `with_metrics()` is called on SyscallExecutor
2. **Backend**: Verify middleware is registered before routes
3. **AI Service**: Ensure `metrics_collector` is imported in handlers
4. **UI**: Check that monitoring is initialized in App.tsx

### High Cardinality

If Prometheus storage grows too large:
- Review label usage
- Reduce metric retention period
- Use recording rules for aggregations

### Performance Impact

Metrics collection should be lightweight:
- Use sampling for high-frequency events
- Aggregate in memory before export
- Use efficient data structures (ring buffers)

## Future Enhancements

- [ ] Distributed tracing with OpenTelemetry
- [ ] Custom metric exporters (StatsD, DataDog)
- [ ] Automatic anomaly detection
- [ ] Real-time alerting dashboard
- [ ] Performance regression testing

## References

- [Prometheus Documentation](https://prometheus.io/docs/)
- [web-vitals](https://github.com/GoogleChrome/web-vitals)
- [Prometheus client_golang](https://github.com/prometheus/client_golang)
- [prometheus-client Python](https://github.com/prometheus/client_python)
