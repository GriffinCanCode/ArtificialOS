# AgentOS

A modern desktop operating system built from scratch in userspace with a production-grade microkernel architecture — featuring true process orchestration, sophisticated IPC, network isolation, and a dynamic UI system that renders applications from JSON specifications or runs native code. Oh, and it can also generate applications with AI if you want.

![AgentOS Interface](assets/demo.png)

## Overview — A Real Desktop OS in Userspace

**What if you rebuilt a desktop OS from scratch with modern architecture?** That's AgentOS.

I spent way too much time thinking about operating systems and built something that shouldn't exist but does: a legitimate userspace microkernel with desktop environment, window management, full IPC stack, 95+ syscalls, and observability woven into the fabric from day one. It runs as an Electron app on top of your host OS, but underneath is a complete process orchestration system combining proven algorithms (CFS scheduling, segregated free lists, Unix IPC) with modern architecture patterns.

**What it is:** A four-layer desktop OS with Rust microkernel, Go backend services, Python AI service, and React/TypeScript desktop shell. Think of it as rebuilding macOS/Windows/Linux from first principles, but it runs in userspace.

## Architecture — Four Layers, One System

```
┌─────────────────────────────────────────────────────────────┐
│  Desktop Shell (TypeScript/React)                           │
│  - Window management (drag, resize, minimize, snap)         │
│  - Desktop environment (menu bar, dock, launcher)           │
│  - Dynamic UI rendering (Blueprint, Native Web, Native Proc)│
│  - Component state management (<10ms tool execution)        │
│  - WebSocket streaming for real-time updates               │
└────────────────┬────────────────────────────────────────────┘
                 │ HTTP/WebSocket (Port 5173)
┌────────────────▼────────────────────────────────────────────┐
│  System Services (Go)                                       │
│  - Application lifecycle (spawn, focus, close, persist)     │
│  - Service providers (filesystem, storage, network, auth)   │
│  - Session management (save/restore workspaces)             │
│  - Blueprint DSL parser and app registry                    │
│  - gRPC orchestration between kernel and AI service         │
└────────────────┬────────────────────────────────────────────┘
                 │ gRPC
        ┌────────┴─────────┐
        │                  │
        ▼                  ▼
┌──────────────┐  ┌────────────────────────┐
│ AI Service   │  │ Microkernel (Rust)     │
│ (Python)     │  │                        │
│              │  │                        │
│ - UI gen     │  │ - Process orchestration│
│ - LLM        │  │ - IPC (4 types)        │
│ - Streaming  │  │ - Scheduler (CFS)      │
│ - Templates  │  │ - VFS & Filesystem     │
│              │  │ - Security & Sandboxing│
│ Port 50052   │  │ - Syscalls (95+)       │
│              │  │ - Observability        │
│              │  │ Port 50051             │
└──────────────┘  └────────────────────────┘
```

**Your kernel pieces map to a desktop OS:**
- **Rust kernel** → The actual OS core (process management, IPC, scheduling, sandboxing)
- **Go backend** → System services layer (like systemd, launchd, or Windows Services)
- **TypeScript/React** → The desktop shell (like GNOME, KDE, or Windows Explorer)
- **Python AI** → Optional app generator (one feature among many)

## Desktop Environment — Because This Should Feel Like An OS

AgentOS includes a complete desktop environment with window management, application launcher, and system apps. This isn't a toy — it's designed to feel like a real desktop OS:

**Core Desktop Features:**
- **Window Management**: Full drag, resize, minimize, maximize with snap-to-edge positioning (9 snap zones)
- **Menu Bar**: Top bar with system menus, app name, and system controls
- **Dock/Taskbar**: Quick access to running applications and favorites
- **App Launcher**: Spotlight-style launcher (⌘K/Ctrl+K) for instant app search
- **Keyboard Shortcuts**: Alt+Tab for window switching, ⌘W to close, ⌘M to minimize
- **Desktop Icons**: Launch apps with double-click (coming soon)
- **Session Management**: Save and restore complete workspace state

**System Applications:**
- **File Manager**: Browse filesystem with tree view, file operations, search (showcase your VFS!)
- **Task Manager**: View processes, CPU, memory, IPC stats (showcase your ProcessManager!)
- **System Monitor**: Real-time kernel observability dashboard with causality tracking
- **Terminal**: Full shell integration for native process apps
- **Settings**: Configure appearance, permissions, performance, and developer options
- **App Store**: Browse and install applications from registry

**Why This Matters:**
Your kernel has 95 syscalls, four IPC types, network isolation, and sophisticated scheduling. Building system apps that **actually use** these features showcases what you've built. A file manager demonstrates your VFS. A task manager demonstrates your process orchestration. A system monitor demonstrates your observability infrastructure.

## The Pitch — What This Really Is

> **AgentOS: A Modern Desktop OS Built From Scratch**
>
> A userspace operating system with a production-grade microkernel architecture, 
> running as an Electron app. Features a complete desktop environment, three-tier
> application system (Blueprint, Native Web, Native Process), full process isolation,
> sophisticated IPC, and an extensible app ecosystem.
>
> **Built in Rust, Go, Python, and TypeScript.**
>
> **Core Features:**
> - ✅ True process orchestration with CFS-inspired scheduling
> - ✅ Four types of IPC (pipes, shared memory, async queues, mmap)
> - ✅ Network namespace isolation (Linux, macOS, simulation)
> - ✅ Observability-first architecture with adaptive sampling
> - ✅ Desktop environment with window management
> - ✅ Three application types (Blueprint, Native Web, Native Process)
> - ✅ 95+ syscalls across 13 categories
> - ✅ Dynamic UI rendering from JSON specifications
> - ✅ Optional AI-powered app generation
> - ✅ Session persistence and workspace restoration
>
> **Think of it as:** What if you rebuilt a desktop OS with modern architecture, 
> where AI generation is a feature, not the core?

## The Core Innovation: Observability Was Never An Afterthought

Here's what makes AgentOS different — and I say this having studied how Linux, Fuchsia, and others approached this problem — observability isn't bolted on. It's woven into the fabric from day one. Every major subsystem emits events through a unified collector, and I spent time making this both sophisticated and fast:

### Dual-Layer Observability System

**Layer 1: Distributed Tracing**
- Request-scoped spans across async boundaries
- Performance profiling with structured context
- JSON/human-readable log output
- Automatic span correlation

**Layer 2: Event Streaming**
- Lock-free 65,536-slot ring buffer (~50ns per event)
- Adaptive sampling maintaining <2% CPU overhead
- Welford's algorithm for streaming anomaly detection (O(1) memory)
- Causality tracking to link related events across subsystems
- Real-time query API without external tools

