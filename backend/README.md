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

## Structure

```
go-service/
├── cmd/server/          # Main entry point
├── internal/
│   ├── app/            # App lifecycle management
│   ├── service/        # Service registry
│   ├── grpc/           # gRPC clients (kernel, AI)
│   ├── http/           # HTTP handlers
│   ├── ws/             # WebSocket handlers
│   ├── server/         # Server setup
│   └── types/          # Shared types
├── proto/              # Protocol buffer definitions
└── Makefile           # Build commands
```

## Quick Start

```bash
# Install dependencies
make deps

# Run tests
make test

# Start server
make run

# Or with custom ports
go run ./cmd/server -port 8000 -kernel localhost:50051 -ai localhost:50052
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

Environment variables:
- `PORT` - Server port (default: 8000)
- `KERNEL_ADDR` - Kernel gRPC address (default: localhost:50051)
- `AI_ADDR` - AI service gRPC address (default: localhost:50052)

## Performance

- Concurrent request handling with goroutines
- Connection pooling for gRPC clients
- Efficient WebSocket multiplexing
- Zero-copy JSON parsing where possible

## Design Principles

1. **Strong Typing** - Leverage Go's type system
2. **Testability** - Interface-based design for mocking
3. **Concurrency** - goroutines and channels throughout
4. **Simplicity** - One-word file names, focused functions
5. **Zero Tech Debt** - Clean code from day one

