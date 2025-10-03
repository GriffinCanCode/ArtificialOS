"""
AI Service - Main Entry Point
Handles LLM inference, streaming, and MCP protocol
"""

import asyncio
import logging
import time
import signal
import sys
import atexit
from typing import Any, Dict, Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Import our modules
from agents import ChatAgent
from agents.chat import ChatHistory, ChatMessage
from agents.ui_generator import UIGeneratorAgent, ToolRegistry
from agents.app_manager import AppManager
from agents.kernel_tools import get_kernel_tools
from agents.context import ContextBuilder
from kernel_client import get_kernel_client
from models import ModelConfig, ModelLoader
from models.config import ModelSize, ModelBackend
from streaming.callbacks import StreamCallback
from streaming.thought_stream import ThoughtStreamManager
from mcp.protocol import MCPHandler
from services import ServiceRegistry, ServiceContext
from services.builtin import StorageService, AIService, AuthService

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


# Graceful shutdown handlers
def cleanup_on_exit():
    """Ensure model is unloaded on any exit."""
    try:
        logger.info("🧹 Emergency cleanup: Unloading model...")
        ModelLoader.unload()
    except Exception as e:
        logger.error(f"Error during emergency cleanup: {e}")


def signal_handler(signum, frame):
    """Handle shutdown signals (SIGTERM, SIGINT)."""
    signal_name = "SIGTERM" if signum == signal.SIGTERM else "SIGINT"
    logger.info(f"⚠️  Received {signal_name}, initiating graceful shutdown...")
    cleanup_on_exit()
    sys.exit(0)


# Register signal handlers for graceful shutdown
signal.signal(signal.SIGTERM, signal_handler)
signal.signal(signal.SIGINT, signal_handler)

# Register atexit handler as last resort
atexit.register(cleanup_on_exit)


