"""Model Loader - Gemini API with streaming."""

import asyncio
from typing import Optional, AsyncGenerator, Generator
import google.generativeai as genai

from core import get_logger
from .config import GeminiConfig


logger = get_logger(__name__)


class ModelLoadError(Exception):
    """Model loading failed."""
    pass


class GeminiModel:
    """Gemini API wrapper with streaming support."""
    
    def __init__(self, config: GeminiConfig):
        self.config = config
        genai.configure(api_key=config.api_key)
        
        generation_config = genai.GenerationConfig(
            temperature=config.temperature,
            max_output_tokens=config.max_tokens,
            top_p=config.top_p,
            top_k=config.top_k,
        )
        
        self.model = genai.GenerativeModel(
            model_name=config.model_name,
            generation_config=generation_config,
        )
        
        logger.info("model_loaded", model=config.model_name)
    
    def stream(self, prompt: str) -> Generator[str, None, None]:
        """Stream tokens."""
        try:
            response = self.model.generate_content(prompt, stream=True)
            for chunk in response:
                if chunk.text:
                    yield chunk.text
        except Exception as e:
            logger.error("stream_error", error=str(e))
            raise
    
    async def astream(self, prompt: str) -> AsyncGenerator[str, None]:
        """Async stream tokens (runs sync API in thread pool)."""
        try:
            # Run blocking sync API in thread to avoid blocking event loop
            loop = asyncio.get_event_loop()
            
            def _sync_generator():
                """Wrapper to collect sync generator results."""
                response = self.model.generate_content(prompt, stream=True)
                for chunk in response:
                    if chunk.text:
                        yield chunk.text
            
            # Convert sync generator to async by running in executor
            gen = _sync_generator()
            while True:
                try:
                    chunk = await loop.run_in_executor(None, next, gen, StopIteration)
                    if chunk is StopIteration:
                        break
                    yield chunk
                except StopIteration:
                    break
        except Exception as e:
            logger.error("astream_error", error=str(e))
            raise
    
    def stream_json(self, prompt: str) -> Generator[str, None, None]:
        """Stream JSON output."""
        try:
            generation_config = genai.GenerationConfig(
                temperature=self.config.temperature,
                max_output_tokens=self.config.max_tokens,
                response_mime_type="application/json",
            )
            
            json_model = genai.GenerativeModel(
                model_name=self.config.model_name,
                generation_config=generation_config,
            )
            
            response = json_model.generate_content(prompt, stream=True)
            for chunk in response:
                if chunk.text:
                    yield chunk.text
        except Exception as e:
            logger.error("json_stream_error", error=str(e))
            raise
    
    def invoke(self, prompt: str) -> str:
        """Non-streaming generation."""
        try:
            response = self.model.generate_content(prompt)
            return response.text
        except Exception as e:
            logger.error("invoke_error", error=str(e))
            raise
    
    async def ainvoke(self, prompt: str) -> str:
        """Async non-streaming generation (runs sync API in thread pool)."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.invoke, prompt)


class ModelLoader:
    """Model lifecycle manager."""
    
    _instance: Optional[GeminiModel] = None
    
    @classmethod
    def load(cls, config: GeminiConfig) -> GeminiModel:
        """Load model with config."""
        logger.info("loading", model=config.model_name)
        try:
            model = GeminiModel(config)
            cls._instance = model
            return model
        except Exception as e:
            logger.error("load_failed", error=str(e))
            raise ModelLoadError(f"Failed to load {config.model_name}") from e
    
    @classmethod
    def unload(cls) -> None:
        """Unload model."""
        if cls._instance:
            logger.info("unloading")
            cls._instance = None
