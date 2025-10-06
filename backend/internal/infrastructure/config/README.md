# Configuration

Type-safe configuration management using environment variables and CLI flags.

## Features

- **12-Factor Compliant** - Environment-based configuration
- **Type-Safe** - Compile-time safety with structs
- **Default Values** - Sensible defaults for all settings
- **CLI Override** - Command-line flags override env vars
- **Validation** - Built-in validation with envconfig

## Usage

### Loading Configuration

```go
import "github.com/GriffinCanCode/AgentOS/backend/internal/config"

// Load from environment
cfg, err := config.Load()
if err != nil {
    log.Fatal(err)
}

// Load with defaults on error
cfg := config.LoadOrDefault()

// Use default configuration
cfg := config.Default()
```

### Accessing Configuration

```go
// Server configuration
port := cfg.Server.Port
host := cfg.Server.Host

// Service addresses
kernelAddr := cfg.Kernel.Address
aiAddr := cfg.AI.Address

// Logging settings
logLevel := cfg.Logging.Level
isDev := cfg.Logging.Development

// Rate limiting
rps := cfg.RateLimit.RequestsPerSecond
burst := cfg.RateLimit.Burst
```

## Configuration Structure

```go
type Config struct {
    Server    ServerConfig
    Kernel    KernelConfig
    AI        AIConfig
    Logging   LogConfig
    RateLimit RateLimitConfig
}
```

## Environment Variables

### Server
```bash
PORT=8000                # Server port
HOST=0.0.0.0             # Server host
```

### Services
```bash
KERNEL_ADDR=localhost:50051  # Kernel gRPC address
KERNEL_ENABLED=true          # Enable/disable kernel
AI_ADDR=localhost:50052      # AI service gRPC address
```

### Logging
```bash
LOG_LEVEL=info          # Log level (debug|info|warn|error)
LOG_DEV=false           # Development mode
```

### Rate Limiting
```bash
RATE_LIMIT_RPS=100      # Requests per second
RATE_LIMIT_BURST=200    # Burst capacity
RATE_LIMIT_ENABLED=true # Enable/disable
```

## Default Values

All configuration has sensible defaults:

```go
Config{
    Server: ServerConfig{
        Port: "8000",
        Host: "0.0.0.0",
    },
    Kernel: KernelConfig{
        Address: "localhost:50051",
        Enabled: true,
    },
    AI: AIConfig{
        Address: "localhost:50052",
    },
    Logging: LogConfig{
        Level:       "info",
        Development: false,
    },
    RateLimit: RateLimitConfig{
        RequestsPerSecond: 100,
        Burst:             200,
        Enabled:           true,
    },
}
```

## Adding New Configuration

To add new configuration:

1. Add fields to the appropriate struct
2. Add `envconfig` struct tags
3. Update `Default()` function
4. Document in this README

Example:
```go
type ServerConfig struct {
    Port    string `envconfig:"PORT" default:"8000"`
    Host    string `envconfig:"HOST" default:"0.0.0.0"`
    Timeout int    `envconfig:"TIMEOUT" default:"30"` // NEW
}
```

## Best Practices

1. **Use environment variables in production** - More secure than config files
2. **Use CLI flags for overrides** - Useful for testing
3. **Never commit secrets** - Use environment variables for sensitive data
4. **Provide defaults** - Make configuration optional
5. **Validate early** - Check configuration at startup

## Docker/Kubernetes Example

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: backend-config
data:
  PORT: "8000"
  LOG_LEVEL: "info"
  RATE_LIMIT_RPS: "1000"
  KERNEL_ADDR: "kernel-service:50051"
  AI_ADDR: "ai-service:50052"
```

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: backend
        envFrom:
        - configMapRef:
            name: backend-config
```