### Key Observability Features

**Adaptive Sampling:**
```rust
// Automatically adjusts to maintain target overhead (default 2%)
if current_overhead > TARGET {
    reduce_sampling_rate();
} else if current_overhead < TARGET {
    increase_sampling_rate();
}
```
- Xorshift RNG for fast sampling decisions (2-3 CPU cycles)
- Per-category sampling rates
- Automatic backpressure control

**Anomaly Detection:**
- Z-score based (3σ = 99.7% confidence)
- Constant memory usage via Welford's online variance
- Detects outliers in real-time without historical data storage

**Causality Tracking:**
```rust
let causality_id = collector.emit_causal(event1);
collector.emit_in_chain(event2, causality_id);
collector.emit_in_chain(event3, causality_id);
// Query entire chain later
```

### Comprehensive Event Coverage

Every major operation emits observable events:
- `SyscallExecutor` → syscall_enter/exit with timing
- `Scheduler` → context switches and policy changes  
- `MemoryManager` → allocations/deallocations with sizes
- `IPCManager` → message sends/receives with throughput
- `SandboxManager` → permission checks and denials
- `ProcessManager` → creation/termination with resource stats
- `TimeoutExecutor` → timeout events with retry counts

---

## System Components — The Four-Layer Stack

### System Services Layer (Go) — The Orchestrator

I chose Go for the system services layer for one simple reason: goroutines. When you're managing multiple applications simultaneously (whether AI-generated, native web apps, or OS processes), true parallel processing matters. The Go backend serves as the central orchestration hub, managing application lifecycle, routing requests, coordinating between services, and providing system-level services like filesystem operations, storage, and authentication — and doing it fast.

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

### AI Service Layer (Python) — The Optional Generator

Python gets a bad rap for performance, but for LLM orchestration? It's perfect. The entire AI service is isolated behind gRPC, so language choice doesn't matter for the overall system latency — and Python's ecosystem for AI is unmatched. **This layer is optional** — the system works perfectly fine with just prebuilt Blueprint apps and native applications. AI generation is a power user feature, not a requirement.

**Key Responsibilities:**
- UI specification generation (template-based with optional LLM enhancement)
- Token-level streaming for real-time updates
- Chat response generation with thought streaming
- UI caching for performance optimization
- Optional LLM inference using Google Gemini API (gemini-2.0-flash-exp) when you want AI-generated apps

**Core Components:**
- `UIGeneratorAgent`: Generates structured JSON UI specifications (rule-based + LLM)
- `BlueprintParser`: Parses Blueprint DSL into Package format
- `ChatAgent`: Handles conversational interactions
- `ModelLoader`: Manages LLM loading and inference
- `UICache`: Caches frequently requested UI patterns
- `ToolRegistry`: Modular tool system with 80+ tools across 5 categories (UI, app, system, math, network)

### Microkernel Layer (Rust) — The Heart of the OS

This is where I spent most of my time, and where I'm most proud of the work. Rust was the only choice here — memory safety without garbage collection overhead, fearless concurrency, and a type system that catches bugs at compile time. The result is a production-grade userspace microkernel that does what real operating systems do: manage processes, schedule execution, isolate resources, enforce security, and provide comprehensive IPC — all with observability-first architecture and performance optimizations that kept me up at night (in a good way).

**What makes this interesting:**
- **Observability-Native Design**: Dual-layer architecture (distributed tracing + event streaming) with adaptive sampling, Welford's algorithm for anomaly detection, causality tracking, and lock-free event streams (65K ring buffer, ~50ns/event)
- **Resource Orchestrator**: Unified trait-based cleanup system with dependency-aware ordering (LIFO), comprehensive statistics, and coverage validation - better orchestration than Linux process cleanup
- **JIT Syscall Compilation**: eBPF-inspired hot path detection and runtime optimization for frequently called syscalls
- **Timeout Infrastructure**: Microoptimized retry loops with adaptive backoff (spin → yield → sleep), pre-computed deadlines, and batch time checks achieving 7.5x speedup
- **io_uring-style Completion**: Lock-free submission/completion queues for async I/O with batched syscall execution

**Core Subsystems:**
- `ProcessManager`: OS process spawning with explicit state machines (Creating → Initializing → Ready) eliminating initialization races
- `ProcessExecutor`: Shell injection prevention, security validation, zombie cleanup via waitpid
- `Scheduler`: CFS-inspired fair scheduling with 3 policies (round-robin, priority, fair), O(1) location index, preemptive scheduling, and dynamic vruntime tracking
- `SchedulerTask`: Autonomous background task with event-driven control, dynamic quantum adaptation, and Tokio integration
- `MemoryManager`: Segregated free lists (12 power-of-2 + 15 linear buckets), block splitting, periodic coalescing, ID recycling to prevent u32 exhaustion
- `SandboxManager`: Granular capability system with path-specific permissions, TOCTOU-safe path handling, fine-grained network rules, permission caching (LRU + TTL), and cross-platform network namespace isolation (Linux namespaces, macOS packet filters, simulation fallback)
- `VFSManager`: Mount manager with pluggable backends (LocalFS, MemFS), 14 filesystem operations
- `IPCManager`: Unix-style pipes (64KB, lock-free SPSC) + shared memory (zero-copy, 100MB/segment) + async queues (FIFO/Priority/PubSub) + mmap + zero-copy IPC with io_uring semantics
- `SyscallExecutor`: 95+ syscalls across 13 categories with modular handler architecture
- `BatchExecutor`: Parallel/sequential batch syscall execution
- `StreamingExecutor`: Bidirectional streaming for large file operations
- `AsyncTaskManager`: Long-running syscall execution with progress tracking, cancellation, and TTL-based cleanup
- `SocketManager`: Full TCP/UDP socket implementation (socket, bind, listen, accept, connect, send, recv, sendto, recvfrom)
- `FdManager`: File descriptor management (open, close, dup, dup2, lseek, fcntl)
- `SignalManager`: POSIX-style signal handling (register handlers, block/unblock, pending signals, wait)
- `JitManager`: Hot path detection, pattern-based optimization, compiled handler caching
- `IoUringManager`: Submission/completion rings per process with async execution
- `TimeoutExecutor`: Generic timeout execution for all blocking operations
- `Collector`: Unified observability with event streaming, sampling, and anomaly detection

**Advanced gRPC Features:**

I spent time on the gRPC layer addressing architectural limitations that would have caused production issues. Three major enhancements:

