"""Dependency Injection Container."""

from injector import Injector, Module, provider, singleton

from models.loader import ModelLoader, GeminiModel
from models.config import GeminiConfig
from agents.ui_generator import UIGenerator
from agents.tools import ToolRegistry
from clients.backend import BackendClient


class CoreModule(Module):
    """Core dependencies."""

    def __init__(self, backend_url: str = "http://localhost:8000") -> None:
        self.backend_url = backend_url

    @singleton
    @provider
    def provide_tool_registry(self) -> ToolRegistry:
        """Provide tool registry singleton."""
        return ToolRegistry()

    @singleton
    @provider
    def provide_backend_client(self) -> BackendClient | None:
        """Provide backend client with service discovery."""
        try:
            client = BackendClient(self.backend_url)
            if client.health_check():
                return client
            return None
        except Exception:
            return None

    @singleton
    @provider
    def provide_gemini_model(self) -> GeminiModel:
        """Provide Gemini model for UI generation."""
        config = GeminiConfig(
            model_name="gemini-2.0-flash-exp",
            streaming=True,
            temperature=0.1,
            max_tokens=4096,
            json_mode=False,
        )
        return ModelLoader.load(config)

    @singleton
    @provider
    def provide_ui_generator(
        self, tool_registry: ToolRegistry, model: GeminiModel, backend: BackendClient | None
    ) -> UIGenerator:
        """Provide UI generator with all dependencies."""
        backend_services = []
        if backend:
            backend_services = backend.discover_services()

        return UIGenerator(
            tool_registry=tool_registry,
            llm=model,
            backend_services=backend_services,
            enable_cache=True,
        )


def create_container(backend_url: str = "http://localhost:8000") -> Injector:
    """Create configured injector."""
    return Injector([CoreModule(backend_url)])
