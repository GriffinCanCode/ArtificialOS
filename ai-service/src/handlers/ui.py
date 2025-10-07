"""UI Handler."""

import time
from typing import Iterator

import ai_pb2

from core import get_logger, UIGenerationRequest, ValidationError
from agents.ui_generator import UIGenerator
from monitoring import metrics_collector, trace_operation


logger = get_logger(__name__)


class UIHandler:
    """Handles UI generation requests."""

    def __init__(self, ui_generator: UIGenerator) -> None:
        self.ui_generator = ui_generator

    def generate(self, request: ai_pb2.UIRequest) -> ai_pb2.UIResponse:
        """Generate UI specification (non-streaming)."""
        start_time = time.time()

        try:
            validated = UIGenerationRequest(
                message=request.message,
                context=dict(request.context) if request.context else {},
            )
            logger.info("ui_generate", message=validated.message[:50])

            with trace_operation("ui_generation", message=validated.message[:50]):
                # Generate UI
                package = self.ui_generator.generate(
                    validated.message, context=validated.context
                )

                # Track metrics
                duration = time.time() - start_time
                metrics_collector.record_ui_request("success", duration, "generate")

                return ai_pb2.UIResponse(
                    app_id=package.app_id,
                    ui_spec=package.model_dump_json(),
                    thoughts=[],  # Non-streaming doesn't track thoughts
                )

        except ValidationError as e:
            duration = time.time() - start_time
            metrics_collector.record_ui_request("validation_error", duration, "generate")
            logger.error("validation", error=str(e))
            raise
        except Exception as e:
            duration = time.time() - start_time
            metrics_collector.record_ui_request("error", duration, "generate")
            metrics_collector.record_error("generation_error", "ui_handler")
            logger.error("generation", error=str(e))
            raise

    def stream(self, request: ai_pb2.UIRequest) -> Iterator[ai_pb2.UIToken]:
        """Stream UI generation."""
        start_time = time.time()

        try:
            validated = UIGenerationRequest(
                message=request.message,
                context=dict(request.context) if request.context else {},
            )
            logger.info("ui_stream", message=validated.message[:50])

            with trace_operation("ui_streaming", message=validated.message[:50]):
                # Start token
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.GENERATION_START,
                    content="",
                    timestamp=int(time.time()),
                )

                # Generate with streaming
                package = self.ui_generator.generate(
                    validated.message, context=validated.context
                )

                # Stream UI spec (in production, this would stream incrementally)
                ui_spec_str = package.model_dump_json()
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.TOKEN,
                    content=ui_spec_str,
                    timestamp=int(time.time()),
                )

                # Complete
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.COMPLETE,
                    content="",
                    timestamp=int(time.time()),
                )

                # Track metrics
                duration = time.time() - start_time
                metrics_collector.record_ui_request("success", duration, "stream")
                metrics_collector.record_stream_message("ui_complete")

                logger.info("complete", duration_ms=duration * 1000)

        except ValidationError as e:
            duration = time.time() - start_time
            metrics_collector.record_ui_request("validation_error", duration, "stream")
            metrics_collector.record_stream_error("validation_error")
            logger.error("validation", error=str(e))
            raise
        except Exception as e:
            duration = time.time() - start_time
            metrics_collector.record_ui_request("error", duration, "stream")
            metrics_collector.record_stream_error("generation_error")
            metrics_collector.record_error("generation_error", "ui_handler")
            logger.error("generation", error=str(e))
            raise
