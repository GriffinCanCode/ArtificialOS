"""
Model loader - handles Gemini API initialization.
Unified loader for all AI models (now using Gemini exclusively).
"""

import logging
from typing import Optional, AsyncGenerator, Generator, Dict, Any
import google.generativeai as genai

from .config import GeminiConfig

logger = logging.getLogger(__name__)


class GeminiModel:
    """
    Wrapper for Gemini API that provides streaming and structured output.
    Compatible with LangChain-style interface for easy integration.
    """
    
    def __init__(self, config: GeminiConfig):
        """Initialize Gemini model with configuration."""
        self.config = config
        
        # Configure the API
        genai.configure(api_key=config.api_key)
        
        # Initialize the model with generation config
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
        
        logger.info(f"✅ Gemini model initialized: {config.model_name}")
        logger.info(f"Temperature: {config.temperature}, Max tokens: {config.max_tokens}")
    
    def stream(self, prompt: str) -> Generator[str, None, None]:
        """
        Stream tokens from Gemini in real-time.
        
        Args:
            prompt: Input prompt text
            
        Yields:
            Generated text tokens as they arrive
        """
        try:
            logger.debug(f"Streaming from Gemini: prompt length {len(prompt)}")
            
            response = self.model.generate_content(
                prompt,
                stream=True,
            )
            
            for chunk in response:
                if chunk.text:
                    yield chunk.text
                    
        except Exception as e:
            logger.error(f"Gemini streaming error: {e}")
            raise
    
    async def astream(self, prompt: str) -> AsyncGenerator[str, None]:
        """
        Async stream tokens from Gemini.
        
        Args:
            prompt: Input prompt text
            
        Yields:
            Generated text tokens as they arrive
        """
        try:
            logger.debug(f"Async streaming from Gemini: prompt length {len(prompt)}")
            
            # Note: google-generativeai doesn't have native async support yet
            # We'll wrap the sync version for compatibility
            response = self.model.generate_content(
                prompt,
                stream=True,
            )
            
            for chunk in response:
                if chunk.text:
                    yield chunk.text
                    
        except Exception as e:
            logger.error(f"Gemini async streaming error: {e}")
            raise
    
    def stream_json(
        self,
        prompt: str,
        response_schema: Optional[Dict[str, Any]] = None
    ) -> Generator[str, None, None]:
        """
        Stream JSON output from Gemini with structured schema.
        
        Args:
            prompt: Input prompt text
            response_schema: JSON schema for structured output
            
        Yields:
            Generated JSON tokens as they arrive
        """
        try:
            logger.debug(f"Streaming JSON from Gemini with schema: {bool(response_schema)}")
            
            # Configure for JSON output
            generation_config = genai.GenerationConfig(
                temperature=self.config.temperature,
                max_output_tokens=self.config.max_tokens,
                top_p=self.config.top_p,
                top_k=self.config.top_k,
                response_mime_type="application/json",
            )
            
            # Add schema if provided
            if response_schema:
                generation_config.response_schema = response_schema
            
            # Create temporary model with JSON config
            json_model = genai.GenerativeModel(
                model_name=self.config.model_name,
                generation_config=generation_config,
            )
            
            response = json_model.generate_content(
                prompt,
                stream=True,
            )
            
            for chunk in response:
                if chunk.text:
                    yield chunk.text
                    
        except Exception as e:
            logger.error(f"Gemini JSON streaming error: {e}")
            raise
    
    def invoke(self, prompt: str) -> str:
        """
        Non-streaming generation (for compatibility).
        
        Args:
            prompt: Input prompt text
            
        Returns:
            Complete generated text
        """
        try:
            logger.debug(f"Invoking Gemini: prompt length {len(prompt)}")
            
            response = self.model.generate_content(prompt)
            return response.text
            
        except Exception as e:
            logger.error(f"Gemini invoke error: {e}")
            raise
    
    async def ainvoke(self, prompt: str) -> str:
        """
        Async non-streaming generation (for compatibility).
        
        Args:
            prompt: Input prompt text
            
        Returns:
            Complete generated text
        """
        # Wrap sync version for now (google-generativeai doesn't have native async)
        return self.invoke(prompt)


class ModelLoader:
    """
    Manages Gemini model loading and lifecycle.
    Single source of truth for all model loading.
    """
    
    _instance: Optional[GeminiModel] = None
    _config: Optional[GeminiConfig] = None
    
    @classmethod
    def load(
        cls,
        config: GeminiConfig,
        callbacks: Optional[list] = None
    ) -> GeminiModel:
        """
        Load Gemini model with given configuration.
        
        Args:
            config: Gemini configuration
            callbacks: Optional callbacks (ignored, for compatibility)
            
        Returns:
            Initialized Gemini model instance
        """
        # Always create fresh instance for Gemini (no local caching needed)
        # API handles caching and optimization on their end
        logger.info(f"Loading Gemini model: {config.model_name}")
        
        try:
            model = GeminiModel(config)
            
            cls._instance = model
            cls._config = config
            
            logger.info(f"✅ Gemini model loaded successfully")
            return model
            
        except Exception as e:
            logger.error(f"Failed to load Gemini model: {e}")
            raise ModelLoadError(f"Could not load {config.model_name}") from e
    
    @classmethod
    def unload(cls) -> None:
        """Unload model and free resources."""
        if cls._instance is not None:
            logger.info("Unloading Gemini model...")
            cls._instance = None
            cls._config = None
            logger.info("Gemini model unloaded")


class ModelLoadError(Exception):
    """Raised when model fails to load."""
    pass
