"""Chat Handler."""

import time
from collections.abc import AsyncGenerator
import ai_pb2

from core import get_logger, ChatRequest, ValidationError
from agents.chat import ChatAgent, ChatHistory, ChatMessage
from models.loader import ModelLoader
from models.config import GeminiConfig
from monitoring import metrics_collector, trace_operation


logger = get_logger(__name__)


class ChatHandler:
    """Handles chat requests."""

    def __init__(self, model_loader: type[ModelLoader]) -> None:
        self.model_loader = model_loader

    async def stream(self, request: ai_pb2.ChatRequest) -> AsyncGenerator[ai_pb2.ChatToken, None]:
        """Stream chat response."""
        start_time = time.time()

        try:
            validated = ChatRequest(message=request.message, history_count=len(request.history))
            logger.info("chat", message=validated.message[:50])

            with trace_operation("chat_streaming", message=validated.message[:50]):
                # Build history
                history = ChatHistory()
                for msg in request.history:
                    history.add(
                        ChatMessage(role=msg.role, content=msg.content, timestamp=msg.timestamp)
                    )
                history.add(ChatAgent.create_user_message(validated.message))

                # Start
                yield ai_pb2.ChatToken(
                    type=ai_pb2.ChatToken.GENERATION_START, content="", timestamp=int(time.time())
                )

                # Load model
                llm = self.model_loader.load(
                    GeminiConfig(
                        model_name="gemini-2.0-flash-exp",
                        streaming=True,
                        temperature=0.7,
                        max_tokens=2048,
                    )
                )

                # Stream
                agent = ChatAgent(llm)
                tokens = 0
                async for token in agent.stream_response(validated.message, history):
                    tokens += 1
                    metrics_collector.record_stream_message("chat_token")
                    yield ai_pb2.ChatToken(
                        type=ai_pb2.ChatToken.TOKEN, content=token, timestamp=int(time.time())
                    )

                # Track metrics
                duration = time.time() - start_time
                metrics_collector.record_chat_request("success", duration)
                metrics_collector.record_chat_tokens(len(validated.message.split()), tokens)

                logger.info("complete", tokens=tokens, duration_ms=duration * 1000)

                yield ai_pb2.ChatToken(
                    type=ai_pb2.ChatToken.COMPLETE, content="", timestamp=int(time.time())
            )
        except ValidationError as e:
            logger.error("validation_failed", error=str(e))
            yield ai_pb2.ChatToken(
                type=ai_pb2.ChatToken.ERROR, content=f"Validation: {e}", timestamp=int(time.time())
            )
        except Exception as e:
            logger.error("chat_failed", error=str(e), exc_info=True)
            yield ai_pb2.ChatToken(
                type=ai_pb2.ChatToken.ERROR, content=str(e), timestamp=int(time.time())
            )
