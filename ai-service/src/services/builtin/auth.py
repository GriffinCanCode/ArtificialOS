"""
Auth Service
Simple authentication and session management
"""

import logging
import hashlib
import secrets
from typing import Any, Dict, Optional
from datetime import datetime, timedelta

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


class AuthService(BaseService):
    """
    Simple authentication service.
    Manages sessions and basic user authentication.
    """
    
    def __init__(self, context: Optional[ServiceContext] = None):
        super().__init__(context)
        self.sessions: Dict[str, Dict] = {}
        self.users: Dict[str, Dict] = {}
    
    def definition(self) -> Service:
        return Service(
            id="auth",
            name="Auth Service",
            description="Authentication and session management",
            category=ServiceCategory.AUTH,
            version="1.0.0",
            capabilities=[
                "session_management",
                "user_auth",
                "token_generation"
            ],
            tools=[
                Tool(
                    id="auth.session.create",
                    name="Create Session",
                    description="Create new user session",
                    parameters=[
                        ToolParameter(
                            name="user_id",
                            type="string",
                            description="User identifier"
                        ),
                        ToolParameter(
                            name="ttl",
                            type="number",
                            description="Session TTL in seconds",
                            required=False,
                            default=3600
                        )
                    ],
                    returns="string",
                    category="session"
                ),
                Tool(
                    id="auth.session.validate",
                    name="Validate Session",
                    description="Validate session token",
                    parameters=[
                        ToolParameter(
                            name="token",
                            type="string",
                            description="Session token"
                        )
                    ],
                    returns="boolean",
                    category="session"
                ),
                Tool(
                    id="auth.session.destroy",
                    name="Destroy Session",
                    description="Destroy session",
                    parameters=[
                        ToolParameter(
                            name="token",
                            type="string",
                            description="Session token"
                        )
                    ],
                    returns="boolean",
                    category="session"
                ),
            ],
            data_models=[
                DataModel(
                    name="Session",
                    fields={
                        "token": "string",
                        "user_id": "string",
                        "created_at": "number",
                        "expires_at": "number"
                    }
                )
            ],
            requires_kernel=False
        )
    
    async def execute(self, tool_id: str, params: Dict[str, Any]) -> Any:
        """Execute auth tool"""
        
        if tool_id == "auth.session.create":
            return await self._create_session(
                params["user_id"],
                params.get("ttl", 3600)
            )
        
        elif tool_id == "auth.session.validate":
            return await self._validate_session(params["token"])
        
        elif tool_id == "auth.session.destroy":
            return await self._destroy_session(params["token"])
        
        else:
            raise ValueError(f"Unknown tool: {tool_id}")
    
    async def _create_session(self, user_id: str, ttl: int) -> str:
        """Create new session"""
        token = secrets.token_urlsafe(32)
        now = datetime.now().timestamp()
        
        self.sessions[token] = {
            "token": token,
            "user_id": user_id,
            "created_at": now,
            "expires_at": now + ttl
        }
        
        logger.info(f"Created session for user: {user_id}")
        return token
    
    async def _validate_session(self, token: str) -> bool:
        """Validate session token"""
        session = self.sessions.get(token)
        
        if not session:
            return False
        
        # Check expiration
        now = datetime.now().timestamp()
        if now > session["expires_at"]:
            del self.sessions[token]
            return False
        
        return True
    
    async def _destroy_session(self, token: str) -> bool:
        """Destroy session"""
        if token in self.sessions:
            del self.sessions[token]
            logger.info("Destroyed session")
            return True
        
        return False

