# AI-Powered Operating System

A lightweight operating system powered by local AI (GPT-OSS) with real-time thought streaming and dynamic application rendering.

## 🏗️ Project Structure

```
os/
├── kernel/          # Rust-based microkernel (OS core)
├── ai-service/      # Python AI integration layer with LangChain
├── ui/              # Electron/React dynamic UI renderer
├── proto/           # Protocol buffer definitions (gRPC)
├── scripts/         # System startup/shutdown scripts
└── logs/            # Runtime logs and PID files (gitignored)
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- Python 3.11+
- Node.js 18+

### Setup & Running

**Option 1: Full System (Recommended)**
```bash
# Start kernel + AI service
./scripts/start-system.sh

# In another terminal, start UI
./scripts/start-ui.sh

# Stop the system
./scripts/stop-system.sh
```

**Option 2: Individual Components**

1. **AI Service** (Python Backend)
```bash
./scripts/start-ai-service.sh
```

2. **UI** (Electron Frontend)
```bash
./scripts/start-ui.sh
```

3. **Kernel** (Rust)
```bash
cd kernel
cargo run
```

## 📚 Documentation

See [plan.md](./plan.md) for the complete architecture and implementation roadmap.

## 🎯 Current Status

- [x] Project structure with clean architecture
- [x] AI service foundation with FastAPI
- [x] UI framework setup (Electron + React)
- [x] **GPT-OSS-20B integrated via Ollama** ✨
- [x] **LangChain streaming callbacks**
- [x] **Chat agent with proper response formatting**
- [x] **WebSocket streaming infrastructure**
- [x] **Type-safe configuration (Pydantic)**
- [x] **Multi-backend support (Ollama + llama.cpp)**
- [x] **Metal GPU acceleration** (Apple Silicon)
- [ ] Dynamic UI rendering
- [ ] Kernel integration

## 🤝 Contributing

This is an experimental project exploring the future of AI-powered operating systems.

