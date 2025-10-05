# Go Backend Service

High-performance orchestration layer for the AI-OS system.

## Architecture

```
Frontend → Go Service → Python AI Service (LLM)
                    → Rust Kernel (Syscalls)
```

## Package Organization

Each package includes a `doc.go` file with comprehensive documentation.
Use `go doc` to view package documentation:

```bash
# View package documentation
go doc github.com/GriffinCanCode/AgentOS/backend/internal/app
go doc github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem

# View specific types/functions
go doc app.Manager
go doc filesystem.BasicOps
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
backend/
├── cmd/server/          # Main entry point
│   ├── main.go         # Server startup and signal handling
│   └── doc.go          # Package documentation
├── internal/
│   ├── app/            # App lifecycle management
│   │   ├── manager.go  # App creation, focus, state management
│   │   └── doc.go      # Package documentation
│   ├── blueprint/      # Blueprint DSL parsing
│   │   ├── parser.go   # .bp file to Package transformation
│   │   └── doc.go      # Package documentation
│   ├── config/         # Configuration management
│   │   ├── config.go   # Environment-based config
│   │   └── doc.go      # Package documentation
│   ├── grpc/           # gRPC clients (kernel, AI)
│   │   ├── kernel.go   # Kernel client wrapper
│   │   ├── ai.go       # AI client wrapper
│   │   └── doc.go      # Package documentation
│   ├── http/           # HTTP handlers
│   │   ├── handlers.go # REST API endpoints
│   │   └── doc.go      # Package documentation
│   ├── logging/        # Structured logging (zap)
│   │   ├── logger.go   # Logger configuration
│   │   └── doc.go      # Package documentation
│   ├── middleware/     # HTTP middleware
│   │   ├── cors.go     # CORS configuration
│   │   ├── rate.go     # Rate limiting
│   │   └── doc.go      # Package documentation
│   ├── providers/      # Service providers
│   │   ├── filesystem/ # File operations (modular)
│   │   │   ├── basic.go      # Read, write, create, delete
│   │   │   ├── directory.go  # List, walk, tree
│   │   │   ├── operations.go # Copy, move, rename
│   │   │   ├── metadata.go   # Stat, size, mime type
│   │   │   ├── search.go     # Find, glob, filter
│   │   │   ├── formats.go    # JSON, YAML, CSV, TOML
│   │   │   ├── archives.go   # ZIP, TAR compression
│   │   │   └── doc.go        # Package documentation
│   │   ├── http/       # HTTP client (modular)
│   │   │   ├── requests.go   # GET, POST, PUT, DELETE
│   │   │   ├── config.go     # Headers, auth, timeout
│   │   │   ├── downloads.go  # File downloads
│   │   │   ├── uploads.go    # File uploads
│   │   │   ├── parse.go      # JSON, XML parsing
│   │   │   ├── url.go        # URL manipulation
│   │   │   ├── resilience.go # Retry, rate limiting
│   │   │   ├── connection.go # Proxy, SSL, cookies
│   │   │   └── doc.go        # Package documentation
│   │   ├── math/       # Math operations (modular)
│   │   │   ├── arithmetic.go # Add, multiply, power
│   │   │   ├── trig.go       # Sin, cos, tan
│   │   │   ├── stats.go      # Mean, stdev, correlation
│   │   │   ├── constants.go  # Pi, e, tau, phi
│   │   │   ├── conversions.go # Temperature, distance
│   │   │   ├── precision.go  # High-precision decimals
│   │   │   ├── special.go    # Gamma, beta, erf
│   │   │   └── doc.go        # Package documentation
│   │   ├── scraper/    # Web scraping (modular)
│   │   │   ├── content.go    # Text, links, images
│   │   │   ├── xpath.go      # XPath queries
│   │   │   ├── extract.go    # Article extraction
│   │   │   ├── forms.go      # Form discovery
│   │   │   ├── metadata.go   # Meta tags, Open Graph
│   │   │   ├── patterns.go   # Email, phone, IP regex
│   │   │   ├── structured.go # Tables, lists
│   │   │   └── doc.go        # Package documentation
│   │   ├── auth.go     # Authentication provider
│   │   ├── storage.go  # Key-value storage provider
│   │   ├── system.go   # System info provider
│   │   └── doc.go      # Package documentation
│   ├── registry/       # App package registry
│   │   ├── manager.go  # Package CRUD with caching
│   │   ├── seeder.go   # Load .bp files on startup
│   │   └── doc.go      # Package documentation
│   ├── server/         # Server initialization
│   │   ├── server.go   # HTTP server setup
│   │   └── doc.go      # Package documentation
│   ├── service/        # Service provider registry
│   │   ├── registry.go # Service discovery and execution
│   │   └── doc.go      # Package documentation
│   ├── session/        # Session persistence
│   │   ├── manager.go  # Save and restore workspaces
│   │   └── doc.go      # Package documentation
│   ├── types/          # Shared types
│   │   ├── app.go      # App, State, Stats
│   │   ├── service.go  # Service, Tool, Context, Result
│   │   ├── session.go  # Session, Workspace, AppSnapshot
│   │   ├── registry.go # Package, PackageMetadata
│   │   ├── request.go  # HTTP request types
│   │   └── doc.go      # Package documentation
│   ├── utils/          # Utility functions
│   │   ├── hash.go        # Deterministic hashing
│   │   ├── validation.go  # Input validation
│   │   └── doc.go         # Package documentation
│   └── ws/             # WebSocket handlers
│       ├── handler.go  # Streaming chat and UI generation
│       └── doc.go      # Package documentation
├── proto/              # Protocol buffer definitions
│   ├── ai/            # AI service protos (generated)
│   │   ├── ai.pb.go
│   │   ├── ai_grpc.pb.go
│   │   └── doc.go      # Package documentation
│   └── kernel/        # Kernel service protos (generated)
│       ├── kernel.pb.go
│       ├── kernel_grpc.pb.go
│       └── doc.go      # Package documentation
├── tests/             # Test suite
│   ├── unit/          # Unit tests
│   ├── integration/   # Integration tests
│   └── helpers/       # Test utilities
├── Makefile          # Build commands
└── README.md         # This file
```

### Import Organization

All Go files follow a consistent import style:

```go
package example

import (
    // 1. Standard library (sorted alphabetically)
    "context"
    "fmt"
    "time"

    // 2. Third-party packages (sorted alphabetically)
    "github.com/gin-gonic/gin"
    "go.uber.org/zap"

    // 3. Internal packages (sorted alphabetically)
    "github.com/GriffinCanCode/AgentOS/backend/internal/app"
    "github.com/GriffinCanCode/AgentOS/backend/internal/types"
)
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

