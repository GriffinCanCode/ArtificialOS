# AI-Powered Operating System

A lightweight operating system powered by local AI (GPT-OSS) with real-time thought streaming and dynamic application rendering.

## 🏗️ Project Structure

```
os/
├── kernel/          # Rust-based microkernel (OS core)
├── ai-service/      # Python AI integration layer with LangChain
└── ui/              # Electron/React dynamic UI renderer
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- Python 3.11+
- Node.js 18+

### Setup

1. **AI Service** (Python Backend)
```bash
cd ai-service
python3 -m venv venv
source venv/bin/activate  # On macOS/Linux
pip install -r requirements.txt
python3 src/main.py
```

2. **UI** (Electron Frontend)
```bash
cd ui
npm install
npm run dev
```

3. **Kernel** (Rust - Coming Soon)
```bash
cd kernel
cargo build --release
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

