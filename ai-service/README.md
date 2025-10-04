# AI Service (gRPC)

Pure AI operations service - handles LLM inference only.

## Purpose

This service is now **AI-only**. All HTTP, WebSocket, app management, and orchestration logic has been moved to the Go service.

## Architecture

```
Go Service → [gRPC] → Python AI Service
                       ├─ Gemini API Client
                       ├─ Chat Agent
                       ├─ UI Generator
                       └─ Streaming
```

## What This Service Does

✅ **LLM Operations**
- LLM inference via Google Gemini API (gemini-2.0-flash-exp)
- Generate UI specifications from natural language
- Stream chat responses
- Token-level streaming for real-time updates

❌ **What Was Removed** (Now in Go)
- HTTP/REST API
- WebSocket connections
- App lifecycle management
- Service registry
- Kernel communication
- Connection management

## Running

### Setup

1. Create a `.env` file in the `ai-service/` directory:
```bash
GOOGLE_API_KEY=your_gemini_api_key_here
```

2. Install dependencies:
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### Start gRPC Server

```bash
cd ai-service
source venv/bin/activate
PYTHONPATH=src python3 -m grpc_server
```

Server runs on port **50052**

### Configuration

- Model: Gemini 2.0 Flash (Experimental) via Google API
- Streaming: Enabled
- Port: 50052 (gRPC)
- Temperature: 0.1 (UI generation), 0.7 (chat)

## gRPC API

### Services

#### `GenerateUI`
```protobuf
rpc GenerateUI(UIRequest) returns (UIResponse);
```
Generate UI specification from natural language (non-streaming).

#### `StreamUI`
```protobuf
rpc StreamUI(UIRequest) returns (stream UIToken);
```
Stream UI generation with real-time token updates.

#### `StreamChat`
```protobuf
rpc StreamChat(ChatRequest) returns (stream ChatToken);
```
Stream chat response with token-by-token delivery.

## File Structure

```
ai-service/
├── src/
│   ├── grpc_server.py      # NEW: gRPC server entry point
│   ├── ai_pb2.py           # Generated: protobuf messages
│   ├── ai_pb2_grpc.py      # Generated: gRPC stubs
│   ├── agents/
│   │   ├── chat.py         # ChatAgent (LLM-powered)
│   │   ├── ui_generator.py # UI generation agent
│   │   └── templates.py    # UI templates
│   ├── models/
│   │   ├── config.py       # Model configuration
│   │   └── loader.py       # Model loading/unloading
│   └── _archived_fastapi/  # OLD CODE (archived)
│       ├── main.py.backup  # Old FastAPI server
│       ├── app_manager.py  # → Moved to Go
│       ├── kernel_client.py # → Moved to Go
│       └── services/       # → Moved to Go
├── proto/
│   └── ai.proto            # gRPC service definition
├── requirements.txt        # Slimmed down (no FastAPI)
└── venv/
```

## Dependencies

**Core:**
- google-generativeai - Gemini API client
- langchain - LLM framework
- grpcio, grpcio-tools - gRPC communication
- pydantic - Data validation

## Development

### Regenerate Protobuf Code

```bash
cd ../
./scripts/compile-protos.sh
```

### Test LLM Loading

```python
from models import ModelLoader, GeminiConfig

loader = ModelLoader()
config = GeminiConfig(
    model_name="gemini-2.0-flash-exp",
    temperature=0.1,
    max_tokens=4096
)
llm = loader.load(config)
```

### Test gRPC Locally

```bash
# Install grpcurl
brew install grpcurl

# Test health (server must be running)
grpcurl -plaintext localhost:50052 list

# Test UI generation
grpcurl -plaintext -d '{"message": "create a calculator"}' \
  localhost:50052 ai.AIService/GenerateUI
```

## Performance

- API initialization: <100ms (first request)
- UI generation: ~1-3s (with Gemini API)
- Chat streaming: Real-time token delivery
- Memory: ~200MB (Python service only, no model weights)

## Migration Notes

This service used to be the main entry point (FastAPI on port 8000). It's now a backend service that only handles AI operations. The Go service is now the main entry point.

**Old Architecture:**
```
Frontend → Python FastAPI (8000)
```

**New Architecture:**
```
Frontend → Go Service (8000) → Python AI gRPC (50052)
```

See `_archived_fastapi/` for old code.

