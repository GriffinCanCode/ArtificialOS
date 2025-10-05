"""Chat Agent."""

from collections.abc import AsyncGenerator
from pydantic import BaseModel, Field
from langchain_core.language_models import BaseLLM

from .prompts import PromptBuilder


class ChatMessage(BaseModel):
    """Chat message."""
    role: str = Field(..., pattern="^(user|assistant|system)$")
    content: str = Field(..., min_length=1)
    timestamp: float


class ChatHistory(BaseModel):
    """Conversation history."""
    messages: list[ChatMessage] = Field(default_factory=list)
    max_history: int = 20

    def add(self, message: ChatMessage) -> None:
        """Add message with limit enforcement."""
        self.messages.append(message)
        if len(self.messages) > self.max_history:
            system = [m for m in self.messages if m.role == "system"]
            others = [m for m in self.messages if m.role != "system"]
            self.messages = system + others[-self.max_history:]

    def clear(self) -> None:
        """Clear non-system messages."""
        self.messages = [m for m in self.messages if m.role == "system"]


class ChatAgent:
    """Conversational AI agent."""

    SYSTEM_PROMPT = "You are a helpful AI assistant."

    def __init__(self, llm: BaseLLM) -> None:
        self.llm = llm

    async def stream_response(
        self,
        user_input: str,
        history: ChatHistory | None = None
    ) -> AsyncGenerator[str, None]:
        """Stream response tokens."""
        history = history or ChatHistory()

        # Build prompt
        messages = [(m.role, m.content) for m in history.messages if m.role in ["user", "assistant"]]
        prompt = PromptBuilder.build_conversation(self.SYSTEM_PROMPT, messages, user_input)

        # Stream
        async for chunk in self.llm.astream(prompt):
            token = chunk.content if hasattr(chunk, 'content') else str(chunk)
            yield token

    async def get_response(self, user_input: str, history: ChatHistory | None = None) -> str:
        """Get complete response."""
        history = history or ChatHistory()
        messages = [(m.role, m.content) for m in history.messages if m.role in ["user", "assistant"]]
        prompt = PromptBuilder.build_conversation(self.SYSTEM_PROMPT, messages, user_input)
        response = await self.llm.ainvoke(prompt)
        return response.content if hasattr(response, 'content') else str(response)

    @staticmethod
    def create_system_message(content: str) -> ChatMessage:
        """Create system message."""
        import time
        return ChatMessage(role="system", content=content, timestamp=time.time())

    @staticmethod
    def create_user_message(content: str) -> ChatMessage:
        """Create user message."""
        import time
        return ChatMessage(role="user", content=content, timestamp=time.time())

    @staticmethod
    def create_assistant_message(content: str) -> ChatMessage:
        """Create assistant message."""
        import time
        return ChatMessage(role="assistant", content=content, timestamp=time.time())
