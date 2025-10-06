# AgentOS

A microkernel-based operating system with AI-native application generation and execution.

![AgentOS Interface](assets/demo.png)

## Overview

AgentOS implements a userspace OS where applications are generated from natural language descriptions and executed through a structured tool system. The system uses a four-layer architecture with specialized components for orchestration, AI inference, kernel operations, and dynamic rendering.

**Implementation Status:** Production-ready userspace OS with 63 syscalls, multi-window management, OS-level process execution, CPU scheduling, advanced IPC (pipes + shared memory), and virtual filesystem. Core infrastructure ~90% complete.

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
- Application lifecycle management (spawn, focus, close, window state)
- Blueprint DSL (.bp file) parsing and prebuilt app seeding
- App registry for persistent application storage
- Session management for workspace persistence
- Service registry for tool discovery and execution
- Middleware layer (CORS, rate limiting)
- gRPC client coordination with AI service and kernel

**Core Modules:**
- `app.Manager`: Tracks running applications and their state
- `registry.Manager`: Persists application definitions to filesystem via kernel
- `registry.Seeder`: Loads prebuilt apps from `/apps` directory
- `blueprint.Parser`: Parses Blueprint DSL (.bp) files
- `session.Manager`: Saves and restores entire workspaces
- `middleware.RateLimit`: Per-IP rate limiting with token bucket algorithm
- `middleware.CORS`: Cross-origin resource sharing configuration
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
- `BlueprintParser`: Parses Blueprint DSL into Package format
- `ChatAgent`: Handles conversational interactions
- `ModelLoader`: Manages LLM loading and inference
- `UICache`: Caches frequently requested UI patterns
- `ToolRegistry`: Modular tool system with 80+ tools across 5 categories (UI, app, system, math, network)

### Kernel Layer (Rust)

Production-ready microkernel with OS-level process execution, advanced IPC, and virtual filesystem.

**Key Responsibilities:**
- Process creation with true OS execution (std::process::Command)
- CPU scheduling with multiple policies (round-robin, priority, CFS-inspired fair)
- Memory allocation with OOM handling and garbage collection
- Capability-based sandboxing with resource limits (memory, CPU, process spawns)
- Virtual filesystem with pluggable backends (local, in-memory)
- Advanced IPC with pipes and shared memory (zero-copy transfers)
- System call execution with 63 fully implemented syscalls

**Core Subsystems:**
- `ProcessManager`: OS process spawning with ExecutionConfig (command, args, env, working dir)
- `ProcessExecutor`: Shell injection prevention, security validation, zombie cleanup
- `Scheduler`: 3 scheduling policies with configurable time quantum and preemption
- `MemoryManager`: Per-process tracking with pressure monitoring and OOM detection
- `SandboxManager`: 14 capability types with per-process resource limits
- `VFSManager`: Mount manager with LocalFS and MemFS backends, 18 filesystem operations
- `IPCManager`: Unix-style pipes (64KB buffer) + shared memory segments (100MB per segment)
- `SyscallExecutor`: 63 syscalls across 12 categories (filesystem, process, IPC, scheduler, memory, network, signals)

### Frontend Layer (TypeScript/React)

Dynamic rendering engine with production-ready window management and sub-10ms tool execution.

**Key Responsibilities:**
- Parse and render JSON UI specifications (23 component types)
- Execute local tools with sub-10ms latency
- Desktop-grade window management (react-rnd powered)
- Per-app component state with observable updates
- WebSocket streaming for real-time AI responses
- Keyboard shortcuts and gesture handling
- App registry and session management UI

**Core Modules:**
- `DynamicRenderer`: Main rendering engine with virtual scrolling and modular architecture
- `ComponentRegistry`: 23 registered components across 6 categories (primitives, layout, forms, media, ui, special)
- `WindowManager`: Production-ready multi-window system with backend state synchronization
- `Window`: Drag, resize, maximize, minimize with snap-to-edge positioning (9 zones)
- `WindowStore`: Zustand store with full window lifecycle (open, close, focus, minimize, restore, snap)
- `ToolExecutor`: 10+ tool categories with validation and error handling
- `ComponentState`: Observable state management per application
- `InputHandler`: Centralized keyboard, mouse, touch, and gesture handling with Zod validation
- `WebSocketContext`: Manages streaming connections with reconnection logic

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
- `POST /apps/:id/window` - Update window state (position, size, minimized, maximized)
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
- [Blueprint DSL](docs/BLUEPRINT_DSL.md) - Blueprint specification and syntax
- [Desktop System](docs/DESKTOP_SYSTEM.md) - Window management architecture
- [Prebuilt Apps](docs/PREBUILT_APPS.md) - Creating and loading prebuilt applications

## Blueprint DSL

AgentOS uses Blueprint - a JSON-based domain-specific language for defining applications. Applications can be created in two ways:

1. **AI Generation**: Natural language → LLM generates Blueprint JSON
2. **Manual Definition**: Write `.bp` files directly and drop in `/apps` directory

### Why Blueprint?