**1. Streaming Syscalls (For Large Data Transfers)**

Problem: Large file operations (multi-GB) were single blocking RPC calls causing memory pressure, timeouts, and no progress feedback.

Solution: Bidirectional streaming with configurable chunk sizes:

```rust
// Kernel: Stream file read in 64KB chunks
pub async fn stream_file_read(
    path: &str,
    chunk_size: usize,
) -> impl Stream<Item = Result<Vec<u8>, String>> {
    // Memory efficient - only one chunk in memory
}
```

**Performance Impact:**
- Before: 1GB file = 30+ seconds, single blocking RPC, memory spike
- After: 1GB file = ~5 seconds, streaming, cancelable, constant memory

**2. Async Syscall Execution (For Long-Running Operations)**

Problem: Long-running syscalls (`sleep()`, `wait()`, IO-heavy operations) blocked RPC threads, causing thread pool exhaustion under load.

Solution: Async execution with task tracking and cancellation:

```rust
// Submit async, returns immediately with task ID
let task_id = async_manager.submit(pid, syscall).await;

// Poll for status and progress
let status = async_manager.get_status(&task_id).await;

// Cancel if needed
async_manager.cancel(&task_id).await;
```

**Task Lifecycle:**
- `PENDING` → `RUNNING` → `COMPLETED` / `FAILED` / `CANCELLED`
- TTL-based automatic cleanup (default 1 hour)
- Per-process task tracking with O(1) removal
- Background cleanup task with graceful shutdown support

**Performance Impact:**
- Before: Long sleep blocks RPC thread → thread pool exhaustion
- After: Async task, no thread blocking, can handle thousands concurrently

**3. Batch Syscall Execution (For Bulk Operations)**

Problem: Each syscall required separate RPC call with network overhead. No transactional semantics.

Solution: Batch execution with parallel or sequential modes:

```go
// Go backend: Execute 100 operations in one RPC
requests := []BatchRequest{
    {PID: 1, SyscallType: "read_file", Params: ...},
    {PID: 1, SyscallType: "write_file", Params: ...},
    // ... 98 more
}
result := client.ExecuteBatch(ctx, requests, true) // parallel execution
fmt.Printf("Success: %d, Failed: %d\n", result.SuccessCount, result.FailureCount)
```

**Performance Impact:**
- Before: 100 syscalls = 100 RPCs = ~500ms overhead
- After: 100 syscalls = 1 batch RPC = ~50ms (10x faster)

These enhancements are detailed in [gRPC Improvements Documentation](docs/GRPC_IMPROVEMENTS.md).

### Desktop Shell Layer (TypeScript/React) — The User Experience

The desktop shell had to feel like a real OS, not a web app pretending to be one. That meant proper window management, a complete desktop environment (menu bar, dock, launcher), and the ability to render three distinct types of applications: Blueprint apps from JSON specifications, native TypeScript/React apps with full npm ecosystem access, and native OS processes with terminal UI. All while maintaining sub-10ms response times and desktop-grade interactions.

**Key Responsibilities:**
- Desktop environment with window management (drag, resize, minimize, snap-to-edge)
- Three rendering modes: Blueprint (JSON specs), Native Web (React apps), Native Process (terminal)
- Execute local tools with sub-10ms latency
- Per-app component state with observable updates
- WebSocket streaming for real-time updates
- Keyboard shortcuts and gesture handling (⌘K launcher, Alt+Tab switching)
- App registry and session management UI
- Workspace persistence and restoration

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
- [Native Apps Developer Guide](docs/NATIVE_APPS_DEV_GUIDE.md) - Complete guide to building native TypeScript/React apps
- [Native Apps Plan](docs/NATIVE_APPS_PLAN.md) - Three-tier application system architecture
- [gRPC Improvements](docs/GRPC_IMPROVEMENTS.md) - Streaming, async, and batch execution details
- [Graceful-with-Fallback Pattern](docs/GRACEFUL_WITH_FALLBACK_PATTERN.md) - Async shutdown pattern for background tasks

## Three-Tier Application System — Because One Size Doesn't Fit All

A real desktop OS needs to run different types of applications. AgentOS supports **three distinct application types**, each optimized for different use cases. This isn't just flexibility for the sake of it — it's architectural recognition that simple utilities, complex UIs, and native executables have fundamentally different needs. AI generation is just one way to create Blueprint apps, not the only way.

### Application Types

| Type | Format | Development | Execution | Components | Use Cases |
|------|--------|-------------|-----------|------------|-----------|
| **Blueprint** | JSON (.bp) | AI-generated | Browser | Prebuilt (Button, Input) | Quick apps, forms, AI UIs |
| **Native Web** | TypeScript/React | Hand-coded | Browser | Custom (your JSX/TSX) | Code editors, file explorers, complex UIs |
| **Native Process** | Executables | Any language | OS process | N/A (terminal UI) | Python scripts, CLI tools, Git, Shell |

### 1. Blueprint Apps (Existing System)

I needed a way to define applications that could be both AI-generated and human-readable. Traditional approaches failed on one dimension or the other. Blueprint emerged from a simple insight: if LLMs are going to generate applications, the format needs to be streaming-first. Not an afterthought. Baked in.

Applications can be created in two ways:

1. **AI Generation**: Natural language → LLM generates Blueprint JSON
2. **Manual Definition**: Write `.bp` files directly and drop in `/apps/blueprint` directory

### Why I Chose JSON (And Why It Matters)

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

### 2. Native TypeScript/React Apps (Full React Applications)

For complex applications that need the full power of React, I built a complete native app system. These aren't Blueprint apps with JSON definitions — they're full TypeScript/React applications with complete freedom.

**What makes native apps different:**
- ✅ Write custom React components (no prebuilt Button/Input constraints)
- ✅ Import any npm packages (Monaco Editor, Chart.js, D3, whatever you need)
- ✅ Full React ecosystem (hooks, context, custom state management)
- ✅ Hot Module Replacement (HMR) for instant feedback during development
- ✅ Production-grade tooling (TypeScript, ESLint, Prettier, Vite)
- ❌ No JSON definitions, no prebuilt components — you own the entire component tree

**Development Workflow:**

```bash
# Create new app (scaffolds entire structure)
make create-native-app name="File Explorer"

# Start development with HMR
cd apps/native/file-explorer
npm install
npm run dev

# Build for production (outputs to apps/dist/)
npm run build

# Validate, lint, and type-check
make validate-native-apps
make lint-native-app name=file-explorer
```

**Example Native App (apps/native/file-explorer/src/App.tsx):**

