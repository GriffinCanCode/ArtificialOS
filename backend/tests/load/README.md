# Load Testing

Comprehensive load testing suite to identify system breaking points and performance bottlenecks.

## Overview

This directory contains load testing scripts using [Vegeta](https://github.com/tsenart/vegeta), a powerful HTTP load testing tool written in Go.

## Prerequisites

```bash
# Install Vegeta
go install github.com/tsenart/vegeta@latest
```

## Test Scenarios

### 1. API Endpoint Load Test

Tests backend HTTP endpoints under various load conditions:

```bash
# Light load - 10 req/sec for 30 seconds
vegeta attack -targets=targets/api.txt -rate=10 -duration=30s | vegeta report

# Medium load - 50 req/sec for 60 seconds
vegeta attack -targets=targets/api.txt -rate=50 -duration=60s | vegeta report

# Heavy load - 100 req/sec for 60 seconds
vegeta attack -targets=targets/api.txt -rate=100 -duration=60s | vegeta report

# Stress test - 200 req/sec for 120 seconds
vegeta attack -targets=targets/api.txt -rate=200 -duration=120s | vegeta report
```

### 2. gRPC Service Load Test

Tests kernel and AI service gRPC endpoints:

```bash
# Run gRPC load tests
go run grpc_load_test.go -addr localhost:50051 -requests 1000 -workers 10
```

### 3. WebSocket Connection Load Test

Tests WebSocket connection handling:

```bash
# Run WebSocket load test
go run ws_load_test.go -addr localhost:8000 -connections 100 -duration 60s
```

## Results Analysis

### HTML Report

```bash
vegeta attack -targets=targets/api.txt -rate=50 -duration=60s | vegeta report -type=html > report.html
open report.html
```

### JSON Report

```bash
vegeta attack -targets=targets/api.txt -rate=50 -duration=60s | vegeta encode | vegeta report -type=json > results.json
```

### Metrics

- **Latency**: p50, p95, p99, max
- **Throughput**: Requests per second
- **Success Rate**: Percentage of successful requests
- **Error Rate**: Failed requests breakdown
- **Resource Usage**: CPU, memory, network

## Performance Baselines

### Backend API
- **Target**: 100 req/sec with p99 < 500ms
- **Acceptable**: 50 req/sec with p99 < 1s
- **Critical**: Maintain 80% success rate at 200 req/sec

### gRPC Services
- **Kernel**: 500 req/sec with p99 < 100ms
- **AI Service**: 20 req/sec with p99 < 2s (LLM latency)

### WebSocket
- **Connections**: Support 1000 concurrent connections
- **Messages**: 100 msg/sec per connection

## Running Full Suite

```bash
make load-test
```

## Continuous Load Testing

For sustained load testing:

```bash
# Run for 10 minutes
vegeta attack -targets=targets/api.txt -rate=30 -duration=600s | vegeta plot > plot.html
```

## Integration with CI/CD

Load tests can be integrated into CI/CD pipelines to detect performance regressions:

```bash
# Run and check for p99 < 500ms
vegeta attack -targets=targets/api.txt -rate=50 -duration=30s | \
  vegeta report -type=json | \
  jq '.latencies."99th" < 500000000' # nanoseconds
```

## Best Practices

1. **Gradual Ramp-Up**: Start with low load and gradually increase
2. **Monitor Resources**: Track CPU, memory, network during tests
3. **Baseline Comparison**: Compare against previous runs
4. **Realistic Scenarios**: Use production-like data and patterns
5. **Test in Isolation**: Test individual services before full stack
6. **Document Results**: Keep history of load test outcomes

## Troubleshooting

### High Error Rates

- Check service logs for errors
- Verify resource limits (file descriptors, memory)
- Review database connection pools
- Check network configuration

### High Latency

- Profile code for bottlenecks
- Check database query performance
- Review caching strategy
- Analyze gRPC keepalive settings

### Connection Failures

- Verify firewall rules
- Check service availability
- Review timeout settings
- Monitor connection pool exhaustion
