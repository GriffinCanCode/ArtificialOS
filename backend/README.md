# Go Service

High-performance orchestration layer for the AI-OS system.

## Architecture

```
Frontend → Go Service → Python AI Service (LLM)
                    → Rust Kernel (Syscalls)
```

## Features

- **Concurrent App Management** - goroutines for multi-app orchestration
- **Type-Safe Service Registry** - compile-time safety with Go
- **WebSocket Streaming** - real-time updates from AI service
- **gRPC Integration** - efficient communication with kernel and AI
- **HTTP REST API** - standard endpoints for all operations
- **Structured Logging** - production-ready logging with zap (JSON/console)
- **Rate Limiting** - per-IP rate limiting with token bucket algorithm
- **Configuration Management** - environment-based 12-factor config
- **Production Middleware** - CORS, recovery, request logging

## Structure

```
go-service/
├── cmd/server/          # Main entry point
├── internal/
│   ├── app/            # App lifecycle management
│   ├── config/         # Configuration management
│   ├── grpc/           # gRPC clients (kernel, AI)
│   ├── http/           # HTTP handlers
│   ├── logging/        # Structured logging
│   ├── middleware/     # HTTP middleware (CORS, rate limiting)
│   ├── providers/      # Service providers (auth, storage, filesystem)
│   ├── registry/       # App registry management
│   ├── server/         # Server setup
│   ├── service/        # Service registry
│   ├── session/        # Session management
│   ├── types/          # Shared types
│   ├── utils/          # Utility functions
│   └── ws/             # WebSocket handlers
├── proto/              # Protocol buffer definitions
└── Makefile           # Build commands
```

## Quick Start

```bash
# Install dependencies
make deps

# Run tests
make test

# Start server (development mode)
make run-dev

# Start server (production mode)
make run

# Or run directly with flags
go run ./cmd/server -dev -port 8000
```

## API Endpoints

### Health
- `GET /` - Basic health check
- `GET /health` - Detailed health with stats

### Apps
- `GET /apps` - List all apps
- `POST /apps/:id/focus` - Focus an app
- `DELETE /apps/:id` - Close an app

### Services
- `GET /services` - List all services
- `POST /services/discover` - Discover services for intent
- `POST /services/execute` - Execute a service tool

### AI
- `POST /generate-ui` - Generate UI (non-streaming)
- `GET /stream` - WebSocket for streaming AI operations

## WebSocket Protocol

### Client → Server
```json
{"type": "chat", "message": "...", "context": {...}}
{"type": "generate_ui", "message": "...", "context": {...}}
{"type": "ping"}
```

### Server → Client
```json
{"type": "generation_start", "message": "...", "timestamp": 123}
{"type": "thought", "content": "...", "timestamp": 123}
{"type": "token", "content": "...", "timestamp": 123}
{"type": "ui_generated", "app_id": "...", "ui_spec": {...}, "timestamp": 123}
{"type": "complete", "timestamp": 123}
{"type": "error", "message": "..."}
```

## Testing

```bash
# Run all tests
go test ./...

# Run with coverage
go test -cover ./...

# Run specific package
go test ./internal/app/...
```

## Development

```bash
# Format code
make fmt

# Lint code
make lint

# Generate protobuf code
make proto
```

## Production Build

```bash
# Build binary
make build

# Run binary
./bin/server -port 8000
```

## Configuration

The server supports both environment variables and CLI flags (flags override env vars).

### Environment Variables

**Server Configuration:**
```bash
PORT=8000                    # Server port (default: 8000)
HOST=0.0.0.0                 # Server host (default: 0.0.0.0)
```

**Service Addresses:**
```bash
KERNEL_ADDR=localhost:50051  # Kernel gRPC address (default: localhost:50051)
KERNEL_ENABLED=true          # Enable/disable kernel (default: true)
AI_ADDR=localhost:50052      # AI service gRPC address (default: localhost:50052)
```