```typescript
import React, { useState, useEffect } from 'react';
import type { NativeAppProps } from '@os/sdk';
import { useVirtualizer } from '@tanstack/react-virtual'; // Any npm package!
import { FileList } from './components/FileList'; // Your custom components

export default function FileExplorer({ context }: NativeAppProps) {
  const { state, executor, window } = context;
  const [files, setFiles] = useState([]);
  const [currentPath, setCurrentPath] = useState('/tmp/ai-os-storage');

  // Load directory contents via executor
  useEffect(() => {
    async function loadFiles() {
      const result = await executor.execute('filesystem.list', { 
        path: currentPath 
      });
      setFiles(result?.entries || []);
    }
    loadFiles();
  }, [currentPath, executor]);

  // Your custom UI, your custom components
  return (
    <div className="file-explorer">
      <FileList 
        files={files} 
        onNavigate={setCurrentPath}
      />
    </div>
  );
}
```

**Native App SDK:**

Every native app receives a `context` prop with:

- **`context.state`**: Observable state management with `get()`, `set()`, `subscribe()`, `batch()`
- **`context.executor`**: Execute backend services (filesystem, storage, HTTP, system)
- **`context.window`**: Window controls — `setTitle()`, `setIcon()`, `close()`, `minimize()`, `maximize()`
- **`context.appId`**: Unique app instance identifier

**Real-World Example: File Explorer Native App**

The File Explorer demonstrates what's possible with native apps:
- Advanced virtualization (@tanstack/react-virtual) handles 10,000+ files
- Multiple view modes (list, grid, compact)
- Multi-select with Ctrl/Cmd/Shift modifier keys
- Copy/cut/paste with system clipboard
- Context menus for file operations
- Full keyboard navigation
- Real-time file operations via executor
- Only 45KB bundle size (optimized production build)

**Tooling & Developer Experience:**

```bash
# Create app from template
make create-native-app name="My App"

# Watch and rebuild on changes (HMR)
make watch-native-app name=my-app

# Validate app structure and manifest
make validate-native-apps

# Type check, lint, format
make lint-native-app name=my-app

# Build all native apps
make build-native-apps
```

See [Native Apps Developer Guide](docs/NATIVE_APPS_DEV_GUIDE.md) for complete documentation.

### 3. Native Process Apps (Run Actual Executables)

For cases where you need to run actual OS processes — Python scripts, CLI tools, Shell commands, compiled binaries — native process apps provide terminal UI and stdio/stderr streaming.

**Supported Executables:**
- Python scripts (`python3 script.py`)
- CLI tools (`ls`, `grep`, `git`, `npm`)
- Shell scripts and interactive shells (`bash`, `zsh`)
- Compiled binaries (Rust, Go, C++)
- Any executable on the host system

**Process App Manifest (apps/native-proc/python-runner/manifest.json):**

```json
{
  "id": "python-runner",
  "name": "Python Runner",
  "type": "native_proc",
  "icon": "🐍",
  "category": "developer",
  "permissions": ["SPAWN_PROCESS", "READ_FILE"],
  "proc_manifest": {
    "executable": "python3",
    "args": ["-i"],
    "working_dir": "/tmp/ai-os-storage",
    "ui_type": "terminal",
    "env": {
      "PYTHONUNBUFFERED": "1"
    }
  }
}
```

**Features:**
- Real-time stdout/stderr streaming via WebSocket
- Bidirectional I/O (send input to stdin)
- Process lifecycle management (spawn, kill, status)
- Terminal UI for interactive shells
- Resource limits and sandboxing via kernel

**When to Use Each Type:**

- **Blueprint**: Quick prototypes, AI-generated UIs, simple forms, dashboard widgets
- **Native Web**: Complex UIs, code editors, file explorers, data visualizations, anything needing npm packages
- **Native Proc**: Running existing executables, Python scripts, Git operations, system utilities

All three types:
- Run in the same windowing system
- Use the same permission model
- Access the same backend services
- Persist via the same registry

## The AI Generation Pattern: Generate-Once-Execute-Many (When You Use It)

When you do use AI generation, AgentOS follows a fundamentally different pattern than chat-based AI interfaces. I watched too many demos where every button click went back to the LLM — 2-5 seconds per interaction, burning tokens like kindling. That's not an application. That's an expensive conversation.

The AI generation in AgentOS (which is optional) follows a better philosophy: generate the application specification once, execute it many times locally. Separate AI generation from application execution at the architectural level. But most apps don't need AI generation at all — they're either prebuilt Blueprint apps (loaded from `.bp` files) or hand-coded native applications.

### Application Lifecycle

