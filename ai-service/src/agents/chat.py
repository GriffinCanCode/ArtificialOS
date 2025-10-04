"""
Chat agent implementation.
Handles conversational AI with streaming support using Gemini.
"""

import logging
from typing import AsyncGenerator, Optional

from langchain_core.language_models import BaseLLM
from langchain_core.messages import HumanMessage, SystemMessage, AIMessage
from langchain_core.prompts import ChatPromptTemplate, MessagesPlaceholder
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


class ChatMessage(BaseModel):
    """Type-safe chat message."""
    
    role: str = Field(..., pattern="^(user|assistant|system)$")
    content: str = Field(..., min_length=1)
    timestamp: float = Field(...)


class ChatHistory(BaseModel):
    """Manages conversation history."""
    
    messages: list[ChatMessage] = Field(default_factory=list)
    max_history: int = Field(default=20)
    
    def add(self, message: ChatMessage) -> None:
        """Add message and maintain max history."""
        self.messages.append(message)
        if len(self.messages) > self.max_history:
            # Keep system messages, trim oldest user/assistant messages
            system = [m for m in self.messages if m.role == "system"]
            others = [m for m in self.messages if m.role != "system"]
            self.messages = system + others[-self.max_history:]
    
    def to_langchain(self) -> list:
        """Convert to LangChain message format."""
        message_map = {
            "system": SystemMessage,
            "user": HumanMessage,
            "assistant": AIMessage,
        }
        return [
            message_map[msg.role](content=msg.content)
            for msg in self.messages
        ]
    
    def clear(self) -> None:
        """Clear all messages except system."""
        self.messages = [m for m in self.messages if m.role == "system"]


class ChatAgent:
    """
    Conversational AI agent with streaming using Gemini.
    Stateless design - history managed externally.
    """
    
    SYSTEM_PROMPT = """You are a helpful AI assistant."""
    
    def __init__(self, llm: BaseLLM):
        """Initialize with LLM instance (GeminiModel)."""
        self.llm = llm
        self.prompt = self._create_prompt()
    
    @staticmethod
    def _create_prompt() -> ChatPromptTemplate:
        """
        Create chat prompt template.
        Returns None since we'll use simple format for Gemini.
        """
        return None
    
    async def stream_response(
        self,
        user_input: str,
        history: Optional[ChatHistory] = None,
    ) -> AsyncGenerator[str, None]:
        """
        Stream response token by token using Gemini.
        
        Args:
            user_input: User's message
            history: Optional conversation history
            
        Yields:
            Response tokens as they're generated
        """
        history = history or ChatHistory()
        
        # Build conversation context for Gemini
        conversation_context = ""
        for msg in history.messages:
            if msg.role in ["user", "assistant"]:
                role_label = "User" if msg.role == "user" else "Assistant"
                conversation_context += f"{role_label}: {msg.content}\n\n"
        
        # Build simple prompt for Gemini
        prompt_text = f"{self.SYSTEM_PROMPT}\n\n{conversation_context}User: {user_input}"
        
        logger.info(f"Generating chat response for: {user_input[:50]}...")
        
        # Stream tokens from Gemini
        full_response = ""
        async for chunk in self.llm.astream(prompt_text):
            # Handle different response formats
            if hasattr(chunk, 'content'):
                token = chunk.content
            else:
                token = str(chunk)
            
            full_response += token
            yield token
        
        logger.debug(f"Generated {len(full_response)} characters")
    
    async def get_response(
        self,
        user_input: str,
        history: Optional[ChatHistory] = None,
    ) -> str:
        """
        Get complete response (non-streaming).
        
        Args:
            user_input: User's message
            history: Optional conversation history
            
        Returns:
            Complete response text
        """
        history = history or ChatHistory()
        
        # Build conversation context
        conversation_context = ""
        for msg in history.messages:
            if msg.role in ["user", "assistant"]:
                role_label = "User" if msg.role == "user" else "Assistant"
                conversation_context += f"{role_label}: {msg.content}\n\n"
        
        # Build prompt
        prompt_text = f"{self.SYSTEM_PROMPT}\n\n{conversation_context}User: {user_input}"
        
        response = await self.llm.ainvoke(prompt_text)
        
        # Extract content from response
        if hasattr(response, 'content'):
            return response.content
        return str(response)
    
    @staticmethod
    def create_system_message(content: str) -> ChatMessage:
        """Factory for system messages."""
        import time
        return ChatMessage(
            role="system",
            content=content,
            timestamp=time.time()
        )
    
    @staticmethod
    def create_user_message(content: str) -> ChatMessage:
        """Factory for user messages."""
        import time
        return ChatMessage(
            role="user",
            content=content,
            timestamp=time.time()
        )
    
    @staticmethod
    def create_assistant_message(content: str) -> ChatMessage:
        """Factory for assistant messages."""
        import time
        return ChatMessage(
            role="assistant",
            content=content,
            timestamp=time.time()
        )
