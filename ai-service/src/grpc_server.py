"""
gRPC Server for AI Operations
Exposes LLM functionality to Go service
"""

import logging
import time
import json
import asyncio
from concurrent import futures
import grpc
import sys

# Import generated protobuf code
import ai_pb2
import ai_pb2_grpc

# Import AI modules
from agents.ui_generator import UIGeneratorAgent, ToolRegistry
from agents.chat import ChatAgent, ChatHistory, ChatMessage
from models.loader import ModelLoader, GeminiModel
from models.config import GeminiConfig
from clients.backend import BackendClient

logger = logging.getLogger(__name__)

# JSON size limits (in bytes)
MAX_UI_SPEC_SIZE = 512 * 1024  # 512KB
MAX_CONTEXT_SIZE = 64 * 1024   # 64KB
MAX_JSON_DEPTH = 20            # Maximum nesting depth


def validate_json_size(data: str, max_size: int, name: str = "JSON") -> None:
    """Validate JSON size to prevent DoS attacks"""
    size = sys.getsizeof(data)
    if size > max_size:
        raise ValueError(f"{name} size {size} bytes exceeds maximum {max_size} bytes")


def validate_json_depth(obj: any, max_depth: int = MAX_JSON_DEPTH, current_depth: int = 0) -> None:
    """Validate JSON nesting depth to prevent stack overflow"""
    if current_depth > max_depth:
        raise ValueError(f"JSON nesting depth {current_depth} exceeds maximum {max_depth}")
    
    if isinstance(obj, dict):
        for value in obj.values():
            validate_json_depth(value, max_depth, current_depth + 1)
    elif isinstance(obj, list):
        for item in obj:
            validate_json_depth(item, max_depth, current_depth + 1)


class AIServiceImpl(ai_pb2_grpc.AIServiceServicer):
    """Implementation of AI service"""
    
    def __init__(self, model_loader: ModelLoader, ui_generator: UIGeneratorAgent):
        self.model_loader = model_loader
        self.ui_generator = ui_generator
        logger.info("gRPC AI service initialized")
    
    def GenerateUI(self, request, context):
        """Generate UI specification (non-streaming)"""
        try:
            logger.info(f"GenerateUI: {request.message}")
            
            # Generate UI spec
            ui_spec = self.ui_generator.generate_ui(request.message)
            ui_json = json.dumps(ui_spec.model_dump())
            
            # Validate generated UI spec size
            validate_json_size(ui_json, MAX_UI_SPEC_SIZE, "UI spec")
            
            # Validate nesting depth
            ui_spec_dict = ui_spec.model_dump()
            validate_json_depth(ui_spec_dict)
            
            thoughts = [
                f"Analyzed request: {request.message}",
                f"Generated {ui_spec.title}",
                f"Components: {len(ui_spec.components)}",
            ]
            
            return ai_pb2.UIResponse(
                app_id="",  # App ID assigned by Go service
                ui_spec_json=ui_json,
                thoughts=thoughts,
                success=True
            )
        
        except ValueError as e:
            # Validation error
            logger.error(f"GenerateUI validation error: {e}")
            return ai_pb2.UIResponse(
                app_id="",
                ui_spec_json="",
                thoughts=[],
                success=False,
                error=f"Validation failed: {str(e)}"
            )
        except Exception as e:
            logger.error(f"GenerateUI error: {e}", exc_info=True)
            return ai_pb2.UIResponse(
                app_id="",
                ui_spec_json="",
                thoughts=[],
                success=False,
                error=str(e)
            )
    
    def StreamUI(self, request, context):
        """Stream UI generation with real-time updates"""
        try:
            logger.info(f"StreamUI: {request.message}")
            
            # Send start token
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.GENERATION_START,
                content="Analyzing request...",
                timestamp=int(time.time())
            )
            
            # Send thought
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.THOUGHT,
                content=f"Analyzing request: {request.message}",
                timestamp=int(time.time())
            )
            
            # Stream tokens in real-time using the generator
            # Batch tokens to avoid gRPC buffering small messages
            ui_spec = None
            token_buffer = ""
            token_count = 0
            BATCH_SIZE = 20  # chars per batch - smaller for more real-time updates
            
            for item in self.ui_generator.generate_ui_stream(request.message):
                if isinstance(item, dict) and item.get("reset"):
                    # LLM failed, resetting - send GENERATION_START to signal fresh start
                    logger.info("LLM generation failed, falling back to rule-based generation")
                    yield ai_pb2.UIToken(
                        type=ai_pb2.UIToken.GENERATION_START,
                        content="Using rule-based generation...",
                        timestamp=int(time.time())
                    )
                    token_buffer = ""  # Clear any accumulated garbage
                    token_count = 0
                elif isinstance(item, str):
                    token_count += 1
                    token_buffer += item
                    
                    # Send when buffer reaches batch size
                    if len(token_buffer) >= BATCH_SIZE:
                        if token_count % 100 == 0:
                            logger.info(f"Sent {token_count} tokens ({len(token_buffer)} chars)...")
                        yield ai_pb2.UIToken(
                            type=ai_pb2.UIToken.TOKEN,
                            content=token_buffer,
                            timestamp=int(time.time())
                        )
                        token_buffer = ""
                else:
                    # UISpec - flush any remaining tokens first
                    if token_buffer:
                        yield ai_pb2.UIToken(
                            type=ai_pb2.UIToken.TOKEN,
                            content=token_buffer,
                            timestamp=int(time.time())
                        )
                        token_buffer = ""
                    ui_spec = item
            
            # Flush any remaining buffered tokens
            if token_buffer:
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.TOKEN,
                    content=token_buffer,
                    timestamp=int(time.time())
                )
            
            logger.info(f"Finished streaming {token_count} tokens")
            
            # Send thoughts
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.THOUGHT,
                content=f"Identified {len(ui_spec.components)} components",
                timestamp=int(time.time())
            )
            
            # Send complete
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.COMPLETE,
                content="",
                timestamp=int(time.time())
            )
        
        except Exception as e:
            logger.error(f"StreamUI error: {e}", exc_info=True)
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.ERROR,
                content=str(e),
                timestamp=int(time.time())
            )
    
    def StreamChat(self, request, context):
        """Stream chat response"""
        try:
            logger.info(f"StreamChat: {request.message}")
            
            # Build history
            history = ChatHistory()
            for msg in request.history:
                history.add(ChatMessage(
                    role=msg.role,
                    content=msg.content,
                    timestamp=msg.timestamp
                ))
            
            # Add user message
            user_msg = ChatAgent.create_user_message(request.message)
            history.add(user_msg)
            
            # Send start token
            yield ai_pb2.ChatToken(
                type=ai_pb2.ChatToken.GENERATION_START,
                content="",
                timestamp=int(time.time())
            )
            
            # Load Gemini model for chat - fresh instance each time
            llm = self.model_loader.load(GeminiConfig(
                model_name="gemini-2.0-flash-exp",
                streaming=True,
                temperature=0.7,  # Higher temperature for natural chat responses
                max_tokens=2048
            ))
            
            logger.debug("Using Gemini for chat streaming")
            agent = ChatAgent(llm)
            
            # Stream response - run async in sync context
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            
            try:
                async def stream_tokens():
                    async for token in agent.stream_response(request.message, history):
                        yield ai_pb2.ChatToken(
                            type=ai_pb2.ChatToken.TOKEN,
                            content=token,
                            timestamp=int(time.time())
                        )
                
                async_gen = stream_tokens()
                while True:
                    try:
                        token = loop.run_until_complete(async_gen.__anext__())
                        yield token
                    except StopAsyncIteration:
                        break
            finally:
                loop.close()
            
            # Send complete
            yield ai_pb2.ChatToken(
                type=ai_pb2.ChatToken.COMPLETE,
                content="",
                timestamp=int(time.time())
            )
        
        except Exception as e:
            logger.error(f"StreamChat error: {e}", exc_info=True)
            yield ai_pb2.ChatToken(
                type=ai_pb2.ChatToken.ERROR,
                content=str(e),
                timestamp=int(time.time())
            )