**Streaming-First Design:**
- Components render incrementally as they're generated
- Explicit JSON structure enables real-time parsing during token streaming
- No special syntax in keys - just clean `type`, `id`, `props` fields

**Example Blueprint:**
```json
{
  "app": {
    "id": "calculator",
    "name": "Calculator",
    "icon": "🧮",
    "category": "utilities",
    "permissions": ["STANDARD"]
  },
  "services": [],
  "ui": {
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
        "on_event": {"click": "ui.append"}
      }
    ]
  }
}
```

### Prebuilt Apps

Drop `.bp` files in the `/apps` directory:
```
apps/
├── creative/
├── productivity/
│   └── notes.bp
├── system/
│   ├── file-explorer.bp
│   ├── hub.bp
│   └── task-manager.bp
└── utilities/
```

The system automatically:
1. Discovers all `.bp` files on startup
2. Parses and validates Blueprint format
3. Registers apps in the app registry
4. Makes them instantly launchable (sub-100ms vs 2-5s for AI generation)

Default system apps (calculator, settings, app-launcher) are automatically seeded if not present.

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

### Component System

The frontend provides 23 registered components across 6 categories, all with Zod validation:

**Primitives** (6 components)
- `button` - Clickable buttons with variants (primary, outline, ghost, danger)
- `input` - Text inputs (text, email, password, number)
- `text` - Text and headings (h1, h2, h3, body, caption, label)
- `checkbox` - Checkbox with label
- `radio` - Radio button selection
- `slider` - Range slider input

**Layout** (3 components)
- `container` - Flexbox container (row/col shortcuts available)
- `grid` - Responsive grid layout
- `list` - Styled lists (default, bordered, striped)

**Forms** (2 components)
- `select` - Dropdown selection
- `textarea` - Multi-line text input

**Media** (4 components)
- `image` - Image display
- `video` - Video player
- `audio` - Audio player
- `canvas` - HTML5 canvas for drawing/games

**UI** (5 components)
- `badge` - Status badges (success, warning, error, info)
- `card` - Card container with header/body
- `divider` - Visual separator
- `modal` - Popup dialog
- `tabs` - Tabbed interface

**Special** (3 components)
- `app_shortcut` - Launch other apps
- `iframe` - Embed external content
- `progress` - Progress bar

All components use a registry-based architecture with automatic registration, making it easy to add new component types.

### Tool Execution System

The system implements a comprehensive tool execution engine with 80+ tools across multiple categories:

**UI Tools** (`ui.*`)
- State management (set, get, append, clear, toggle, backspace, compute)
- Component manipulation (show, hide, enable, disable)
- Dynamic updates (add_item, remove_item)
- Generic operations work for ALL app types

**Math Tools** (`math.*` - 80+ tools via AI service)
- **Arithmetic** (24 tools): add, subtract, multiply, divide, power, sqrt, log, factorial, etc.
- **Trigonometry** (13 tools): sin, cos, tan, asin, acos, atan, sinh, cosh, etc.
- **Statistics** (15 tools): mean, median, mode, stdev, variance, percentile, correlation
- **Algebra** (11 tools): solve, factor, expand, simplify, matrix operations
- **Calculus** (9 tools): derivative, integrate, limit, series, taylor
- **Constants** (6): pi, e, tau, phi, infinity, nan

**App Tools** (`app.*`)
- Spawn new applications
- Close applications
- Focus/unfocus management
- List running apps

**System Tools** (`system.*`)
- Alerts, confirmations, notifications
- Clipboard operations (copy, paste)
- Timer operations (start, stop, reset)
- Browser APIs

**HTTP Tools** (`http.*`)
- RESTful API requests (get, post, request)
- Response handling
- Web content fetching

**Hub Tools** (`hub.*`)
- App launcher integration
- Registry management

**Service Tools** (Backend-integrated)
- **Storage**: Persistent key-value store (set, get, remove, list, clear)
- **Filesystem**: File operations (read, write, create, delete, list, move, copy)
- **System**: System info and logging (info, time, log, getLogs, ping)
- **Auth**: User authentication (register, login, logout, verify, getUser)

### Persistence Architecture

**Blueprint DSL (.bp files)**
- JSON-based domain-specific language for defining applications
- Streaming-first architecture for real-time component rendering
- Explicit format optimized for LLM generation and incremental parsing
- Supports templates, service bindings, and lifecycle hooks
- Located in `/apps` directory with automatic seeding on startup

**Prebuilt Apps**
- System automatically loads `.bp` and `.aiapp` files from `/apps` directory
- Organized by category (creative/, productivity/, system/, utilities/)
- Default apps (calculator, settings, app-launcher) seeded if not present
- Instant launch without AI generation (sub-100ms)

**App Registry**
- Stores generated UI specifications
- Enables instant app launches (50-100x faster than generation)
- Uses kernel filesystem syscalls for persistence
- Supports categories, metadata, and versioning