**Logging:**
```bash
LOG_LEVEL=info               # Log level: debug, info, warn, error (default: info)
LOG_DEV=false                # Development mode: colored console logs (default: false)
```

**Rate Limiting:**
```bash
RATE_LIMIT_RPS=100           # Requests per second per IP (default: 100)
RATE_LIMIT_BURST=200         # Burst capacity (default: 200)
RATE_LIMIT_ENABLED=true      # Enable/disable rate limiting (default: true)
```

### CLI Flags

```bash
./bin/server \
  -port 8000 \
  -kernel localhost:50051 \
  -ai localhost:50052 \
  -dev  # Enable development mode (colored logs, debug level)
```

### Example Configurations

**Development:**
```bash
export LOG_DEV=true
export LOG_LEVEL=debug
export RATE_LIMIT_ENABLED=false
./bin/server -dev
```

**Production:**
```bash
export LOG_LEVEL=info
export RATE_LIMIT_RPS=1000
export RATE_LIMIT_BURST=2000
./bin/server
```

**Docker/Kubernetes:**
```yaml
env:
  - name: PORT
    value: "8000"
  - name: LOG_LEVEL
    value: "info"
  - name: RATE_LIMIT_RPS
    value: "500"
```

## Performance

- **Concurrent request handling** - goroutines for all operations
- **Connection pooling** - gRPC client connection reuse
- **Efficient WebSocket multiplexing** - single connection per client
- **Rate limiting** - token bucket algorithm prevents abuse
- **Structured logging** - zero-allocation logging in production
- **Optimized string operations** - `strings.Builder` for O(n) performance

## Design Principles

1. **Strong Typing** - Leverage Go's type system for compile-time safety
2. **Testability** - Interface-based design for easy mocking
3. **Concurrency** - goroutines and channels throughout
4. **Simplicity** - One-word package names, focused files (<250 lines)
5. **Zero Tech Debt** - Production-ready libraries, no custom implementations
6. **Extensibility** - Configuration-driven behavior, easy to extend
7. **Observability** - Structured logging for production debugging

## Dependencies

**Core:**
- `gin-gonic/gin` - HTTP framework
- `gorilla/websocket` - WebSocket support
- `google/uuid` - UUID generation
- `grpc` - gRPC communication

**New (Production-Grade):**
- `gin-contrib/cors` - CORS middleware
- `go.uber.org/zap` - Structured logging
- `golang.org/x/time/rate` - Rate limiting
- `kelseyhightower/envconfig` - Configuration management

## Middleware

The server includes production-ready middleware:

1. **Recovery** - Panic recovery with graceful error responses
2. **CORS** - Cross-origin resource sharing (configurable origins)
3. **Rate Limiting** - Per-IP token bucket rate limiting
4. **Logging** - Request/response logging with structured fields

## Logging

Structured logging with different modes:

**Development Mode:**
```
2025-01-04T12:00:00.000Z    INFO    Starting server    port=8000 kernel=localhost:50051
```

**Production Mode (JSON):**
```json
{"timestamp":"2025-01-04T12:00:00.000Z","level":"info","message":"Starting server","port":"8000","kernel":"localhost:50051"}
```

**Log Levels:**
- `Debug` - Verbose debugging information
- `Info` - General informational messages
- `Warn` - Warning messages
- `Error` - Error messages
- `Fatal` - Fatal errors (exits process)

## Troubleshooting

**Port already in use:**
```bash
lsof -i :8000  # Check what's using the port
kill -9 <PID>  # Kill the process
```

**Kernel/AI service not connecting:**
```bash
# Check if services are running
lsof -i :50051  # Kernel
lsof -i :50052  # AI service

# Disable kernel if not needed
export KERNEL_ENABLED=false
./bin/server
```

**Rate limiting issues:**
```bash
# Disable for development
export RATE_LIMIT_ENABLED=false
./bin/server -dev

# Or increase limits
export RATE_LIMIT_RPS=1000
export RATE_LIMIT_BURST=2000
```