**Generation Phase (One-Time, ~100ms-5s)**
```
1. User: "create a calculator"
2. Go Backend → AI Service (gRPC)
3. Template-based or LLM generates structured JSON UISpec
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

### Why This Matters — A Comparison

**Traditional AI Approach (The Slow Way):**
- Every interaction requires LLM inference
- 2-5 seconds per button click
- High token cost per interaction
- Non-deterministic behavior
- Unusable for actual applications

**AgentOS Approach (The Fast Way):**
- Blueprint apps: UI spec loaded once (instant from `.bp` file or 2-5s from LLM), tools execute locally many times
- Native apps: Zero generation time, just TypeScript/React development
- Native processes: Direct OS process execution
- Sub-10ms tool execution for all app types
- Deterministic, local execution
- No network latency for interactions
- Actually feels like software, not a chatbot

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

## Syscall Interface: I Implemented 95+ System Calls (And Yes, They All Work)

Building a kernel means implementing syscalls. Lots of them. I didn't cut corners here — the kernel exposes a comprehensive interface via gRPC with 95+ fully implemented system calls across 13 categories. Not stubs. Not partial implementations. Fully working, tested, and optimized.

### Syscall Categories

| Category | Count | Key Operations |
|----------|-------|----------------|
| **File System** | 14 | read, write, create, delete, list, stat, move, copy, mkdir, rmdir, getcwd, setcwd, truncate, exists |
| **Process Management** | 8 | spawn, kill, get_info, list, set_priority, get_state, get_stats, wait |
| **IPC - Pipes** | 6 | create, write, read, close, destroy, stats |
| **IPC - Shared Memory** | 7 | create, attach, detach, write, read, destroy, stats |
| **IPC - Memory Mapping** | 6 | mmap, mmap_read, mmap_write, msync, munmap, stats |
| **IPC - Async Queues** | 8 | create (FIFO/Priority/PubSub), send, receive, subscribe, unsubscribe, close, destroy, stats |
| **Network Sockets** | 12 | socket, bind, listen, accept, connect, send, recv, sendto, recvfrom, close, setsockopt, getsockopt |
| **File Descriptors** | 6 | open, close, dup, dup2, lseek, fcntl |
| **Signal Handling** | 8 | send_signal, register_handler, block, unblock, get_pending, get_stats, wait_for_signal, get_state |
| **Scheduler** | 10 | schedule_next, yield, get_current, get_stats, set_policy, get_policy, set_quantum, get_quantum, boost_priority, lower_priority |
| **Memory** | 3 | get_stats, get_process_stats, trigger_gc |
| **System Info** | 4 | get_system_info, get_env, set_env, network_request |
| **Time** | 3 | get_current_time, sleep, get_uptime |
| **TOTAL** | **95+** | Fully type-safe via Protocol Buffers |

### Syscall Architecture

**Modular Handler System:**
```rust
pub trait SyscallHandler {
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult>;
    fn name(&self) -> &'static str;
}
```

Handlers registered per category:
- `FsHandler` - Filesystem operations with VFS routing
- `ProcessHandler` - Process management with lifecycle coordination
- `IpcHandler` - IPC with unified manager (pipes + shm + queues + mmap)
- `NetworkHandler` - Socket operations with full TCP/UDP stack
- `FdHandler` - File descriptor table management
- `SignalHandler` - POSIX-style signal delivery
- `SchedulerHandler` - Policy management and vruntime tracking
- `MemoryHandler` - Allocation tracking and GC
- `SystemHandler` - System info and environment
- `TimeHandler` - Time operations
- `MmapHandler` - Memory-mapped file operations
- `AsyncHandler` - Integration with AsyncTaskManager
- `IoUringHandler` - Async completion routing

**Security Integration:**
Every syscall passes through four security layers:
1. **Capability check** - Does process have required capability?
2. **Path validation** - Is path access allowed? (for filesystem ops)
3. **Resource limits** - Within memory/CPU/FD limits?
4. **Permission cache** - Sub-microsecond cached decisions (LRU + TTL)

**Performance Features:**
- **JIT Compilation**: Hot syscalls (>100 calls) compiled with pattern-based optimizations
- **io_uring Integration**: I/O-bound syscalls routed to async completion queues
- **Timeout Handling**: Unified timeout infrastructure with adaptive backoff
- **Zero-Copy IPC**: Shared memory and mmap avoid data copying
- **Lock-Free Structures**: SPSC pipes, MPMC queues, submission rings

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
make build-native-apps  # Build all native TypeScript/React apps
```

**Native Apps Development**
```bash
make create-native-app name="App Name"  # Create new native app from template
make watch-native-apps                  # Watch all native apps with HMR
make watch-native-app name=app-id       # Watch specific app with HMR
make validate-native-apps               # Validate app structure and manifests
make lint-native-apps                   # Lint and type-check all native apps
make lint-native-app name=app-id        # Lint specific app
make fix-native-apps                    # Auto-fix linting issues
make clean-native-apps                  # Clean native app build artifacts
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

### Frontend Performance (TypeScript/React)
- **Tool Execution**: Sub-10ms local tool execution
- **Rendering**: Virtual scrolling for apps with 1000+ components
- **State Management**: Selective Zustand subscriptions prevent unnecessary re-renders
- **Animations**: Hardware-accelerated CSS and GSAP animations
- **Bundle Size**: Code splitting and lazy loading for optimal load times

## System Capabilities

### Desktop-Grade Window Management (Because This Should Feel Like An OS)

I wanted AgentOS to feel like a real desktop OS, not a web app pretending to be one. That meant implementing proper window management:

- **Production-Ready Implementation**: Powered by react-rnd library with full drag/resize/focus — stood on shoulders here
- **Drag & Drop**: Free-form window dragging with smooth animations and visual feedback — feels native
- **Snap-to-Edge**: Automatic window snapping to screen edges and corners (9 snap zones) — Windows 10 style
- **Resize**: Interactive window resizing from all edges and corners with min/max constraints — all 8 drag points work
- **Minimize/Maximize**: Full window state management with smooth transitions — because animations matter
- **Backend Synchronization**: Window positions and sizes synced to Go backend via `POST /apps/:id/window` — state persists
- **Session Restoration**: Window geometry captured in sessions and restored on load — resume exactly where you left off
- **Keyboard Shortcuts**: 
  - `⌘K` / `Ctrl+K` - Spotlight-style app creator
  - `Alt+Tab` - Cycle through open windows
  - `⌘W` / `Ctrl+W` - Close focused window
  - `⌘M` / `Ctrl+M` - Minimize focused window
- **Cascade Positioning**: Automatic cascading for new windows with offset calculation
- **Z-Index Management**: Automatic focus and bring-to-front on interaction
- **Dual-Mode Architecture**: WindowManager for windowed apps + DynamicRenderer for fullscreen (backward compatible)

### Multi-Application Management
- Concurrent execution of multiple applications (Blueprint, Native Web, Native Process)
- Parent-child application relationships
- Focus management with foreground/background states
- Graceful cleanup of child applications when parent closes
- Desktop environment with menu bar, dock, taskbar, and app launcher

### Persistence Layer
- **Blueprint DSL**: Define apps in `.bp` files with streaming-optimized JSON format
- **Prebuilt Apps**: Auto-load apps from `/apps` directory on startup
- **App Registry**: Store and instantly launch generated applications (50-100x faster than regeneration)
- **Session Management**: Save and restore complete workspace state (apps, windows, positions, sizes)
- **Filesystem Integration**: All persistence goes through kernel syscalls
- **Structured Storage**: JSON-based storage with metadata support

### Security Model — Four Layers of "No"

Security is hard. Really hard. My approach was defense in depth: if one layer fails, three more are waiting. Here's the four-layer permission system I built:

**Layer 1: Granular Capability System (Path-Specific Permissions)**
```rust
pub enum Capability {
    ReadFile(Option<PathBuf>),   // Path-specific or wildcard
    WriteFile(Option<PathBuf>),
    CreateFile(Option<PathBuf>),
    DeleteFile(Option<PathBuf>),
    ListDirectory(Option<PathBuf>),
    SpawnProcess,
    KillProcess,
    NetworkAccess(NetworkRule),   // Host/port/CIDR specific
    BindPort(Option<u16>),
    NetworkNamespace,             // Can create network isolation
    SystemInfo,
    TimeAccess,
    SendMessage,
    ReceiveMessage,
}
```
- **Smart Path Matching**: `ReadFile(Some("/tmp"))` grants access to `/tmp/test.txt` — hierarchical makes sense
- **TOCTOU-Safe**: Early canonicalization via `PathHandle` eliminates Time-of-Check-to-Time-of-Use races — classic security bug, eliminated at the type level
- **Network Rules**: Wildcard domains (`*.example.com`), CIDR blocks, port-specific, priority-based evaluation — because network permissions aren't binary

**Layer 2: Permission Caching (Making Security Fast)**
```rust
#[repr(C, align(64))]  // Cache-line aligned for hot path
pub struct PermissionCache {
    cache: DashMap<CacheKey, CachedDecision>,
    hits: AtomicU64,
    misses: AtomicU64,
    ttl: Duration,  // 5 second expiry
}
```
- LRU eviction when full — bounded memory usage
- Per-PID invalidation on policy changes — can't cache stale security decisions
- 10-100x speedup (nanoseconds vs microseconds) — security doesn't have to be slow

**Layer 3: Network Namespace Isolation (The Platform-Specific Nightmare)**

Building cross-platform network isolation taught me why most projects just support Linux. But I made it work:
- **Linux**: True network namespaces
  - Leverages `/proc/self/ns/net` kernel interface
  - Virtual ethernet (veth) pairs for connectivity
  - Bridge networking for inter-namespace communication
  - NAT support for private networks with outbound access
  - Port forwarding for inbound connections
  
- **macOS**: Packet filter-based isolation
  - `pfctl` for network filtering
  - Process-based network rules
  - Application firewall integration
  
- **Simulation**: Fallback for unsupported platforms
  - API-compatible with full implementations
  - Capability-based restrictions
  - Suitable for development and testing

**4 Isolation Modes** (from paranoid to permissive):
1. **Full Isolation**: Complete network lockdown (no external access, loopback only) — maximum security
2. **Private Network**: Isolated with NAT (10.0.0.0/24 private IPs, configurable DNS, optional port forwarding) — practical compromise
3. **Shared Network**: Uses host network stack (no isolation) — when you need full access
4. **Bridged Network**: Custom bridge configuration for inter-namespace communication — for multi-process apps

**Layer 4: Resource Limits (Preventing Resource Exhaustion)**
- cgroups v2 on Linux (memory, CPU shares, max PIDs)
- Per-process memory tracking with OOM detection
- Proactive garbage collection triggers
- File descriptor limits

**Additional Security Features** (The Details Matter):

**Shell Injection Prevention** (because Bobby Tables is real):
- Command validation blocks: `;`, `|`, `&`, `` ` ``, `$`, `>`, `<`, `\n`, `\r` — all the classics
- Environment variable sanitization — `LD_PRELOAD` attacks, I see you
- Working directory restrictions — you spawn where I say you spawn

