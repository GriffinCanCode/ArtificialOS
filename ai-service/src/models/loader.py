"""
Model loader - handles LLM initialization.
Supports GPT-OSS via llama-cpp-python.
"""

import logging
from pathlib import Path
from typing import Optional

from langchain_community.llms import LlamaCpp
from langchain_ollama import ChatOllama
from langchain_core.language_models import BaseLLM

from .config import ModelConfig, ModelSize, ModelBackend

logger = logging.getLogger(__name__)


class ModelLoader:
    """Manages model loading and lifecycle."""
    
    _instance: Optional[BaseLLM] = None
    _config: Optional[ModelConfig] = None
    
    @classmethod
    def load(
        cls,
        config: ModelConfig,
        callbacks: Optional[list] = None
    ) -> BaseLLM:
        """
        Load model with given configuration.
        
        Args:
            config: Model configuration
            callbacks: Optional LangChain callbacks
            
        Returns:
            Initialized LLM instance
        """
        # Cache model instance (singleton pattern for resource efficiency)
        if cls._instance is not None and cls._config == config:
            logger.info("Returning cached model instance")
            return cls._instance
        
        backend_name = config.backend if isinstance(config.backend, str) else config.backend.value
        logger.info(f"Loading {config.model_name} via {backend_name}...")
        
        try:
            if config.backend == ModelBackend.OLLAMA:
                llm = cls._load_ollama(config, callbacks)
            else:
                llm = cls._load_llama_cpp(config, callbacks)
            
            cls._instance = llm
            cls._config = config
            
            logger.info(f"✅ Model loaded: {config.model_name}")
            logger.info(f"Backend: {backend_name}")
            
            return llm
            
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise ModelLoadError(f"Could not load {config.model_name}") from e
    
    @classmethod
    def _load_ollama(
        cls,
        config: ModelConfig,
        callbacks: Optional[list] = None
    ) -> BaseLLM:
        """Load model via Ollama."""
        size_str = config.size if isinstance(config.size, str) else config.size.value
        model_name = f"gpt-oss:{size_str.split('-')[-1]}"  # gpt-oss:20b
        
        logger.info(f"Connecting to Ollama: {model_name}")
        
        # Note: ChatOllama doesn't support streaming parameter in constructor
        # It handles streaming automatically based on how you call it
        # (.stream() vs .invoke())
        return ChatOllama(
            model=model_name,
            base_url=config.ollama_base_url,
            temperature=config.temperature,
            num_predict=config.max_tokens,
            callbacks=callbacks or [],
        )
    
    @classmethod
    def _load_llama_cpp(
        cls,
        config: ModelConfig,
        callbacks: Optional[list] = None
    ) -> BaseLLM:
        """Load model via llama-cpp-python."""
        model_path = cls._resolve_model_path(config)
        
        logger.info(f"Loading GGUF from: {model_path}")
        
        try:
            llm = LlamaCpp(
                model_path=str(model_path),
                n_ctx=config.context_length,
                n_batch=config.batch_size,
                n_gpu_layers=config.gpu_layers,
                n_threads=config.threads,
                temperature=config.temperature,
                top_p=config.top_p,
                max_tokens=config.max_tokens,
                streaming=config.streaming,
                callbacks=callbacks or [],
                seed=config.seed,
                verbose=False,  # Use logger instead
                # Stop tokens to prevent rambling
                stop=["### Instruction:", "\n\n\n", "User:", "###"],
                repeat_penalty=1.1,
            )
            
            logger.info(f"Context: {config.context_length}, GPU layers: {config.gpu_layers}")
            return llm
            
        except Exception as e:
            raise ModelLoadError(f"Could not load GGUF model") from e
    
    @classmethod
    def unload(cls) -> None:
        """Unload model and free resources."""
        if cls._instance is not None:
            logger.info("Unloading model...")
            
            # Explicit cleanup for llama-cpp models
            if hasattr(cls._instance, '_model'):
                try:
                    # LlamaCpp has internal cleanup
                    logger.info("Freeing llama-cpp model memory...")
                    del cls._instance._model
                except Exception as e:
                    logger.warning(f"Error freeing llama-cpp model: {e}")
            
            # Clear instance references
            cls._instance = None
            cls._config = None
            
            # Force garbage collection to free memory immediately
            import gc
            gc.collect()
            logger.info("Model unloaded and memory freed")
    
    @staticmethod
    def _resolve_model_path(config: ModelConfig) -> Path:
        """
        Resolve model file path.
        
        Priority:
        1. Explicit config.model_path
        2. Models directory with standard naming
        3. Hugging Face cache (future)
        """
        if config.model_path:
            return config.model_path
        
        # Check standard models directory
        models_dir = Path(__file__).parent.parent.parent / "models"
        
        # Standard GGUF naming convention
        model_files = {
            ModelSize.SMALL: [
                "gpt-oss-20b.gguf",
                "gpt-oss-20b-Q4_K_M.gguf",
                "gpt-oss-20b-Q8_0.gguf",
            ],
            ModelSize.LARGE: [
                "gpt-oss-120b.gguf",
                "gpt-oss-120b-Q4_K_M.gguf",
                "gpt-oss-120b-Q8_0.gguf",
            ],
        }
        
        for filename in model_files.get(config.size, []):
            path = models_dir / filename
            if path.exists():
                logger.info(f"Found model: {path}")
                return path
        
        # Provide helpful error message
        raise ModelNotFoundError(
            f"Model {config.model_name} not found. "
            f"Please download GGUF model to: {models_dir}\n"
            "Download from: https://huggingface.co/openai/gpt-oss"
        )


class ModelLoadError(Exception):
    """Raised when model fails to load."""
    pass


class ModelNotFoundError(FileNotFoundError):
    """Raised when model file not found."""
    pass

