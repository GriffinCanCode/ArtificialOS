"""
Custom LangChain streaming callbacks.
Bridges LLM output to WebSocket clients.
"""

import asyncio
import logging
import time
from typing import Any, Dict, Optional

from langchain_core.callbacks import AsyncCallbackHandler
from fastapi import WebSocket

logger = logging.getLogger(__name__)


class StreamCallback(AsyncCallbackHandler):
    """
    Streams LLM tokens and thoughts to WebSocket.
    Implements LangChain's async callback interface.
    """
    
    def __init__(self, websocket: WebSocket):
        """Initialize with WebSocket connection."""
        self.ws = websocket
        self.token_count = 0
        self.start_time: Optional[float] = None
    
    async def on_llm_start(
        self,
        serialized: Dict[str, Any],
        prompts: list[str],
        **kwargs: Any,
    ) -> None:
        """Called when LLM starts generating."""
        self.start_time = time.time()
        self.token_count = 0
        
        logger.debug("LLM generation started")
        
        await self._send({
            "type": "generation_start",
            "timestamp": self.start_time,
            "prompts": len(prompts),
        })
    
    async def on_llm_new_token(
        self,
        token: str,
        **kwargs: Any,
    ) -> None:
        """Called for each new token generated."""
        self.token_count += 1
        
        await self._send({
            "type": "token",
            "content": token,
            "index": self.token_count,
            "timestamp": time.time(),
        })
    
    async def on_llm_end(
        self,
        response: Any,
        **kwargs: Any,
    ) -> None:
        """Called when LLM finishes generating."""
        duration = time.time() - self.start_time if self.start_time else 0
        tokens_per_sec = self.token_count / duration if duration > 0 else 0
        
        logger.info(
            f"Generation complete: {self.token_count} tokens "
            f"in {duration:.2f}s ({tokens_per_sec:.1f} tok/s)"
        )
        
        await self._send({
            "type": "generation_end",
            "timestamp": time.time(),
            "token_count": self.token_count,
            "duration": duration,
            "tokens_per_second": tokens_per_sec,
        })
    
    async def on_llm_error(
        self,
        error: BaseException,
        **kwargs: Any,
    ) -> None:
        """Called when LLM encounters an error."""
        logger.error(f"LLM error: {error}")
        
        await self._send({
            "type": "error",
            "message": str(error),
            "timestamp": time.time(),
        })
    
    async def on_chain_start(
        self,
        serialized: Dict[str, Any],
        inputs: Dict[str, Any],
        **kwargs: Any,
    ) -> None:
        """Called when a chain/agent starts."""
        chain_name = serialized.get("name", "unknown")
        
        logger.debug(f"Chain started: {chain_name}")
        
        await self._send({
            "type": "thought",
            "content": f"Starting {chain_name}...",
            "timestamp": time.time(),
        })
    
    async def on_chain_end(
        self,
        outputs: Dict[str, Any],
        **kwargs: Any,
    ) -> None:
        """Called when a chain/agent completes."""
        logger.debug("Chain completed")
    
    async def on_tool_start(
        self,
        serialized: Dict[str, Any],
        input_str: str,
        **kwargs: Any,
    ) -> None:
        """Called when agent uses a tool."""
        tool_name = serialized.get("name", "unknown")
        
        logger.debug(f"Tool called: {tool_name}")
        
        await self._send({
            "type": "thought",
            "content": f"Using tool: {tool_name}",
            "tool": tool_name,
            "timestamp": time.time(),
        })
    
    async def on_tool_end(
        self,
        output: str,
        **kwargs: Any,
    ) -> None:
        """Called when tool execution completes."""
        logger.debug("Tool execution complete")
        
        await self._send({
            "type": "thought",
            "content": "Tool execution complete",
            "timestamp": time.time(),
        })
    
    async def _send(self, message: dict) -> None:
        """Send message to WebSocket with error handling."""
        try:
            await self.ws.send_json(message)
        except Exception as e:
            logger.warning(f"Failed to send WebSocket message: {e}")


class ThoughtCallback(AsyncCallbackHandler):
    """
    Specialized callback for capturing reasoning steps.
    Used for chain-of-thought visualization.
    """
    
    def __init__(self, websocket: WebSocket):
        """Initialize with WebSocket connection."""
        self.ws = websocket
        self.thought_buffer: list[str] = []
    
    async def on_text(
        self,
        text: str,
        **kwargs: Any,
    ) -> None:
        """Capture intermediate reasoning text."""
        self.thought_buffer.append(text)
        
        await self.ws.send_json({
            "type": "reasoning",
            "content": text,
            "timestamp": time.time(),
        })
    
    def get_thoughts(self) -> list[str]:
        """Retrieve all captured thoughts."""
        return self.thought_buffer.copy()
    
    def clear(self) -> None:
        """Clear thought buffer."""
        self.thought_buffer.clear()