**Path Security:**
- Allowed/blocked path lists with canonicalization
- Parent directory restrictions
- Symlink resolution with loop detection
- Non-existent path handling (canonicalize parent)

**Syscall Verification:**
- All 95+ syscalls pass through capability checks
- Per-category permission requirements
- Path validation for filesystem operations
- Resource limit enforcement before execution

**IPC Isolation:**
- Pipes: Per-process ownership, reader/writer validation
- Shared Memory: Permission-based access (read-only or read-write)
- Async Queues: Owner-based lifecycle, subscriber management
- Memory Mapping: Process-specific address spaces

**HTTP Layer Protection:**
- Rate limiting: Per-IP token bucket (configurable RPS and burst)
- CORS: Configurable cross-origin policies
- Request size limits
- Timeout enforcement

**Application Security** (Why Blueprint Apps Are Safe):
- **No Arbitrary Code Execution**: Blueprint specs are pure JSON data — it's data, not code
- **Pre-defined Tools**: All operations go through registered tool functions — no dynamic code execution
- **Sandboxed by Design**: Blueprint apps can only invoke predefined tools, not create new syscalls or operations
- **AI-Generated Apps Follow Same Rules**: When you use AI to generate a Blueprint, it's still just JSON data — huge security win

**Automatic Cleanup:**
- Zombie process reaping via waitpid
- IPC resource deallocation (pipes, shm, queues, mmap)
- Network namespace destruction
- File descriptor closing
- Signal handler deregistration
- Socket cleanup
- Memory deallocation
- Unified orchestrator ensures comprehensive coverage

### Extensibility & Architecture

**Application Layer:**
- **Blueprint DSL**: Streaming-optimized JSON format for defining apps with `.bp` files
- **Prebuilt Apps**: Drop `.bp` files in `/apps` directory for automatic loading on startup
- **Tool System**: 80+ modular tools across 10+ categories (UI, app, system, math, network, service-integrated)
- **Component System**: 23 pluggable UI components (primitives, layout, forms, media, UI, special) with Zod validation
- **Service Registry**: Dynamic service discovery with tool binding

**Kernel Layer:**
- **VFS Architecture**: Pluggable filesystem backends (LocalFS for host, MemFS for in-memory) with trait-based design
- **Scheduler Policies**: 3 swappable policies (round-robin, priority, CFS-inspired fair) with dynamic switching
- **IPC Mechanisms**: 4 types - Pipes (lock-free SPSC), Shared Memory (zero-copy), Async Queues (FIFO/Priority/PubSub), Memory Mapping (mmap/msync/munmap)
- **Handler System**: Modular syscall handlers per category with trait-based dispatch
- **Resource Cleanup**: Trait-based `ResourceCleanup` for adding new resource types
- **Network Isolation**: Platform-specific implementations (Linux, macOS, simulation) with unified interface
- **Timeout Policies**: Hierarchical timeouts (Lock: 1-100ms, IPC: 1-30s, IO: 5-300s, Task: 10-3600s, Custom)
- **Observability**: Event categories with severity levels, extensible query system

**Backend Layer:**
- **Middleware Stack**: Extensible HTTP middleware (CORS, rate limiting, authentication-ready)
- **Provider System**: Service providers (filesystem, storage, auth, system) with trait-based registration
- **App Registry**: Persistent application storage with category organization
- **Session Management**: Workspace state persistence with JSON serialization

**Protocol Layer:**
- **gRPC**: Type-safe Protocol Buffers with versioned service definitions
- **Syscalls**: 95+ syscalls across 13 categories with strongly-typed messages
- **Extensibility**: Add new syscalls by implementing handler trait and updating proto definitions

## Performance Monitoring

AgentOS includes comprehensive performance monitoring across all layers of the stack:

