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

# Import generated protobuf code
import ai_pb2
import ai_pb2_grpc

# Import AI modules
from agents.ui_generator import UIGeneratorAgent, ToolRegistry
from agents.chat import ChatAgent, ChatHistory, ChatMessage
from models import ModelConfig, ModelLoader
from models.config import ModelSize, ModelBackend

logger = logging.getLogger(__name__)


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
            
            # Collect tokens during generation
            tokens = []
            
            def stream_callback(token: str):
                tokens.append(token)
            
            # Generate UI (with streaming if LLM available)
            ui_spec = self.ui_generator.generate_ui(
                request.message,
                stream_callback=stream_callback if self.ui_generator.use_llm else None
            )
            
            # Send collected tokens
            for token in tokens:
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.TOKEN,
                    content=token,
                    timestamp=int(time.time())
                )
            
            # If no tokens (rule-based), send the full JSON
            if not tokens:
                ui_json = json.dumps(ui_spec.model_dump())
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.TOKEN,
                    content=ui_json,
                    timestamp=int(time.time())
                )
            
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
            
            # Load LLM and create agent
            llm = self.model_loader.load(ModelConfig(
                backend=ModelBackend.OLLAMA,
                size=ModelSize.SMALL,
                streaming=True
            ))
            
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


def serve(port="50052"):
    """Start the gRPC server"""
    # Initialize components
    logger.info("Initializing AI service components...")
    
    model_loader = ModelLoader
    tool_registry = ToolRegistry()
    ui_generator = UIGeneratorAgent(
        tool_registry=tool_registry,
        service_registry=None,  # Not needed for gRPC service
        context_builder=None
    )
    
    # Try to load LLM
    try:
        logger.info("Loading LLM...")
        llm = model_loader.load(ModelConfig(
            backend=ModelBackend.OLLAMA,
            size=ModelSize.SMALL,
            streaming=True
        ))
        ui_generator.llm = llm
        ui_generator.use_llm = True
        logger.info("✅ LLM loaded")
    except Exception as e:
        logger.warning(f"Could not load LLM: {e}. Using rule-based generation.")
    
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

