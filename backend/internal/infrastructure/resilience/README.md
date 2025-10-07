# Circuit Breaker

Lightweight circuit breaker implementation for graceful degradation.

## Overview

The circuit breaker pattern prevents cascading failures by detecting when a service is failing and temporarily blocking requests to give it time to recover.

## States

```
    Closed ──────────[failures]──────────> Open
      ↑                                      │
      │                                      │
      │                               [timeout]
      │                                      │
      │                                      ↓
      └───────[successes]─────────── Half-Open
```

- **Closed**: Normal operation, requests pass through
- **Open**: Service unavailable, requests fail immediately  
- **Half-Open**: Testing recovery, limited requests allowed

## Usage

### Basic Example

```go
import "github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"

breaker := resilience.New("my-service", resilience.Settings{
    MaxRequests: 3,
    Interval:    60 * time.Second,
    Timeout:     30 * time.Second,
    ReadyToTrip: func(counts resilience.Counts) bool {
        return counts.ConsecutiveFailures >= 5
    },
    OnStateChange: func(name string, from, to resilience.State) {
        log.Printf("Circuit %s: %s -> %s", name, from, to)
    },
})

result, err := breaker.Execute(func() (interface{}, error) {
    return client.Call()
})
```

### gRPC Client Integration

```go
type ServiceClient struct {
    client  pb.ServiceClient
    breaker *resilience.Breaker
}

func (c *ServiceClient) Call(ctx context.Context, req *pb.Request) (*pb.Response, error) {
    result, err := c.breaker.Execute(func() (interface{}, error) {
        return c.client.Call(ctx, req)
    })
    
    if err == resilience.ErrCircuitOpen {
        return nil, fmt.Errorf("service unavailable")
    }
    
    if err != nil {
        return nil, err
    }
    
    return result.(*pb.Response), nil
}
```

### HTTP Client Integration

```go
client := &http.Client{Timeout: 10 * time.Second}
breaker := resilience.New("api", settings)

result, err := breaker.Execute(func() (interface{}, error) {
    return client.Get("https://api.example.com/data")
})

if err == resilience.ErrCircuitOpen {
    // Use fallback or cached data
    return getCachedData()
}
```

## Configuration

### Settings

```go
type Settings struct {
    // Max requests allowed in half-open state
    MaxRequests uint32
    
    // Closed state interval to reset counts
    Interval time.Duration
    
    // Open state timeout before half-open
    Timeout time.Duration
    
    // Function to determine if circuit should open
    ReadyToTrip func(counts Counts) bool
    
    // Callback for state changes
    OnStateChange func(name string, from State, to State)
}
```

### Defaults

```go
MaxRequests:  1
Interval:     60 seconds
Timeout:      60 seconds
ReadyToTrip:  ConsecutiveFailures > 5
```

## Metrics

The circuit breaker tracks:

```go
type Counts struct {
    Requests             uint32
    TotalSuccesses       uint32
    TotalFailures        uint32
    ConsecutiveSuccesses uint32
    ConsecutiveFailures  uint32
}
```

Access current counts:

```go
counts := breaker.Counts()
fmt.Printf("Success rate: %.2f%%\n", 
    float64(counts.TotalSuccesses) / float64(counts.Requests) * 100)
```

## Best Practices

### 1. Set Appropriate Thresholds

```go
// Conservative - open after 10 failures
ReadyToTrip: func(c resilience.Counts) bool {
    return c.ConsecutiveFailures >= 10
}

// Aggressive - open after 3 failures
ReadyToTrip: func(c resilience.Counts) bool {
    return c.ConsecutiveFailures >= 3
}

// Percentage-based - open if >50% fail
ReadyToTrip: func(c resilience.Counts) bool {
    return c.Requests > 10 && 
           float64(c.TotalFailures)/float64(c.Requests) > 0.5
}
```

### 2. Use Meaningful Names

```go
// Good
resilience.New("kernel-grpc", settings)
resilience.New("ai-service", settings)
resilience.New("postgres-db", settings)

// Bad
resilience.New("service1", settings)
resilience.New("cb", settings)
```

### 3. Monitor State Changes

```go
OnStateChange: func(name string, from, to resilience.State) {
    metrics.RecordCircuitState(name, to.String())
    logger.Warn("circuit breaker state change",
        zap.String("breaker", name),
        zap.String("from", from.String()),
        zap.String("to", to.String()),
    )
}
```

### 4. Implement Fallbacks

```go
result, err := breaker.Execute(func() (interface{}, error) {
    return primaryService.Call()
})

if err == resilience.ErrCircuitOpen {
    // Use fallback
    return fallbackService.Call()
}
```

### 5. Test Circuit Behavior

```go
func TestCircuitBehavior(t *testing.T) {
    breaker := resilience.New("test", settings)
    
    // Trigger failures
    for i := 0; i < 5; i++ {
        breaker.Execute(func() (interface{}, error) {
            return nil, errors.New("failed")
        })
    }
    
    assert.Equal(t, resilience.StateOpen, breaker.State())
}
```

## Performance

- **Overhead**: < 1μs per request
- **Memory**: ~1KB per breaker instance
- **Thread-safe**: Uses mutex for state transitions only
- **Scalable**: Independent breakers per service

## Integration Points

### Recommended Usage

1. **gRPC Clients**: Wrap all external gRPC calls
2. **HTTP Clients**: Wrap external HTTP API calls
3. **Database**: Wrap database connection pools
4. **Cache**: Wrap cache clients (Redis, etc.)
5. **Message Queues**: Wrap queue producers/consumers

### Do NOT Use For

1. Internal function calls
2. Fast, reliable operations
3. Operations that must succeed
4. Already retried operations

## Monitoring

### Prometheus Metrics

```go
var (
    circuitState = prometheus.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "circuit_breaker_state",
            Help: "Circuit breaker state (0=closed, 1=half-open, 2=open)",
        },
        []string{"breaker"},
    )
    
    circuitRequests = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "circuit_breaker_requests_total",
            Help: "Total requests through circuit breaker",
        },
        []string{"breaker", "result"},
    )
)

OnStateChange: func(name string, from, to resilience.State) {
    circuitState.WithLabelValues(name).Set(float64(to))
}
```

## Testing

See `breaker_test.go` for comprehensive examples:

```bash
go test ./internal/infrastructure/resilience/... -v
```

## Related

- [Distributed Tracing](../tracing/README.md)
- [Load Testing](../../../tests/load/README.md)
- [Integration Tests](../../../tests/integration/)