### Kernel (Rust)
- **Metrics**: Custom metrics collector with counters, gauges, and histograms
- **Tracing**: Structured tracing for syscalls and operations
- **Format**: JSON and Prometheus-compatible metrics export
- **Access**: Via kernel API

**Key Metrics:**
- Syscall latency (p50, p95, p99)
- Process creation/termination rates
- Memory allocation/deallocation
- IPC throughput (pipes, shared memory)
- VFS operation latency

### Backend (Go)
- **Library**: Prometheus client_golang
- **Metrics**: HTTP requests, service calls, gRPC operations, system metrics
- **Middleware**: Automatic request tracking with duration, size, and status
- **Endpoint**: `GET /metrics` (Prometheus format)

**Key Metrics:**
- HTTP request duration (p50, p95, p99)
- Request/response sizes
- Active applications count
- Service call latency
- gRPC call metrics
- WebSocket connections
- Session operations

### AI Service (Python)
- **Library**: prometheus-client
- **Tracing**: Structured tracing with context managers
- **Metrics**: UI generation, chat, LLM calls, cache performance
- **Format**: Prometheus-compatible

**Key Metrics:**
- UI generation duration and token counts
- Chat response latency
- LLM API call latency and token usage
- Cache hit/miss rates
- gRPC request metrics
- Stream message counts

### UI (TypeScript/React)
- **Library**: web-vitals
- **Metrics**: Core Web Vitals, custom performance metrics
- **Format**: Prometheus-compatible JSON export

**Key Metrics:**
- Core Web Vitals (CLS, FID, LCP)
- First Contentful Paint (FCP)
- Time to First Byte (TTFB)
- Interaction to Next Paint (INP)
- Component render duration
- Tool execution latency
- WebSocket message latency

### Metrics Collection

Each service exposes metrics on a dedicated endpoint:

- **Kernel**: Via kernel API (port 50051)
- **Backend**: `http://localhost:8000/metrics`
- **AI Service**: Via gRPC API (port 50052)
- **UI**: Client-side collection, exportable as JSON

### Prometheus Integration

To scrape metrics with Prometheus, use the following `prometheus.yml` configuration:

```yaml
scrape_configs:
  - job_name: 'agentos-backend'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
    scrape_interval: 15s

  # Add kernel and AI service endpoints as needed
```

### Monitoring Best Practices

1. **Set Alerts**: Configure alerts for high latency (p95 > threshold)
2. **Track Trends**: Monitor metrics over time to identify degradation
3. **Resource Limits**: Watch memory and CPU usage against configured limits
4. **Cache Performance**: Monitor cache hit rates to optimize caching strategy
5. **Error Rates**: Track error metrics to identify reliability issues

## License

MIT License - see LICENSE file for details

## DashMap Stress Test Metrics — Proving It Works Under Load

I was paranoid about concurrent access bugs, so I built stress tests that hammer the system. Here are the results for the kernel's DashMap-based managers (8 worker threads, 4.61s total runtime):

| Component | Metric | Operations | Details |
|-----------|--------|------------|---------|
| Queue Manager | Concurrent Creation | 1,000 | 1,000 successes, 0 errors |
| Queue Manager | Send/Receive | 19,741 | 9,995 sent, 9,746 received |
| Queue Manager | Create/Destroy | 10,000 | Full lifecycle stress test |
| Shared Memory | Concurrent Creation | 1,000 | 1,000 segments allocated |
| Shared Memory | Read/Write | 4,900 | 2,400 writes, 2,500 reads |
| Shared Memory | Attach/Detach | 10,000 | Multi-process attachment |
| Shared Memory | Create/Destroy | 5,000 | Full lifecycle stress test |
| Process Manager | Concurrent Creation | 1,000 | All processes created successfully |
| Process Manager | Priority Changes | 40,000 | High-frequency concurrent updates |
| Process Manager | Info Access | 25,000 | 20,000 reads, 5,000 list calls |
| Process Manager | Create/Terminate | 10,000 | Full lifecycle stress test |
| Combined | Process+IPC Stress | 200 | Multi-manager concurrent operations |
| Extreme | DashMap Operations | 6,000 | 1,000 combined + 5,000 entry API |

All 18 tests passed with zero deadlocks, demonstrating robust concurrent access patterns across all DashMap-based kernel components.

## What Makes This Different — A Desktop OS Built With Modern Architecture

AgentOS isn't trying to be Linux or Windows. It's what you get when you rebuild a desktop operating system from scratch with modern architecture principles, proven algorithms, and production-grade engineering. The innovation is in **how these pieces integrate** to create a legitimate userspace OS. Here's what makes it unique:

### 1. Observability-First Design (Woven Into The Fabric)

Studying how Linux and Fuchsia added observability layer by layer over time inspired me to do something different: design it in from the start. The result is a custom dual-layer system where observability is as fundamental as the scheduler:

- **Dual-layer architecture** (tracing + streaming) — distributed tracing for causality, event streaming for real-time analytics
- **Adaptive sampling with custom Xorshift RNG** — automatically adjusts to stay under 2% CPU, using a fast 2-3 cycle RNG instead of the standard rand crate
- **Welford's algorithm for streaming anomaly detection** — O(1) memory usage, real-time 3σ outlier detection without storing history
- **Causality tracking** — custom correlation IDs that let you follow an event through the entire stack, from syscall to IPC to scheduler
- **Lock-free 65K ring buffer** — power-of-2 sized for fast modulo via bit masking, achieving ~50ns per event emission

### 2. Resource Orchestration (A Unified Cleanup Architecture)

Looking at how Linux handles process cleanup across scattered functions (`do_exit()`, `exit_mm()`, `exit_files()`), I saw an opportunity to design something more unified. The result is a trait-based resource orchestrator that treats cleanup as a first-class system:

- **Unified trait-based system** — every resource type implements `ResourceCleanup`, creating a single consistent pattern
- **Dependency-aware LIFO ordering** — custom ordering system ensures sockets close before memory frees, file descriptors close before processes terminate
- **Comprehensive per-type statistics** — tracks exactly what was cleaned up, when, and in what order for debugging
- **Coverage validation** — compile-time and runtime checks warn if you forgot to register a resource type
- **Extensible design** — adding a new resource type is 20 lines of trait implementation, automatically integrated into the orchestrator

### 3. Lifecycle Management (Type-Safe State Machines)

Inspired by Rust's "make impossible states unrepresentable" philosophy, I designed explicit state machines for process initialization. The type system enforces correct ordering:

