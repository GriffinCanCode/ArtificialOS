# AgentOS

A microkernel-based operating system with AI-native application generation and execution.

![AgentOS Interface](assets/Screenshot%202025-10-03%20at%205.35.35%20PM.png)

## Overview

AgentOS implements a novel architecture where applications are generated from natural language descriptions and executed through a structured tool system. The system uses a four-layer architecture with specialized components for orchestration, AI inference, kernel operations, and dynamic rendering.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (TypeScript/React)                                │
│  - Dynamic UI rendering from JSON specifications            │
│  - Local tool execution (<10ms)                             │
│  - Component state management                               │
│  - WebSocket streaming                                      │
└────────────────┬────────────────────────────────────────────┘
                 │ HTTP/WebSocket (Port 5173)
┌────────────────▼────────────────────────────────────────────┐
│  Backend Orchestration (Go)                                 │
│  - App lifecycle management                                 │
│  - Service registry                                         │
│  - Session persistence                                      │
│  - gRPC client coordination                                 │
└────────────────┬────────────────────────────────────────────┘
                 │ gRPC
        ┌────────┴─────────┐
        │                  │
        ▼                  ▼
┌──────────────┐  ┌──────────────────┐
│ AI Service   │  │ Rust Kernel      │
│ (Python)     │  │ (Microkernel)    │
│              │  │                  │
│ - LLM        │  │ - Process mgmt   │
│ - UI gen     │  │ - Memory mgmt    │
│ - Streaming  │  │ - Sandboxing     │
│              │  │ - Syscalls       │
│ Port 50052   │  │ Port 50051       │
└──────────────┘  └──────────────────┘
```

## System Components

### Backend Orchestration Layer (Go)

The Go backend serves as the central orchestration hub, managing application lifecycle, routing requests, and coordinating between services.

**Key Responsibilities:**
- HTTP/REST API and WebSocket server
- Application lifecycle management (spawn, focus, close)
- App registry for persistent application storage
- Session management for workspace persistence
- Service registry for tool discovery and execution
- gRPC client coordination with AI service and kernel

**Core Modules:**
- `app.Manager`: Tracks running applications and their state
- `registry.Manager`: Persists application definitions to filesystem via kernel
- `session.Manager`: Saves and restores entire workspaces
- `grpc.AIClient`: Communicates with Python AI service
- `grpc.KernelClient`: Executes syscalls through Rust kernel
- `ws.Handler`: Streams real-time updates to frontend

### AI Service Layer (Python)

Isolated Python service focused exclusively on LLM operations via gRPC.

**Key Responsibilities:**
- LLM inference using Google Gemini API (gemini-2.0-flash-exp)
- UI specification generation from natural language
- Token-level streaming for real-time updates
- Chat response generation with thought streaming
- UI caching for performance optimization

**Core Components:**
- `UIGeneratorAgent`: Generates structured JSON UI specifications
- `ChatAgent`: Handles conversational interactions
- `ModelLoader`: Manages LLM loading and inference
- `UICache`: Caches frequently requested UI patterns
- `ToolRegistry`: Defines available tools and their schemas

### Kernel Layer (Rust)

Lightweight microkernel providing sandboxed system operations.

**Key Responsibilities:**
- Process creation and lifecycle management
- Memory allocation with OOM (Out of Memory) handling
- Capability-based sandboxing (filesystem, network, process access)
- System call execution with permission checking
- Resource limits enforcement per process
- IPC (Inter-Process Communication) management

**Core Subsystems:**
- `ProcessManager`: Creates and tracks processes with resource limits
- `MemoryManager`: Allocates memory with pressure monitoring
- `SandboxManager`: Enforces capability-based security policies
- `SyscallExecutor`: Executes filesystem, process, and system operations
- `IPCManager`: Handles message passing between processes

### Frontend Layer (TypeScript/React)

Dynamic rendering engine that executes AI-generated applications.

**Key Responsibilities:**
- Parse and render JSON UI specifications
- Execute local tools with sub-10ms latency
- Manage per-app component state
- Handle WebSocket streaming for real-time updates
- Provide app registry UI for saved applications
- Session management interface

**Core Modules:**
- `DynamicRenderer`: Main rendering engine with virtual scrolling
- `ComponentRenderer`: Renders individual UI components
- `ToolExecutor`: Executes tools (calc, ui, system, app, http, etc.)
- `ComponentState`: Observable state management per application
- `AppStore`: Zustand-based global state with selectors
- `WebSocketContext`: Manages streaming connections

## Quick Start

### Prerequisites

* Go 1.21+
* Rust 1.70+
* Python 3.11+
* Node.js 18+
* Google API Key (for Gemini API) - Set as `GOOGLE_API_KEY` environment variable

### Setup & Running

**Configure API Key:**

Create a `.env` file in the `ai-service/` directory:
```bash
GOOGLE_API_KEY=your_api_key_here
```

The `start-backend.sh` script will automatically load this environment variable.

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

## API Reference

### HTTP Endpoints

**Health & Status**
- `GET /` - Basic health check
- `GET /health` - Detailed health with system statistics

**Application Management**
- `GET /apps` - List all running applications
- `POST /apps/:id/focus` - Bring application to foreground
- `DELETE /apps/:id` - Close application and children

**Service Management**
- `GET /services` - List available services
- `POST /services/discover` - Discover services for intent
- `POST /services/execute` - Execute service tool

**AI Operations**
- `POST /generate-ui` - Generate UI specification (non-streaming)
- `GET /stream` - WebSocket endpoint for streaming operations

**App Registry**
- `POST /registry/save` - Save application to registry
- `GET /registry/apps` - List saved applications
- `GET /registry/apps/:id` - Get application details
- `POST /registry/apps/:id/launch` - Launch saved application
- `DELETE /registry/apps/:id` - Delete saved application

**Session Management**
- `POST /sessions/save` - Save current workspace
- `POST /sessions/save-default` - Save with default name
- `GET /sessions` - List saved sessions
- `GET /sessions/:id` - Get session details
- `POST /sessions/:id/restore` - Restore saved session
- `DELETE /sessions/:id` - Delete session

### WebSocket Protocol

**Client to Server Messages:**
```json
{"type": "chat", "message": "...", "context": {...}}
{"type": "generate_ui", "message": "...", "context": {...}}
{"type": "ping"}
```

**Server to Client Messages:**
```json
{"type": "token", "content": "..."}
{"type": "thought", "content": "..."}
{"type": "ui_complete", "ui_spec": {...}, "app_id": "..."}
{"type": "error", "error": "..."}
```

## Documentation

- [Architecture Details](docs/ARCHITECTURE.md) - Comprehensive system design
- [Migration Guide](docs/MIGRATION_COMPLETE.md) - Go migration technical details
- [OOM Handling](docs/OOM_HANDLING_IMPROVEMENTS.md) - Memory management improvements
- [Persistence Roadmap](docs/PERSISTENCE_ROADMAP.md) - Future persistence features

## Core Architecture: Generate-Once-Execute-Many

AgentOS implements a generate-once-execute-many pattern that fundamentally separates AI generation from application execution.

### Application Lifecycle

**Generation Phase (One-Time, ~2-5s)**
```
1. User: "create a calculator"
2. Go Backend → AI Service (gRPC)
3. LLM generates structured JSON UISpec
4. Spec includes components + tool bindings
5. Backend stores app state
6. Frontend receives complete specification
```

**Execution Phase (Repeated, <10ms per interaction)**
```
1. User clicks button (e.g., "7")
2. Button's on_event handler triggers: "calc.append_digit"
3. ToolExecutor executes locally
4. ComponentState updates
5. React re-renders affected components
```

### Comparison

**Traditional AI Approach:**
- Every interaction requires LLM inference
- 2-5 seconds per button click
- High token cost per interaction
- Non-deterministic behavior

**AgentOS Approach:**
- LLM generates once, tools execute many times
- Sub-10ms response after initial generation
- Single token cost per application
- Deterministic tool execution
- No network latency for interactions

### UISpec Format

The AI service generates structured JSON specifications that define the entire application:

```json
{
  "type": "app",
  "title": "Calculator",
  "layout": "vertical",
  "components": [
    {
      "type": "input",
      "id": "display",
      "props": {"value": "0", "readonly": true}
    },
    {
      "type": "button",
      "id": "btn-7",
      "props": {"text": "7"},
      "on_event": {"click": "calc.append_digit"}
    }
  ],
  "services": ["calc"],
  "service_bindings": {
    "calc": ["append_digit", "add", "subtract", "multiply", "divide"]
  }
}
```

### Tool Execution System

The frontend implements a comprehensive tool execution engine with multiple categories:

**Calculation Tools** (`calc.*`)
- Arithmetic operations (add, subtract, multiply, divide)
- Scientific functions (sqrt, power, sin, cos, etc.)
- Number formatting and validation

**UI Tools** (`ui.*`)
- State management (set_state, get_state)
- Component manipulation (show, hide, enable, disable)
- Dynamic updates (add_item, remove_item)

**System Tools** (`system.*`)
- Alerts and confirmations
- Clipboard operations
- Browser APIs

**App Tools** (`app.*`)
- Spawn new applications
- Close applications
- Focus/unfocus management

**HTTP Tools** (`http.*`)
- RESTful API requests
- WebSocket connections
- Response handling

**Service Tools** (Dynamic)
- Storage operations
- Authentication
- Custom backend services

### Persistence Architecture

**App Registry**
- Stores generated UI specifications
- Enables instant app launches (50-100x faster than generation)
- Uses kernel filesystem syscalls for persistence
- Supports categories, metadata, and versioning

**Session Management**
- Captures complete workspace state
- Saves all running apps and their component states
- Preserves chat history and UI state
- Enables restore from any saved point

## Technology Stack

**Languages:** Go, Python, Rust, TypeScript

**Backend Orchestration:**
- Go 1.21+ with Gin web framework
- Goroutines for concurrent app management
- gRPC clients for service communication

**AI Service:**
- Python 3.11+ with async/await
- Google Gemini API for LLM inference
- LangChain for prompt management
- Pydantic for structured outputs
- gRPC for service communication

**Kernel:**
- Rust 1.70+ with Tokio async runtime
- Tonic for gRPC server
- Parking lot for synchronization
- Crossbeam for IPC

**Frontend:**
- React 18 with TypeScript
- Zustand for state management
- React Spring + GSAP for animations
- TanStack Query for data fetching
- Tailwind CSS with CVA patterns
- WebSockets for real-time streaming

**Inter-Process Communication:**
- gRPC with Protocol Buffers
- Bidirectional streaming
- Type-safe generated code

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

## Performance Characteristics

### Backend Performance
- **Request Handling**: 5-10x faster than equivalent Python FastAPI implementation
- **Concurrency**: True parallel processing with goroutines; handles multiple apps simultaneously
- **Memory**: Efficient allocation with Go's garbage collector
- **Type Safety**: Compile-time type checking prevents entire classes of runtime errors

### AI Service Performance
- **Inference**: Google Gemini API with cloud-based optimization
- **Streaming**: Token-level streaming for real-time user feedback
- **Caching**: LRU cache for frequently requested UI specifications
- **Model**: gemini-2.0-flash-exp for fast, high-quality responses

### Kernel Performance
- **Syscall Latency**: Sub-millisecond syscall execution through gRPC
- **Memory Management**: Proactive OOM detection with graceful degradation
- **Sandboxing**: Zero-copy capability checks for minimal overhead
- **Process Isolation**: Lightweight process tracking without OS-level isolation

### Frontend Performance
- **Tool Execution**: Sub-10ms local tool execution
- **Rendering**: Virtual scrolling for apps with 1000+ components
- **State Management**: Selective Zustand subscriptions prevent unnecessary re-renders
- **Animations**: Hardware-accelerated CSS and GSAP animations
- **Bundle Size**: Code splitting and lazy loading for optimal load times

## System Capabilities

### Multi-Application Management
- Concurrent execution of multiple AI-generated applications
- Parent-child application relationships
- Focus management with foreground/background states
- Graceful cleanup of child applications when parent closes

### Persistence Layer
- **App Registry**: Store and instantly launch generated applications (50-100x faster than regeneration)
- **Session Management**: Save and restore complete workspace state
- **Filesystem Integration**: All persistence goes through kernel syscalls
- **Structured Storage**: JSON-based storage with metadata support

### Security Model
- **Capability-Based Sandboxing**: Processes request specific capabilities (filesystem, network, process)
- **Resource Limits**: Per-process memory limits with OOM protection
- **Permission Checking**: All syscalls verified against process capabilities
- **No Arbitrary Code Execution**: UI specs are pure data, tools are pre-defined functions

### Extensibility
- **Service Registry**: Dynamic service discovery and tool binding
- **Tool System**: Extensible tool categories with parameter validation
- **Component System**: Pluggable UI components with CVA variants
- **Protocol Buffers**: Versioned, type-safe service definitions

## License

MIT License - see LICENSE file for details

## Acknowledgments

This project builds upon established technologies:
- Google Gemini API for efficient LLM inference
- gRPC for high-performance RPC
- Tokio for async Rust runtime
- React ecosystem for dynamic UIs