def serve(port="50052", backend_url="http://localhost:8000"):
    """Start the gRPC server"""
    # Initialize components
    logger.info("Initializing AI service components...")
    
    model_loader = ModelLoader
    tool_registry = ToolRegistry()
    
    # Discover backend services
    backend_services = []
    try:
        logger.info(f"Discovering backend services from {backend_url}...")
        backend_client = BackendClient(backend_url)
        if backend_client.health_check():
            backend_services = backend_client.discover_services()
            logger.info(f"✅ Discovered {len(backend_services)} backend services")
        else:
            logger.warning("Backend not reachable, proceeding without backend services")
        backend_client.close()
    except Exception as e:
        logger.warning(f"Failed to discover backend services: {e}. Continuing without backend integration.")
    
    ui_generator = UIGeneratorAgent(
        tool_registry=tool_registry,
        service_registry=None,  # Not needed for gRPC service
        context_builder=None,
        backend_services=backend_services
    )
    
    # Load Gemini model for UI generation
    try:
        logger.info("Loading Gemini model...")
        llm = model_loader.load(GeminiConfig(
            model_name="gemini-2.0-flash-exp",
            streaming=True,
            temperature=0.1,  # Low temperature for structured JSON output
            max_tokens=4096,  # Larger for complex UIs
            json_mode=False  # We'll handle JSON in prompts for better streaming visibility
        ))
        ui_generator.llm = llm
        ui_generator.use_llm = True
        logger.info("✅ Gemini model loaded successfully")
    except Exception as e:
        logger.warning(f"Could not load Gemini: {e}. Using rule-based generation.")
    
    # Create gRPC server
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    ai_pb2_grpc.add_AIServiceServicer_to_server(
        AIServiceImpl(model_loader, ui_generator),
        server
    )
    
    server.add_insecure_port(f"[::]:{port}")
    
    logger.info(f"🌐 AI gRPC server starting on port {port}")
    server.start()
    
    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        logger.info("🛑 Shutting down AI service...")
        server.stop(grace=5)
        model_loader.unload()


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    serve()