- **Explicit state transitions** — `ProcessState::Creating` → `Initializing` → `Ready`, each state has specific allowed operations
- **Scheduler gating** — processes are invisible to the scheduler until they reach `Ready` state, eliminating initialization races
- **Atomic resource initialization** — all IPC, file descriptors, and memory allocated in `Initializing`, failing any step fails the entire initialization
- **Compile-time guarantees** — Rust's type system prevents calling process operations on partially-initialized processes

### 4. Background Task Management (Graceful-with-Fallback Pattern)

One architectural challenge I solved: Rust's `Drop` trait cannot be async, but background tasks require async cleanup. Most systems either leak tasks, force immediate abort, or require manual shutdown. I designed a better pattern used throughout the kernel:

**The Graceful-with-Fallback Pattern:**

```rust
// Preferred path: Explicit graceful shutdown
scheduler_task.shutdown().await;  // Awaitable, clean
// - Sets atomic flag
// - Sends shutdown signal via channel
// - Awaits task completion
// - Logs success

// Fallback path: Automatic abort in Drop (if graceful wasn't called)
drop(scheduler_task);
// - Checks atomic flag
// - Aborts task if graceful wasn't called
// - Logs warning to alert developer
// - Prevents resource leak
```

**Used By:**
- `SchedulerTask`: Autonomous preemptive scheduling task
- `AsyncTaskManager`: Background cleanup task (removes expired tasks every 5 minutes)
- Other long-lived async tasks requiring clean shutdown

**Why This Matters:**
- Fail-safe: Tasks always stop, no resource leaks
- Ergonomic: Drop prevents forgetting manual cleanup
- Feedback: Warning logs make debugging easy
- Production-ready: Handles ungraceful shutdown gracefully

**SchedulerTask Architecture:**

The scheduler isn't just a priority tracker — it's a true preemptive system with autonomous time-quantum enforcement:

```rust
pub struct SchedulerTask {
    scheduler: Arc<Scheduler>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    control_tx: mpsc::Sender<SchedulerCommand>,
    // Graceful-with-fallback shutdown fields
}
```

**Key Features:**
- Autonomous background task runs independently using Tokio
- Dynamic interval adaptation (quantum = 10ms → task ticks every 10ms, automatically adjusts)
- Event-driven control via channels: `pause()`, `resume()`, `trigger()`, `update_quantum()`
- Enforces preemption by periodically invoking scheduler
- Non-blocking, doesn't waste threads
- Graceful shutdown with fallback abort in Drop

**Traditional Problem:**
```
Process A runs → No timer enforcement → Process never yields → Monopolizes CPU
```

**AgentOS Solution:**
```
Process A runs → SchedulerTask ticks every quantum → 
Scheduler checks elapsed time → If quantum expired → 
Preempt Process A → Schedule Process B
```

This is better than cooperative scheduling (no forced preemption) and simpler than Linux's complex timer interrupt system (we're in userspace).

### 5. Performance Engineering (Where Inspiration Meets Implementation)

Every optimization here came from studying how the best systems work, then adapting those ideas to my specific needs. Measured with flamegraphs, criterion benchmarks, and CPU performance counters:

- **Sharded slot pattern** — Inspired by Linux futexes, but adapted for userspace with 512 fixed parking slots and power-of-2 addressing for cache efficiency
- **Adaptive backoff for timeout loops** — Borrowed the idea from spin locks, created a custom three-tier system (spin → yield → sleep) that achieved 7.5x speedup (615ns → 82ns)
- **Lock-free data structures with SIMD batching** — Took the SPSC ring buffer concept and added SIMD batching for 64x fewer atomic operations
- **Permission caching** — Standard caching pattern, custom implementation with cache-line alignment and TTL for the security context (10-100x speedup on hot paths)
- **JIT syscall compilation** — eBPF showed what's possible for kernel syscalls; I built a userspace version with pattern-based optimizations
- **DashMap shard tuning** — Started with defaults, profiled contention patterns, tuned to 128/64/32 shards based on actual workload characteristics
- **ID recycling** — Calculated the exhaustion point (71 minutes at 1 alloc/μs), built a custom recycling system to prevent it

### 6. Cross-Platform Network Isolation (One API, Three Implementations)

Network isolation is trivial on Linux with namespaces, impossible on macOS without them. Rather than limit the system to Linux-only, I built a platform abstraction layer that provides the same security guarantees through different mechanisms:

- **Linux implementation** — leverages `/proc/self/ns/net` for true kernel namespaces with veth pairs and bridge networking
- **macOS implementation** — custom pfctl (packet filter) integration that achieves similar isolation through firewall rules
- **Simulation mode** — capability-based restrictions for unsupported platforms, maintaining API compatibility
- **Unified interface** — all three expose identical APIs, the platform detection happens at compile time

### 7. Production Thinking (Anticipating Failure Modes)

These features came from asking "what breaks in production?" and designing solutions before the problems appear:

- **ID recycling system** — calculated that u32 exhaustion happens in 71 minutes at 1 alloc/μs, built a custom free-list recycler that prevents wraparound
- **Poisoned mutex recovery** — instead of panicking on poisoned mutexes, the system logs the error, marks the resource as failed, and continues serving other requests
- **Attack vector testing** — built validators for shell injection (`;`, `|`, `&`), TOCTOU races (early canonicalization), and path traversal (`..` handling)
- **Coverage validation** — custom compile-time checker that warns if you add a resource type but forget to register it with the cleanup orchestrator
- **Graceful degradation architecture** — each subsystem (observability, JIT, caching) can fail independently without bringing down the core kernel


## Acknowledgments — Standing On Shoulders

I didn't invent most of this. What I did was ask "what would a desktop OS look like if designed from scratch today?" and then put proven pieces together to answer that question:

- **Proven OS Algorithms**: CFS scheduling (Linux), segregated free lists (jemalloc), Unix IPC (POSIX), network namespaces — borrowed from decades of OS research
- **Modern Rust**: Tokio async runtime, DashMap lock-free maps, crossbeam concurrency, parking_lot synchronization — the ecosystem is incredible
- **Desktop Patterns**: Window management (react-rnd), application lifecycle, session persistence — lessons from macOS, Windows, Linux desktop environments
- **gRPC & Protocol Buffers**: Type-safe high-performance RPC — better than REST for inter-service communication
- **React Ecosystem**: Dynamic UI rendering, Zustand state management, virtual scrolling — React enables building a desktop shell in the browser
- **Google Gemini API**: Optional LLM inference for AI-generated Blueprint apps — a power user feature, not the core system

**The innovation is in how these pieces integrate to create a legitimate userspace desktop OS, not in reinventing any particular wheel. AI generation is just one optional feature among many.**