# Lifespan context manager for startup/shutdown
@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize services on startup, cleanup on shutdown"""
    logger.info("🚀 AI Service starting up...")
    
    # Initialize model configuration
    # Using Ollama for proper chat template support
    config = ModelConfig(
        backend=ModelBackend.OLLAMA,  # Use Ollama (better quality)
        size=ModelSize.SMALL,  # 20B model
        context_length=8192,
        max_tokens=4096,  # Increased to prevent JSON truncation
        temperature=0.3,  # Lower temperature for more consistent JSON output
        streaming=True,
    )
    
    # Initialize components
    app.state.config = config
    app.state.stream_manager = ThoughtStreamManager()
    app.state.mcp_handler = MCPHandler()
    app.state.model_loader = ModelLoader
    app.state.tool_registry = ToolRegistry()
    
    # Initialize service system
    app.state.service_registry = ServiceRegistry()
    app.state.context_builder = None  # Will be initialized after kernel
    
    logger.info(f"Configuration: {config.model_name}")
    logger.info(f"Tools registered: {len(app.state.tool_registry.tools)}")
    
    # Initialize kernel connection
    try:
        kernel_client = get_kernel_client()
        kernel_tools = get_kernel_tools()
        
        # Create default sandboxed process for AI operations
        default_pid = kernel_tools.create_sandboxed_process(
            name="ai-service-default",
            sandbox_level="STANDARD"
        )
        
        if default_pid:
            app.state.kernel_client = kernel_client
            app.state.kernel_tools = kernel_tools
            app.state.default_pid = default_pid
            logger.info(f"✅ Connected to kernel (default PID: {default_pid})")
            
            # Register kernel-dependent services
            storage_service = StorageService(kernel_tools=kernel_tools)
            app.state.service_registry.register(storage_service)
            logger.info("✅ Storage service registered")
        else:
            logger.warning("⚠️  Could not create default kernel process")
            app.state.kernel_client = None
            app.state.kernel_tools = None
            app.state.default_pid = None
    except Exception as e:
        logger.warning(f"⚠️  Kernel connection failed: {e}")
        logger.warning("AI Service will run without kernel integration")
        app.state.kernel_client = None
        app.state.kernel_tools = None
        app.state.default_pid = None
    
    # Register non-kernel services
    auth_service = AuthService()
    app.state.service_registry.register(auth_service)
    logger.info("✅ Auth service registered")
    
    # AI service will be registered when LLM loads
    
    # Initialize context builder and managers
    app.state.context_builder = ContextBuilder(app.state.service_registry)
    app.state.ui_generator = UIGeneratorAgent(
        tool_registry=app.state.tool_registry,
        service_registry=app.state.service_registry,
        context_builder=app.state.context_builder
    )
    app.state.app_manager = AppManager(
        service_registry=app.state.service_registry,
        kernel_tools=app.state.kernel_tools
    )
    
    # Log service statistics
    stats = app.state.service_registry.get_stats()
    logger.info(f"✅ Services: {stats['total_services']} services, {stats['total_tools']} tools")
    
    logger.info("✅ AI Service ready!")
    logger.info("Note: Model will load on first request")
    
    yield
    
    # Shutdown cleanup
    logger.info("🛑 AI Service shutting down...")
    
    # 1. Close all active WebSocket connections
    try:
        active_conns = list(app.state.stream_manager.active_connections.values())
        logger.info(f"Closing {len(active_conns)} active WebSocket connections...")
        for ws in active_conns:
            try:
                await ws.close()
            except:
                pass
    except Exception as e:
        logger.error(f"Error closing WebSocket connections: {e}")
    
    # 2. Unload model and free memory
    try:
        logger.info("Unloading LLM model...")
        ModelLoader.unload()
        logger.info("✅ Model unloaded")
    except Exception as e:
        logger.error(f"Error unloading model: {e}")
    
    # 3. Close kernel connection
    try:
        if hasattr(app.state, 'kernel_client') and app.state.kernel_client:
            logger.info("Closing kernel connection...")
            await app.state.kernel_client.close()
            logger.info("✅ Kernel connection closed")
    except Exception as e:
        logger.error(f"Error closing kernel connection: {e}")
    
    logger.info("✅ Shutdown complete")


app = FastAPI(
    title="AI-OS Service",
    description="AI Integration Layer for the AI-Powered Operating System",
    version="0.1.0",
    lifespan=lifespan
)

# Enable CORS for local development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify exact origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request/Response Models
class ChatRequest(BaseModel):
    message: str
    context: dict = {}


class ChatResponse(BaseModel):
    response: str
    thoughts: list[str] = []
    ui_spec: dict = None


@app.get("/")
async def root():
    """Health check endpoint"""
    return {
        "status": "online",
        "service": "AI-OS Service",
        "version": "0.1.0",
        "timestamp": time.time()
    }


@app.get("/health")
async def health():
    """Detailed health check"""
    health_data = {
        "status": "healthy",
        "model": app.state.config.model_name,
        "model_loaded": app.state.model_loader._instance is not None,
        "active_connections": len(app.state.stream_manager.active_connections),
        "gpu_enabled": app.state.config.requires_gpu,
        "app_manager": app.state.app_manager.get_stats(),
        "kernel": {
            "connected": app.state.kernel_client is not None,
            "default_pid": app.state.default_pid
        }
    }
    
    # Get kernel system info if available
    if app.state.kernel_tools and app.state.default_pid:
        try:
            kernel_info = app.state.kernel_tools.system_info()
            if kernel_info:
                health_data["kernel"]["system_info"] = kernel_info
        except:
            pass
    
    return health_data


@app.websocket("/stream")
async def websocket_endpoint(websocket: WebSocket):
    """
    WebSocket endpoint for real-time thought streaming.
    
    Client sends: {"type": "chat", "message": "...", "context": {...}}
    Server streams: 
        - {"type": "generation_start", ...}
        - {"type": "token", "content": "...", ...}
        - {"type": "thought", "content": "...", ...}
        - {"type": "generation_end", ...}
        - {"type": "complete", ...}
    """
    await websocket.accept()
    connection_id = id(websocket)
    app.state.stream_manager.add_connection(connection_id, websocket)
    
    logger.info(f"🔌 WebSocket connected: {connection_id}")
    
    try:
        # Send welcome message
        await websocket.send_json({
            "type": "system",
            "message": "Connected to AI-OS Service",
            "connection_id": connection_id,
            "model": app.state.config.model_name
        })
        
        # Listen for messages
        while True:
            data = await websocket.receive_json()
            
            if data.get("type") == "chat":
                # Process chat message with streaming
                message = data.get("message", "")
                context = data.get("context", {})
                
                logger.info(f"💬 Received: {message[:100]}...")
                
                # Stream response with real LLM
                await stream_ai_response(message, context, websocket, app)
                
                # Send completion signal
                await websocket.send_json({
                    "type": "complete",
                    "timestamp": time.time()
                })
            
            elif data.get("type") == "generate_ui":
                # Generate UI with real-time streaming updates
                message = data.get("message", "")
                context = data.get("context", {})
                
                logger.info(f"🎨 Generating UI: {message[:100]}...")
                
                # Stream UI generation with thoughts
                await stream_ui_generation(message, context, websocket, app)
                
                # Send completion signal
                await websocket.send_json({
                    "type": "complete",
                    "timestamp": time.time()
                })
            
            elif data.get("type") == "ping":
                await websocket.send_json({"type": "pong"})
    
    except WebSocketDisconnect:
        logger.info(f"🔌 WebSocket disconnected: {connection_id}")
    except Exception as e:
        logger.error(f"❌ WebSocket error: {e}", exc_info=True)
        try:
            await websocket.send_json({
                "type": "error",
                "message": str(e)
            })
        except:
            pass
    finally:
        app.state.stream_manager.remove_connection(connection_id)


async def stream_ui_generation(
    message: str,
    context: dict,
    websocket: WebSocket,
    app: FastAPI
) -> None:
    """
    Stream UI generation with real-time updates.
    
    Args:
        message: User's request (e.g., "create a calculator")
        context: Additional context (parent_app_id, etc.)
        websocket: WebSocket connection
        app: FastAPI app instance
    """
    try:
        # Send generation start
        await websocket.send_json({
            "type": "generation_start",
            "message": "Analyzing request...",
            "timestamp": time.time()
        })
        
        # Send thought: analyzing
        await websocket.send_json({
            "type": "thought",
            "content": f"Analyzing request: {message}",
            "timestamp": time.time()
        })
        
        # Load LLM if needed and update UI generator
        if not app.state.ui_generator.use_llm:
            try:
                await websocket.send_json({
                    "type": "thought",
                    "content": "Loading AI model for intelligent UI generation...",
                    "timestamp": time.time()
                })
                llm = app.state.model_loader.load(app.state.config)
                app.state.ui_generator.llm = llm
                app.state.ui_generator.use_llm = True
                logger.info("LLM loaded and attached to UI generator")
            except Exception as e:
                logger.warning(f"Could not load LLM: {e}. Using rule-based generation.")
        
        # Create streaming callback to send tokens in real-time
        async def token_callback(token: str):
            """Send each token as it's generated"""
            try:
                await websocket.send_json({
                    "type": "generation_token",
                    "content": token,
                    "timestamp": time.time()
                })
            except Exception as e:
                logger.warning(f"Failed to send token: {e}")
        
        # Send thought: generating UI
        await websocket.send_json({
            "type": "thought",
            "content": "Generating UI specification with AI...",
            "timestamp": time.time()
        })
        
        # Generate UI spec with LLM and streaming callback
        # Note: generate_ui is synchronous but calls stream_callback for each token
        ui_spec = app.state.ui_generator.generate_ui(
            message,
            stream_callback=lambda token: asyncio.create_task(token_callback(token))
        )
        
        # Send thought: components identified
        await websocket.send_json({
            "type": "thought",
            "content": f"Identified {len(ui_spec.components)} components",
            "timestamp": time.time()
        })
        
        # Convert to JSON
        ui_json = ui_spec.model_dump()
        
        # Send thought: binding tools
        await websocket.send_json({
            "type": "thought",
            "content": "Binding interactive tools to components",
            "timestamp": time.time()
        })
        
        # Register with AppManager
        parent_id = context.get("parent_app_id")
        app_instance = app.state.app_manager.spawn_app(
            request=message,
            ui_spec=ui_json,
            parent_id=parent_id
        )
        
        logger.info(f"Generated UI: {ui_spec.title} (app_id={app_instance.id})")
        
        # Send UI spec
        await websocket.send_json({
            "type": "ui_generated",
            "app_id": app_instance.id,
            "ui_spec": ui_json,
            "timestamp": time.time()
        })
        
    except Exception as e:
        logger.error(f"Error in stream_ui_generation: {e}", exc_info=True)
        await websocket.send_json({
            "type": "error",
            "message": f"UI generation error: {str(e)}",
            "timestamp": time.time()
        })