**Session Management**
- Captures complete workspace state
- Saves all running apps and their component states
- Preserves window positions, sizes, and states
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
- **Syscall Latency**: Sub-millisecond syscall execution through gRPC (63 syscalls fully implemented)
- **OS Process Execution**: Native process spawning with command validation and zombie cleanup
- **CPU Scheduling**: Configurable policies with 10ms default quantum and preemptive multitasking
- **Memory Management**: Proactive OOM detection with per-process tracking and garbage collection
- **IPC Throughput**: 
  - Pipes: 64KB buffer, 50MB global limit, streaming data transfer
  - Shared Memory: Zero-copy transfers, 100MB per segment, 500MB global limit
- **VFS Operations**: Mount manager routes operations to correct backend (LocalFS, MemFS)
- **Sandboxing**: Zero-copy capability checks for minimal overhead
- **Resource Limits**: cgroups v2 enforcement on Linux (memory, CPU shares, max PIDs)

### Frontend Performance
- **Tool Execution**: Sub-10ms local tool execution
- **Rendering**: Virtual scrolling for apps with 1000+ components
- **State Management**: Selective Zustand subscriptions prevent unnecessary re-renders
- **Animations**: Hardware-accelerated CSS and GSAP animations
- **Bundle Size**: Code splitting and lazy loading for optimal load times

## System Capabilities

### Desktop-Grade Window Management
- **Production-Ready Implementation**: Powered by react-rnd library with full drag/resize/focus
- **Drag & Drop**: Free-form window dragging with smooth animations and visual feedback
- **Snap-to-Edge**: Automatic window snapping to screen edges and corners (9 snap zones)
- **Resize**: Interactive window resizing from all edges and corners with min/max constraints
- **Minimize/Maximize**: Full window state management with smooth transitions
- **Backend Synchronization**: Window positions and sizes synced to Go backend via `POST /apps/:id/window`
- **Session Restoration**: Window geometry captured in sessions and restored on load
- **Keyboard Shortcuts**: 
  - `⌘K` / `Ctrl+K` - Spotlight-style app creator
  - `Alt+Tab` - Cycle through open windows
  - `⌘W` / `Ctrl+W` - Close focused window
  - `⌘M` / `Ctrl+M` - Minimize focused window
- **Cascade Positioning**: Automatic cascading for new windows with offset calculation
- **Z-Index Management**: Automatic focus and bring-to-front on interaction
- **Dual-Mode Architecture**: WindowManager for windowed apps + DynamicRenderer for fullscreen (backward compatible)

### Multi-Application Management
- Concurrent execution of multiple AI-generated applications
- Parent-child application relationships
- Focus management with foreground/background states
- Graceful cleanup of child applications when parent closes
- Desktop environment with menu bar, dock, and taskbar

### Persistence Layer
- **Blueprint DSL**: Define apps in `.bp` files with streaming-optimized JSON format
- **Prebuilt Apps**: Auto-load apps from `/apps` directory on startup
- **App Registry**: Store and instantly launch generated applications (50-100x faster than regeneration)
- **Session Management**: Save and restore complete workspace state (apps, windows, positions, sizes)
- **Filesystem Integration**: All persistence goes through kernel syscalls
- **Structured Storage**: JSON-based storage with metadata support

### Security Model
- **Capability-Based Sandboxing**: Processes request specific capabilities (14 types: filesystem, network, process, IPC, memory, scheduler)
- **Resource Limits**: Per-process memory, CPU shares (cgroups v2 on Linux), and process spawn limits
- **OS Process Isolation**: True process isolation with std::process::Command and security validation
- **Shell Injection Prevention**: Command validation blocks dangerous characters (;, |, &, `, $, etc.)
- **Path Restrictions**: Allowed/blocked path lists enforced at VFS mount level
- **Permission Checking**: All 63 syscalls verified against sandbox capabilities before execution
- **Rate Limiting**: Per-IP token bucket algorithm (configurable RPS and burst)
- **CORS Configuration**: Configurable cross-origin policies
- **No Arbitrary Code Execution**: UI specs are pure data, tools are pre-defined functions
- **Automatic Cleanup**: Zombie processes cleaned up, IPC resources freed on process termination

### Extensibility
- **Blueprint DSL**: Extensible component and service definitions via `.bp` files
- **Prebuilt Apps**: Drop `.bp` files in `/apps` directory for automatic loading
- **Service Registry**: Dynamic service discovery and tool binding
- **Tool System**: 80+ modular tools across 10+ categories with parameter validation
- **Component System**: 23 pluggable UI components across 6 categories with Zod validation
- **VFS Architecture**: Pluggable filesystem backends (LocalFS, MemFS) with trait-based design
- **Scheduler Policies**: 3 configurable policies (round-robin, priority, fair) with customizable quantum
- **IPC Mechanisms**: Multiple communication methods (message queues, pipes, shared memory)
- **Modular Architecture**: Registry-based component and tool registration
- **Middleware Stack**: Extensible HTTP middleware (CORS, rate limiting, auth-ready)
- **Protocol Buffers**: Versioned, type-safe service definitions (63 syscalls, 12 categories)

## License

MIT License - see LICENSE file for details

## Acknowledgments

This project builds upon established technologies:
- Google Gemini API for efficient LLM inference
- gRPC for high-performance RPC
- Tokio for async Rust runtime
- React ecosystem for dynamic UIs
