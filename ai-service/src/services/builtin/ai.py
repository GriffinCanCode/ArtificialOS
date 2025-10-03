"""
AI Service
LLM capabilities for app features
"""

import logging
from typing import Any, Dict, Optional, List

from ..base import BaseService
from ..types import (
    Service,
    ServiceCategory,
    Tool,
    ToolParameter,
    DataModel,
    ServiceContext,
)

logger = logging.getLogger(__name__)


class AIService(BaseService):
    """
    AI service providing LLM capabilities.
    Self-reference to enable AI features within apps.
    """
    
    def __init__(
        self,
        context: Optional[ServiceContext] = None,
        llm: Optional[Any] = None
    ):
        super().__init__(context)
        self.llm = llm
    
    def definition(self) -> Service:
        return Service(
            id="ai",
            name="AI Service",
            description="LLM capabilities for intelligent features",
            category=ServiceCategory.AI,
            version="1.0.0",
            capabilities=[
                "text_completion",
                "chat",
                "classification",
                "summarization",
                "embeddings"
            ],
            tools=[
                Tool(
                    id="ai.complete",
                    name="Text Completion",
                    description="Generate text completion",
                    parameters=[
                        ToolParameter(
                            name="prompt",
                            type="string",
                            description="Prompt text"
                        ),
                        ToolParameter(
                            name="max_tokens",
                            type="number",
                            description="Max tokens to generate",
                            required=False,
                            default=100
                        )
                    ],
                    returns="string",
                    category="generation"
                ),
                Tool(
                    id="ai.chat",
                    name="Chat Completion",
                    description="Chat with context",
                    parameters=[
                        ToolParameter(
                            name="messages",
                            type="array",
                            description="Chat messages"
                        )
                    ],
                    returns="string",
                    category="generation"
                ),
                Tool(
                    id="ai.classify",
                    name="Text Classification",
                    description="Classify text into categories",
                    parameters=[
                        ToolParameter(
                            name="text",
                            type="string",
                            description="Text to classify"
                        ),
                        ToolParameter(
                            name="categories",
                            type="array",
                            description="Possible categories"
                        )
                    ],
                    returns="string",
                    category="analysis"
                ),
                Tool(
                    id="ai.summarize",
                    name="Summarize Text",
                    description="Generate text summary",
                    parameters=[
                        ToolParameter(
                            name="text",
                            type="string",
                            description="Text to summarize"
                        ),
                        ToolParameter(
                            name="max_length",
                            type="number",
                            description="Max summary length",
                            required=False,
                            default=100
                        )
                    ],
                    returns="string",
                    category="analysis"
                ),
            ],
            data_models=[
                DataModel(
                    name="ChatMessage",
                    fields={
                        "role": "string",
                        "content": "string"
                    }
                ),
                DataModel(
                    name="Classification",
                    fields={
                        "category": "string",
                        "confidence": "number"
                    }
                )
            ],
            requires_kernel=False
        )
    
    async def execute(self, tool_id: str, params: Dict[str, Any]) -> Any:
        """Execute AI tool"""
        
        if tool_id == "ai.complete":
            return await self._complete(
                params["prompt"],
                params.get("max_tokens", 100)
            )
        
        elif tool_id == "ai.chat":
            return await self._chat(params["messages"])
        
        elif tool_id == "ai.classify":
            return await self._classify(
                params["text"],
                params["categories"]
            )
        
        elif tool_id == "ai.summarize":
            return await self._summarize(
                params["text"],
                params.get("max_length", 100)
            )
        
        else:
            raise ValueError(f"Unknown tool: {tool_id}")
    
    async def _complete(self, prompt: str, max_tokens: int) -> str:
        """Generate text completion"""
        if not self.llm:
            return f"[AI completion for: {prompt[:50]}...]"
        
        # Use LLM streaming
        response = ""
        for chunk in self.llm.stream(prompt):
            if hasattr(chunk, 'content'):
                response += chunk.content
        
        return response[:max_tokens]
    
    async def _chat(self, messages: List[Dict]) -> str:
        """Chat completion"""
        if not self.llm:
            return "[AI chat response]"
        
        # Format messages into prompt
        prompt = "\n".join(
            f"{msg['role']}: {msg['content']}"
            for msg in messages
        )
        
        return await self._complete(prompt, 500)
    
    async def _classify(self, text: str, categories: List[str]) -> str:
        """Classify text"""
        prompt = f"""Classify the following text into one of these categories:
{', '.join(categories)}

Text: {text}

Category:"""
        
        result = await self._complete(prompt, 20)
        
        # Extract first category mentioned
        for cat in categories:
            if cat.lower() in result.lower():
                return cat
        
        return categories[0] if categories else "unknown"
    
    async def _summarize(self, text: str, max_length: int) -> str:
        """Summarize text"""
        prompt = f"""Summarize this text in {max_length} words or less:

{text}

Summary:"""
        
        return await self._complete(prompt, max_length)