async def stream_ai_response(
    message: str,
    context: dict,
    websocket: WebSocket,
    app: FastAPI
) -> None:
    """
    Stream AI response with real LLM integration.
    
    Args:
        message: User's message
        context: Additional context (history, etc.)
        websocket: WebSocket connection
        app: FastAPI app instance (for state access)
    """
    try:
        # Get or create chat history
        history = context.get("history")
        if isinstance(history, list):
            chat_history = ChatHistory(
                messages=[ChatMessage(**msg) for msg in history]
            )
        else:
            chat_history = ChatHistory()
        
        # Add user message to history
        user_msg = ChatAgent.create_user_message(message)
        chat_history.add(user_msg)
        
        # Load model with streaming callbacks
        callback = StreamCallback(websocket)
        llm = app.state.model_loader.load(
            app.state.config,
            callbacks=[callback]
        )
        
        # Create agent and stream response
        agent = ChatAgent(llm)
        
        response_content = ""
        async for token in agent.stream_response(message, chat_history):
            response_content += token
            # Token already sent via callback
        
        # Add assistant response to history
        assistant_msg = ChatAgent.create_assistant_message(response_content)
        chat_history.add(assistant_msg)
        
        # Send updated history back (optional)
        await websocket.send_json({
            "type": "history_update",
            "history": [msg.model_dump() for msg in chat_history.messages[-10:]],
            "timestamp": time.time()
        })
        
    except Exception as e:
        logger.error(f"Error in stream_ai_response: {e}", exc_info=True)
        await websocket.send_json({
            "type": "error",
            "message": f"Generation error: {str(e)}",
            "timestamp": time.time()
        })


