# AI-Powered Operating System

A lightweight operating system powered by local AI (GPT-OSS) with real-time thought streaming and dynamic application rendering.

## 🏗️ Architecture

```
Frontend (TS/React) → Go Service → Python AI (gRPC) [LLM Only]
                              ↓
                         Rust Kernel [Syscalls]
```

## 📁 Project Structure

```
os/
├── go-service/      # Go orchestration layer (NEW!)
│   ├── cmd/         # Main entry point
│   ├── internal/    # App management, services, HTTP, WebSocket
│   └── proto/       # Generated protobuf code
├── ai-service/      # Python AI service (gRPC - LLM only)
│   ├── src/         # Chat agent, UI generator, models
│   └── proto/       # AI service protobuf definitions
├── kernel/          # Rust microkernel (process, memory, sandbox)
├── ui/              # Electron/React dynamic UI renderer
├── proto/           # Shared protocol buffer definitions
├── scripts/         # System startup/shutdown scripts
├── docs/            # Architecture and migration docs
└── logs/            # Runtime logs (gitignored)
```

## 🚀 Quick Start

### Prerequisites

* **Go** 1.21+
* **Rust** 1.70+
* **Python** 3.11+
* **Node.js** 18+

### Setup & Running

**Simple 2-Script System**

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
- **50052** - Python AI Service (gRPC)
- **8000** - Go Service (HTTP/WebSocket)
- **5173** - UI (Vite dev server)

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md) - Complete system design
- [Migration Guide](docs/MIGRATION_COMPLETE.md) - Go migration details
- [Persistence Roadmap](docs/PERSISTENCE_ROADMAP.md) - Future plans

## ✅ Current Status

### Phase 1: Foundation (Complete)
- [x] Rust kernel with process management & sandboxing
- [x] Python AI service with LLM integration
- [x] React/Electron UI with dynamic rendering
- [x] gRPC communication (Kernel ↔ Services)

### Phase 2: Go Migration (Complete) 🎉
- [x] **Go orchestration layer** - HTTP, WebSocket, app management
- [x] **Python AI reduced to gRPC** - LLM operations only
- [x] **Type-safe architecture** - Strong typing throughout
- [x] **Concurrent app management** - Goroutines + sync.Map
- [x] **Service registry** - Discovery and execution
- [x] **Comprehensive tests** - Go test coverage
- [x] **Dead code removed** - Archived old FastAPI code

### Tech Stack
* **Backend**: Go (orchestration) + Python (AI) + Rust (kernel)
* **Frontend**: TypeScript + React + Electron
* **AI**: GPT-OSS-20B via Ollama with Metal GPU
* **IPC**: gRPC + Protocol Buffers

### Features
* 🤖 Natural language UI generation
* 💬 Real-time chat with thought streaming
* 🎨 Dynamic component rendering
* 🔒 Sandboxed app execution
* 📦 Service-based architecture
* ⚡ High-performance Go orchestration
* 🧵 True concurrency with goroutines

## 🧪 Testing

```bash
# Go tests
cd go-service && go test ./...

# Python tests
cd ai-service && pytest

# Integration test
curl http://localhost:8000/health
```

## 🤝 Contributing

This is an experimental project exploring the future of AI-powered operating systems.

## 📊 Performance

- **HTTP Latency**: 5-10x faster than Python FastAPI
- **Concurrency**: True parallel processing with goroutines
- **Memory**: Efficient resource usage in Go layer
- **Type Safety**: Compile-time guarantees prevent runtime errors

## 🔄 Recent Changes

**January 2025 - Go Migration**
- Migrated orchestration layer from Python to Go
- Reduced Python service to pure AI operations (gRPC)
- Improved performance and type safety
- Added comprehensive test coverage
- See [MIGRATION_COMPLETE.md](docs/MIGRATION_COMPLETE.md) for details
