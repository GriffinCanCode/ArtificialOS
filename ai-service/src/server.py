"""
gRPC Server
Modern async gRPC server with grpc.aio.
"""

import asyncio
import signal
from typing import AsyncIterator

import grpc

import ai_pb2
import ai_pb2_grpc

from core import configure_logging, get_logger, get_settings, create_container
from handlers import UIHandler, ChatHandler
from models.loader import ModelLoader
from agents.ui_generator import UIGenerator


logger = get_logger(__name__)


class AsyncAIService(ai_pb2_grpc.AIServiceServicer):
    """Async AI Service with native grpc.aio."""
    
    def __init__(self, ui_handler: UIHandler, chat_handler: ChatHandler):
        self.ui_handler = ui_handler
        self.chat_handler = chat_handler
        logger.info("service_ready")
    
    async def GenerateUI(
        self,
        request: ai_pb2.UIRequest,
        context: grpc.aio.ServicerContext
    ) -> ai_pb2.UIResponse:
        """Generate UI specification (non-streaming)."""
        # Run sync handler in executor to avoid blocking
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.ui_handler.generate, request)
    
    async def StreamUI(
        self,
        request: ai_pb2.UIRequest,
        context: grpc.aio.ServicerContext
    ) -> AsyncIterator[ai_pb2.UIToken]:
        """Stream UI generation (async)."""
        # Run sync generator in thread pool
        loop = asyncio.get_event_loop()
        
        def sync_stream():
            return list(self.ui_handler.stream(request))
        
        tokens = await loop.run_in_executor(None, sync_stream)
        for token in tokens:
            yield token
    
    async def StreamChat(
        self,
        request: ai_pb2.ChatRequest,
        context: grpc.aio.ServicerContext
    ) -> AsyncIterator[ai_pb2.ChatToken]:
        """Stream chat response (fully async)."""
        async for token in self.chat_handler.stream(request):
            yield token


async def serve_async():
    """Start async gRPC server."""
    settings = get_settings()
    configure_logging(settings.log_level, settings.json_logs)
    
    logger.info("starting", port=settings.grpc_port)
    
    # Resolve dependencies
    container = create_container(settings.backend_url)
    ui_generator = container.get(UIGenerator)
    
    # Create handlers
    ui_handler = UIHandler(ui_generator)
    chat_handler = ChatHandler(ModelLoader)
    service = AsyncAIService(ui_handler, chat_handler)
    
    # Create async server with keepalive options
    server = grpc.aio.server(
        options=[
            # Keepalive settings to match client expectations
            ('grpc.keepalive_time_ms', 10000),  # 10 seconds
            ('grpc.keepalive_timeout_ms', 3000),  # 3 seconds
            ('grpc.keepalive_permit_without_calls', 1),  # Allow pings without active RPCs
            ('grpc.http2.min_time_between_pings_ms', 10000),  # Minimum 10s between pings
            ('grpc.http2.max_pings_without_data', 0),  # Allow unlimited pings without data
            # Message size limits
            ('grpc.max_send_message_length', 50 * 1024 * 1024),  # 50MB
            ('grpc.max_receive_message_length', 10 * 1024 * 1024),  # 10MB
        ]
    )
    ai_pb2_grpc.add_AIServiceServicer_to_server(service, server)
    
    # Listen
    address = f"[::]:{settings.grpc_port}"
    server.add_insecure_port(address)
    
    logger.info("listening", address=address)
    
    # Start server
    await server.start()
    
    # Setup graceful shutdown
    async def shutdown(sig):
        logger.info("shutdown_signal", signal=sig.name)
        await server.stop(grace=5)
        ModelLoader.unload()
        logger.info("stopped")
    
    # Handle signals
    loop = asyncio.get_event_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(
            sig,
            lambda s=sig: asyncio.create_task(shutdown(s))
        )
    
    # Wait for termination
    await server.wait_for_termination()


def serve():
    """Entry point - run async server."""
    asyncio.run(serve_async())


if __name__ == "__main__":
    serve()