@app.post("/chat")
async def chat(request: ChatRequest) -> ChatResponse:
    """
    Non-streaming chat endpoint
    Useful for simple requests that don't need real-time streaming
    """
    # Placeholder response
    return ChatResponse(
        response=f"Echo: {request.message}",
        thoughts=[
            "Received message",
            "Processing request",
            "Generating response"
        ]
    )


@app.post("/generate-ui")
async def generate_ui(request: ChatRequest):
    """
    Generate UI specification for dynamic rendering.
    Also registers the app with AppManager.
    
    Example: "Create a calculator" -> Returns JSON UI spec + app_id
    """
    try:
        logger.info(f"Generating UI for: {request.message}")
        
        # Load LLM if needed and update UI generator
        if not app.state.ui_generator.use_llm:
            try:
                logger.info("Loading LLM for UI generation...")
                llm = app.state.model_loader.load(app.state.config)
                app.state.ui_generator.llm = llm
                app.state.ui_generator.use_llm = True
                logger.info("LLM loaded and attached to UI generator")
            except Exception as e:
                logger.warning(f"Could not load LLM: {e}. Using rule-based generation.")
        
        # Use UI generator agent
        ui_spec = app.state.ui_generator.generate_ui(request.message)
        
        # Convert to JSON
        ui_json = ui_spec.model_dump()
        
        # Register with AppManager
        parent_id = request.context.get("parent_app_id")
        app_instance = app.state.app_manager.spawn_app(
            request=request.message,
            ui_spec=ui_json,
            parent_id=parent_id
        )
        
        logger.info(f"Generated UI: {ui_spec.title} (app_id={app_instance.id})")
        
        return {
            "app_id": app_instance.id,
            "ui_spec": ui_json,
            "thoughts": [
                f"Analyzed request: {request.message}",
                f"Generated {ui_spec.title}",
                f"Components: {len(ui_spec.components)}",
            ]
        }
    except Exception as e:
        logger.error(f"Error generating UI: {e}", exc_info=True)
        return {
            "error": str(e),
            "app_id": None,
            "ui_spec": None,
            "thoughts": [f"Error: {str(e)}"]
        }


@app.get("/apps")
async def list_apps():
    """List all running apps."""
    apps = app.state.app_manager.list_apps()
    return {
        "apps": [
            {
                "id": a.id,
                "title": a.title,
                "state": a.state,
                "created_at": a.created_at,
            }
            for a in apps
        ],
        "stats": app.state.app_manager.get_stats()
    }


@app.post("/apps/{app_id}/focus")
async def focus_app(app_id: str):
    """Focus an app (bring to foreground)."""
    success = app.state.app_manager.focus_app(app_id)
    return {"success": success, "app_id": app_id}


@app.delete("/apps/{app_id}")
async def close_app(app_id: str):
    """Close and destroy an app."""
    success = app.state.app_manager.close_app(app_id)
    return {"success": success, "app_id": app_id}


@app.get("/services")
async def list_services(category: Optional[str] = None):
    """List all available services."""
    from services import ServiceCategory
    
    cat = ServiceCategory(category) if category else None
    services = app.state.service_registry.list_all(category=cat)
    
    return {
        "services": [s.model_dump() for s in services],
        "stats": app.state.service_registry.get_stats()
    }


@app.post("/services/discover")
async def discover_services(request: ChatRequest):
    """Discover relevant services for a request."""
    services = app.state.service_registry.discover(
        request.message,
        limit=5
    )
    
    return {
        "query": request.message,
        "services": [s.model_dump() for s in services]
    }


class ServiceExecuteRequest(BaseModel):
    """Service tool execution request"""
    tool_id: str
    params: Dict[str, Any]
    app_id: Optional[str] = None


@app.post("/services/execute")
async def execute_service_tool(request: ServiceExecuteRequest):
    """Execute a service tool."""
    from services import ServiceContext
    
    # Build context
    context = None
    if request.app_id:
        app_instance = app.state.app_manager.get_app(request.app_id)
        if app_instance:
            context = ServiceContext(
                app_id=request.app_id,
                sandbox_pid=app_instance.sandbox_pid
            )
    
    # Execute tool
    result = await app.state.service_registry.execute(
        tool_id=request.tool_id,
        params=request.params,
        context=context
    )
    
    return result.model_dump()


if __name__ == "__main__":
    import uvicorn
    
    print("=" * 60)
    print("🤖 AI-Powered OS - AI Service")
    print("=" * 60)
    
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,  # Hot reload for development
        log_level="info"
    )
