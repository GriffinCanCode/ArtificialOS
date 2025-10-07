# Distributed Tracing

Lightweight distributed tracing for debugging production issues.

## Overview

Distributed tracing tracks requests across multiple services (Backend → AI Service → Kernel) to understand:
- Request flow and latency
- Error propagation
- Performance bottlenecks
- Service dependencies

## Architecture

```
UI Request
  ↓ [HTTP + X-Trace-ID]
Backend (HTTP Middleware)
  ↓ [gRPC + trace metadata]
AI Service (gRPC Interceptor)
  ↓ [Traced operations]
Backend
  ↓ [gRPC + trace metadata]  
Kernel (gRPC Interceptor)
```

## Usage

### Backend Setup

```go
import (
    "github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/tracing"
    "go.uber.org/zap"
)

// Create tracer
logger, _ := zap.NewProduction()
tracer := tracing.New("backend", logger)

// HTTP middleware
router.Use(tracing.HTTPMiddleware(tracer))

// gRPC server
server := grpc.NewServer(
    grpc.UnaryInterceptor(tracing.GRPCUnaryInterceptor(tracer)),
    grpc.StreamInterceptor(tracing.GRPCStreamInterceptor(tracer)),
)

// gRPC client
conn, err := grpc.Dial(addr,
    grpc.WithUnaryInterceptor(tracing.GRPCClientInterceptor(tracer)),
)
```

### Manual Span Creation

```go
func ProcessData(ctx context.Context, data string) error {
    span, ctx := tracer.StartSpan(ctx, "process_data")
    defer func() {
        span.Finish()
        tracer.Submit(span)
    }()
    
    span.SetTag("data_size", fmt.Sprintf("%d", len(data)))
    
    // Do work
    result, err := doWork(ctx, data)
    if err != nil {
        span.SetError(err)
        return err
    }
    
    span.Log("processing complete", map[string]interface{}{
        "result": result,
    })
    
    return nil
}
```

### Python AI Service

```python
from core.tracing import Tracer, traced

tracer = Tracer("ai-service")

@traced("generate_ui")
def generate_ui(message: str) -> str:
    # Automatically traced
    return process(message)

# Manual span creation
def complex_operation():
    span = tracer.start_span("complex_op")
    try:
        # Do work
        result = perform_work()
        span.set_status(200)
        return result
    except Exception as e:
        span.set_error(e)
        raise
    finally:
        span.finish()
        tracer.submit(span)
```

## Trace Context Propagation

### HTTP Headers

```
X-Trace-ID: 550e8400-e29b-41d4-a716-446655440000
X-Span-ID:  6ba7b810-9dad-11d1-80b4-00c04fd430c8
```

### gRPC Metadata

```go
// Client side - inject
headers := make(map[string]string)
tracing.InjectTraceContext(ctx, headers)
md := metadata.New(headers)
ctx = metadata.NewOutgoingContext(ctx, md)

// Server side - extract
md, ok := metadata.FromIncomingContext(ctx)
if ok {
    headers := make(map[string]string)
    if vals := md.Get("x-trace-id"); len(vals) > 0 {
        headers["X-Trace-ID"] = vals[0]
    }
    traceID, spanID := tracing.ExtractTraceContext(headers)
}
```

## Span Structure

```go
type Span struct {
    TraceID    TraceID              // Unique trace ID
    SpanID     SpanID               // Unique span ID
    ParentID   SpanID               // Parent span ID
    Name       string               // Operation name
    Service    string               // Service name
    StartTime  time.Time            // Start timestamp
    EndTime    time.Time            // End timestamp
    Duration   time.Duration        // Calculated duration
    Tags       map[string]string    // Key-value tags
    Logs       []LogEntry           // Structured logs
    Error      error                // Error if failed
    StatusCode int                  // HTTP/gRPC status
}
```

## Logging Integration

Spans are automatically logged with structured fields:

```json
{
  "level": "info",
  "ts": "2025-01-07T10:30:45.123Z",
  "msg": "span completed",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "span_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "parent_id": "12345678-90ab-cdef-1234-567890abcdef",
  "operation": "CreateProcess",
  "duration_ms": 45.2,
  "service": "backend",
  "status_code": 200
}
```

