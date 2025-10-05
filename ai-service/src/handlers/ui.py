"""UI Generation Handler - Optimized with fast JSON."""

import time
import ai_pb2

from core import get_logger, UIGenerationRequest, ValidationError, BlueprintValidator, safe_json_dumps
from agents.ui_generator import UIGenerator


logger = get_logger(__name__)


class UIHandler:
    """Handles UI generation."""
    
    def __init__(self, ui_generator: UIGenerator):
        self.ui_generator = ui_generator
    
    def generate(self, request: ai_pb2.UIRequest) -> ai_pb2.UIResponse:
        """Generate UI spec."""
        try:
            validated = UIGenerationRequest(message=request.message)
            logger.info("generate", message=validated.message[:50])
            
            ui_spec = self.ui_generator.generate_ui(validated.message)
            spec_dict = ui_spec.model_dump()
            
            # Use orjson for fast serialization (2-3x faster)
            ui_json = safe_json_dumps(spec_dict, indent=2)
            
            # Validate before sending
            BlueprintValidator.validate(spec_dict, ui_json)
            
            return ai_pb2.UIResponse(
                app_id="",
                ui_spec_json=ui_json,
                thoughts=[
                    f"Request: {validated.message[:50]}",
                    f"Title: {ui_spec.title}",
                    f"Components: {len(ui_spec.components)}"
                ],
                success=True
            )
        except ValidationError as e:
            logger.error("validation_failed", error=str(e))
            return ai_pb2.UIResponse(
                app_id="", ui_spec_json="", thoughts=[], success=False,
                error=f"Validation: {e}"
            )
        except Exception as e:
            logger.error("generation_failed", error=str(e), exc_info=True)
            return ai_pb2.UIResponse(
                app_id="", ui_spec_json="", thoughts=[], success=False,
                error=str(e)
            )
    
    def stream(self, request: ai_pb2.UIRequest):
        """Stream UI generation."""
        try:
            validated = UIGenerationRequest(message=request.message)
            logger.info("stream", message=validated.message[:50])
            
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.GENERATION_START,
                content="Analyzing...",
                timestamp=int(time.time())
            )
            
            ui_spec = None
            tokens = 0
            
            for item in self.ui_generator.generate_ui_stream(validated.message):
                if isinstance(item, dict) and item.get("reset"):
                    logger.info("fallback")
                    yield ai_pb2.UIToken(
                        type=ai_pb2.UIToken.GENERATION_START,
                        content="Rule-based generation...",
                        timestamp=int(time.time())
                    )
                elif isinstance(item, str):
                    tokens += 1
                    # Send tokens for incremental parsing
                    yield ai_pb2.UIToken(
                        type=ai_pb2.UIToken.TOKEN,
                        content=item,
                        timestamp=int(time.time())
                    )
                else:
                    ui_spec = item
            
            logger.info("complete", tokens=tokens)
            
            if ui_spec:
                yield ai_pb2.UIToken(
                    type=ai_pb2.UIToken.THOUGHT,
                    content=f"{len(ui_spec.components)} components",
                    timestamp=int(time.time())
                )
                
                # Validate the final spec
                spec_dict = ui_spec.model_dump()
                ui_json = safe_json_dumps(spec_dict, indent=2)
                BlueprintValidator.validate(spec_dict, ui_json)
            
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.COMPLETE,
                content="",
                timestamp=int(time.time())
            )
        except Exception as e:
            logger.error("stream_failed", error=str(e), exc_info=True)
            yield ai_pb2.UIToken(
                type=ai_pb2.UIToken.ERROR,
                content=str(e),
                timestamp=int(time.time())
            )
