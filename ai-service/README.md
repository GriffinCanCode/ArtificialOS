# AI Service (gRPC)

Pure AI operations service - handles LLM inference only.

## Purpose

This service is now **AI-only**. All HTTP, WebSocket, app management, and orchestration logic has been moved to the Go service.

## Architecture

```
Go Service → [gRPC] → Python AI Service
                       ├─ LLM Loading
                       ├─ Chat Agent
                       ├─ UI Generator
                       └─ Streaming
```

## What This Service Does

✅ **LLM Operations**
- Load and manage language models (Ollama/llama.cpp)
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

### Start gRPC Server

```bash
cd ai-service
source venv/bin/activate
PYTHONPATH=src python3 -m grpc_server
```

Server runs on port **50052**

### Configuration

- Model: GPT-OSS-20B via Ollama
- Backend: Ollama (for better chat templates)
- Streaming: Enabled
- Port: 50052 (gRPC)

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

**Kept:**
- langchain, llama-cpp-python, langchain-ollama
- grpcio, grpcio-tools
- pydantic

**Removed:**
- ❌ fastapi - No longer needed
- ❌ uvicorn - No longer needed
- ❌ websockets - No longer needed

## Development

### Regenerate Protobuf Code

```bash
cd ../
./scripts/compile-protos.sh
```

### Test LLM Loading

```python
from models import ModelLoader, ModelConfig
from models.config import ModelBackend, ModelSize

loader = ModelLoader()
config = ModelConfig(backend=ModelBackend.OLLAMA, size=ModelSize.SMALL)
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

- Model loading: ~2-5 seconds (first request)
- UI generation: ~500ms-2s (with LLM)
- Chat streaming: Real-time token delivery
- Memory: ~4GB (with 20B model)

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