## Best Practices

### 1. Use Meaningful Span Names

```go
// Good
tracer.StartSpan(ctx, "database.query")
tracer.StartSpan(ctx, "cache.get")
tracer.StartSpan(ctx, "ai.generate_ui")

// Bad
tracer.StartSpan(ctx, "operation")
tracer.StartSpan(ctx, "func1")
```

### 2. Add Relevant Tags

```go
span.SetTag("user_id", userID)
span.SetTag("request_size", fmt.Sprintf("%d", size))
span.SetTag("cache_hit", "true")
span.SetTag("model", "gemini-pro")
```

### 3. Log Important Events

```go
span.Log("cache miss", map[string]interface{}{
    "key": cacheKey,
})

span.Log("retry attempt", map[string]interface{}{
    "attempt": 2,
    "max_retries": 3,
})
```

### 4. Always Finish Spans

```go
// Good
defer func() {
    span.Finish()
    tracer.Submit(span)
}()

// Also good
span, ctx := tracer.StartSpan(ctx, "operation")
defer span.Finish()
defer tracer.Submit(span)
```

### 5. Propagate Context

```go
// Always pass context to child operations
func ParentOp(ctx context.Context) {
    span, ctx := tracer.StartSpan(ctx, "parent")
    defer span.Finish()
    
    // Context is passed down
    ChildOp(ctx)
}

func ChildOp(ctx context.Context) {
    // This span will have correct parent
    span, ctx := tracer.StartSpan(ctx, "child")
    defer span.Finish()
}
```

## Performance

- **Overhead**: ~10μs per span
- **Memory**: 1000-span buffer (configurable)
- **Network**: No external calls (logs to stdout)
- **Async**: Span collection is non-blocking

## Configuration

### Buffer Size

Adjust buffer size based on throughput:

```go
tracer := &Tracer{
    service: "backend",
    logger:  logger,
    spans:   make(chan *Span, 5000), // Larger buffer
}
```

### Sampling (Future)

For high-traffic scenarios, implement sampling:

```go
func shouldSample(traceID TraceID) bool {
    // Sample 10% of traces
    hash := fnv.New32a()
    hash.Write([]byte(traceID))
    return hash.Sum32() % 10 == 0
}
```

## Integration with OpenTelemetry

This implementation provides a foundation for future OpenTelemetry integration:

```go
// Future migration path
import "go.opentelemetry.io/otel"

// Replace custom tracer with OTEL
tracer := otel.Tracer("backend")
```

## Visualization

### Current: Log Analysis

```bash
# Find all spans for a trace
grep "trace_id\":\"550e8400" logs/backend.log

# Calculate request latency
grep "trace_id\":\"550e8400" logs/*.log | \
  jq -r '.duration_ms' | \
  awk '{sum+=$1} END {print "Total:", sum, "ms"}'
```

### Future: Jaeger/Zipkin

Export spans to Jaeger for visualization:

```go
func (t *Tracer) exportToJaeger(span *Span) {
    // Convert to Jaeger format
    // Send to collector
}
```

## Testing

### Unit Tests

```go
func TestSpanCreation(t *testing.T) {
    tracer := tracing.New("test", logger)
    ctx := context.Background()
    
    span, ctx := tracer.StartSpan(ctx, "test_op")
    assert.NotEmpty(t, span.TraceID)
    assert.NotEmpty(t, span.SpanID)
    
    span.Finish()
    assert.True(t, span.Duration > 0)
}
```

### Integration Tests

See `backend/tests/integration/` for trace propagation tests.

## Troubleshooting

### Missing Traces

1. Check context is propagated correctly
2. Verify middleware is registered
3. Ensure spans are submitted

### High Memory Usage

1. Reduce buffer size
2. Implement sampling
3. Check for span leaks (not finished)

### Lost Spans

1. Check buffer capacity
2. Monitor "span buffer full" warnings
3. Increase buffer or add backpressure

## Related

- [Circuit Breakers](../resilience/README.md)
- [Monitoring](../monitoring/README.md)
- [Load Testing](../../../tests/load/README.md)
