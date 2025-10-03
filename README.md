# AI-Powered Operating System

A lightweight operating system powered by local AI (GPT-OSS) with real-time thought streaming and dynamic application rendering.

## 🏗️ Architecture

```
Frontend (React/TS) → Go Backend → Python AI (gRPC) [LLM]
         ↓                    ↓
     Port 5173          Rust Kernel [Syscalls]
                        Port 50051
```

## 📁 Project Structure

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
- **50052** - Python AI (gRPC)
- **8000** - Go Backend (HTTP/WebSocket)
- **5173** - UI (React/Vite)

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md) - Complete system design
- [Migration Guide](docs/MIGRATION_COMPLETE.md) - Go migration details
- [Persistence Roadmap](docs/PERSISTENCE_ROADMAP.md) - Future plans

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
# Backend tests (Go)
cd backend && go test ./...

# AI service tests (Python)
cd ai-service && pytest

# Health check
curl http://localhost:8000/health

# WebSocket test (after starting backend)
wscat -c ws://localhost:8000/stream
```