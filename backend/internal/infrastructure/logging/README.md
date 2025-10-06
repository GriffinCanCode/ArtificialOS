# Logging

Structured logging abstraction using Uber's Zap.

## Features

- **Structured Logging** - JSON in production, colored console in development
- **Log Levels** - Debug, Info, Warn, Error, Fatal
- **Zero Allocations** - High-performance logging
- **Context Support** - Add structured fields to logs
- **Environment Aware** - Different configs for dev/prod

## Usage

### Basic Logging

```go
import "github.com/GriffinCanCode/AgentOS/backend/internal/logging"

// Create logger
logger := logging.NewDefault() // Production
// or
logger := logging.NewDevelopment() // Development
defer logger.Sync()

// Log messages
logger.Info("Server started")
logger.Warn("High memory usage")
logger.Error("Failed to connect")
```

### Structured Fields

```go
import "go.uber.org/zap"

logger.Info("Request completed",
    zap.String("method", "GET"),
    zap.String("path", "/api/apps"),
    zap.Int("status", 200),
    zap.Duration("latency", time.Since(start)),
)

logger.Error("Database error",
    zap.Error(err),
    zap.String("query", sql),
)
```

### Configuration

```go
// Custom configuration
cfg := logging.Config{
    Level:       "debug",
    Development: true,
    OutputPaths: []string{"stdout", "/var/log/app.log"},
}

logger, err := logging.New(cfg)
if err != nil {
    panic(err)
}
```

## Log Levels

- **Debug** - Detailed debugging information (verbose)
- **Info** - General informational messages
- **Warn** - Warning messages (potential issues)
- **Error** - Error messages (errors that need attention)
- **Fatal** - Fatal errors (exits process after logging)

## Output Formats

### Development Mode
```
2025-01-04T12:00:00.000Z    INFO    Starting server    port=8000
2025-01-04T12:00:01.000Z    WARN    High memory usage  usage=85%
2025-01-04T12:00:02.000Z    ERROR   Connection failed  error="timeout"
```

### Production Mode (JSON)
```json
{
  "timestamp": "2025-01-04T12:00:00.000Z",
  "level": "info",
  "message": "Starting server",
  "port": "8000"
}
{
  "timestamp": "2025-01-04T12:00:01.000Z",
  "level": "warn",
  "message": "High memory usage",
  "usage": "85%"
}
```

## Best Practices

1. **Always defer Sync()** - Flushes buffered logs
2. **Use structured fields** - Better than string concatenation
3. **Log at appropriate levels** - Don't overuse Debug or Error
4. **Include context** - Add relevant fields for debugging
5. **Avoid sensitive data** - Don't log passwords, tokens, etc.

## Environment Detection

The logger automatically detects the environment:

```go
if logging.IsProduction() {
    // Production configuration
}

if logging.IsDevelopment() {
    // Development configuration
}
```

Set `ENV=production` or `ENV=prod` to enable production mode.

