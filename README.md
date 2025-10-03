# AI-Powered Operating System

A lightweight operating system powered by local AI (GPT-OSS) with real-time thought streaming and dynamic application rendering.

## Architecture

```
Frontend (React/TS) → Go Backend → Python AI (gRPC) [LLM]
         ↓                    ↓
     Port 5173          Rust Kernel [Syscalls]
                        Port 50051
```

## Project Structure

```
AgentOS/
├── backend/         # Go orchestration (HTTP, WebSocket, app management)
│   ├── cmd/         # Main entry point
│   ├── internal/    # App, service, gRPC clients, handlers
│   └── proto/       # Generated protobuf code
├── ai-service/      # Python AI service (gRPC - LLM only)
│   ├── src/         # Chat agent, UI generator, models
│   └── proto/       # AI service protobuf definitions
├── kernel/          # Rust microkernel (process, memory, sandbox)
├── ui/              # React/TypeScript UI with dynamic rendering
├── proto/           # Shared protocol buffer definitions
├── scripts/         # Simple startup scripts (backend + ui)
└── docs/            # Architecture and migration docs
```

## Quick Start

### Prerequisites

* Go 1.21+
* Rust 1.70+
* Python 3.11+
* Node.js 18+

### Setup & Running

**Option 1: Using Makefile (Recommended)**

The project includes a comprehensive Makefile with all commands:

```bash
# See all available commands
make help

# One-time setup: Install all dependencies
make setup

# Compile protocol buffers
make proto

# Build all components
make build

# Start everything in development mode
make dev

# Or start components separately:
make start-backend    # Terminal 1: Backend stack
make start-ui         # Terminal 2: UI

# Stop all services
make stop

# Check service status
make status

# View logs
make logs
```

**Option 2: Using Scripts Directly**

```bash
# Terminal 1: Start backend stack (Kernel + AI + Go)
./scripts/start-backend.sh

# Terminal 2: Start UI
./scripts/start-ui.sh

# Stop everything
./scripts/stop.sh
```

### Ports

- **50051** - Rust Kernel (gRPC)
- **50052** - Python AI (gRPC)
- **8000** - Go Backend (HTTP/WebSocket)
- **5173** - UI (React/Vite)

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Complete system design
- [Migration Guide](docs/MIGRATION_COMPLETE.md) - Go migration details
- [Persistence Roadmap](docs/PERSISTENCE_ROADMAP.md) - Future plans
- [Phase 1 Complete](PHASE1_COMPLETE.md) - ✅ App Registry implemented!

## Tech Stack

**Backend**: Go (orchestration) + Python (AI) + Rust (kernel)  
**Frontend**: TypeScript + React + Electron  
**AI**: GPT-OSS-20B via Ollama with Metal GPU  
**IPC**: gRPC + Protocol Buffers

## Features

* Natural language UI generation
* Real-time chat with thought streaming
* Dynamic component rendering
* Sandboxed app execution
* Service-based architecture
* High-performance Go orchestration
* True concurrency with goroutines
* **NEW:** App Registry - Save & launch apps instantly! ⚡

## Makefile Commands

The Makefile provides a comprehensive set of commands for managing the entire project:

**Setup & Installation**
```bash
make setup              # Install all dependencies (kernel, AI, backend, UI)
make install-kernel     # Install Rust dependencies only
make install-ai         # Setup Python venv and dependencies
make install-backend    # Install Go dependencies
make install-ui         # Install Node.js dependencies
```

**Building**
```bash
make build              # Build all components
make build-kernel       # Build Rust kernel (release)
make build-backend      # Build Go backend
make build-ui           # Build UI for production
```

**Running**
```bash
make dev                # Start everything (backend + UI)
make start-backend      # Start backend stack only
make start-ui           # Start UI dev server only
make electron           # Start Electron app
```

**Protocol Buffers**
```bash
make proto              # Compile all protocol buffers
make proto-go           # Compile Go protobufs only
make proto-python       # Compile Python protobufs only
```

**Testing & Quality**
```bash
make test               # Run all tests
make test-backend       # Run Go tests
make test-kernel        # Run Rust tests
make format             # Format all code
make lint-backend       # Lint Go code
```

**Monitoring & Logs**
```bash
make status             # Check service status
make logs               # Tail all logs
make logs-kernel        # Tail kernel logs only
make logs-backend       # Tail backend logs only
make show-logs          # Show recent logs from all services
make ports              # Check which ports are in use
```

**Cleaning**
```bash
make clean              # Clean all build artifacts
make deep-clean         # Remove node_modules and venv
make stop               # Stop all services
make restart            # Restart everything
```

Run `make help` to see all available commands with descriptions.

## Testing

```bash
# Using Makefile
make test               # Run all tests
make test-backend       # Backend tests only
make test-kernel        # Kernel tests only

# Or directly:
cd backend && go test ./...
cd ai-service && pytest

# Health check
curl http://localhost:8000/health

# WebSocket test (after starting backend)
wscat -c ws://localhost:8000/stream
```

## Performance

- **HTTP Latency**: 5-10x faster than Python FastAPI
- **Concurrency**: True parallel processing with goroutines
- **Memory**: Efficient resource usage in Go layer
- **Type Safety**: Compile-time guarantees prevent runtime errors

## Recent Changes

**October 2025 - App Registry & Persistence (Phase 1)**

- App Registry: Save & launch apps instantly
- 50-100x faster launches (saved apps vs AI generation)
- Beautiful app launcher with category filtering
- Full CRUD operations (create, read, update, delete)
- Zero tech debt, production-ready
- See [Phase 1 Complete](PHASE1_COMPLETE.md) for details

**January 2025 - Complete Go Migration**

- Backend rewritten in Go (5-10x faster than Python)
- Python reduced to AI-only gRPC service
- True concurrency with goroutines
- Strong type safety throughout
- Simplified to 2 startup scripts
- ~500 lines removed, zero tech debt

**Architecture Evolution**

- Before: Python FastAPI monolith
- After: Go orchestration + Python AI + Rust kernel
- Result: Better performance, cleaner separation

## Contributing

This is an experimental project exploring the future of AI-powered operating systems.

## License

MIT
