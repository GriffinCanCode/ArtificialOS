"""
App Manager - Orchestrates app lifecycle and multi-app coordination
Handles app spawning, state management, and inter-app communication
"""

import logging
import uuid
from typing import Any, Dict, Optional, List
from enum import Enum
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


class AppState(str, Enum):
    """App lifecycle states."""
    SPAWNING = "spawning"      # AI is generating the UI
    ACTIVE = "active"          # App is running
    BACKGROUND = "background"  # App is running but not focused
    SUSPENDED = "suspended"    # App is paused
    DESTROYED = "destroyed"    # App is closed


class AppInstance(BaseModel):
    """Represents a running app instance."""
    
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    title: str = Field(..., description="App title")
    ui_spec: Dict = Field(..., description="UI specification")
    state: AppState = Field(default=AppState.SPAWNING)
    parent_id: Optional[str] = Field(default=None, description="Parent app ID if spawned by another app")
    created_at: float = Field(default_factory=lambda: __import__('time').time())
    metadata: Dict = Field(default_factory=dict, description="Additional metadata")
    services: List[str] = Field(default_factory=list, description="Required service IDs")
    sandbox_pid: Optional[int] = Field(default=None, description="Kernel sandbox process ID")
    
    class Config:
        use_enum_values = True


class AppManager:
    """
    Central orchestrator for app lifecycle management.
    
    Responsibilities:
    - Track all running apps
    - Handle app spawning (from user OR from apps)
    - Manage focus and foreground/background states
    - Coordinate inter-app communication
    - Clean up destroyed apps
    
    Architecture:
    - User/App requests new app → AppManager
    - AppManager delegates to UIGeneratorAgent
    - UIGeneratorAgent returns UISpec (LLM call happens here)
    - AppManager creates AppInstance, stores it
    - DynamicRenderer subscribes to AppManager, renders active apps
    """
    
    def __init__(self, service_registry: Optional[Any] = None, kernel_tools: Optional[Any] = None):
        self.apps: Dict[str, AppInstance] = {}
        self.focused_app_id: Optional[str] = None
        self.service_registry = service_registry
        self.kernel_tools = kernel_tools
        logger.info("AppManager initialized")
    
    def spawn_app(
        self,
        request: str,
        ui_spec: Dict,
        parent_id: Optional[str] = None,
        metadata: Optional[Dict] = None
    ) -> AppInstance:
        """
        Spawn a new app instance with service initialization.
        
        Args:
            request: User's original request (for logging)
            ui_spec: Generated UI specification
            parent_id: ID of parent app if spawned by another app
            metadata: Additional metadata
            
        Returns:
            AppInstance: The newly created app
        """
        title = ui_spec.get("title", "Untitled App")
        services = ui_spec.get("services", [])
        
        # Create sandboxed process if kernel tools available
        sandbox_pid = None
        if self.kernel_tools and services:
            try:
                sandbox_pid = self.kernel_tools.create_sandboxed_process(
                    name=f"app-{title.lower().replace(' ', '-')}",
                    sandbox_level="STANDARD"
                )
                logger.info(f"Created sandbox for app (PID: {sandbox_pid})")
            except Exception as e:
                logger.warning(f"Could not create sandbox: {e}")
        
        app = AppInstance(
            title=title,
            ui_spec=ui_spec,
            state=AppState.ACTIVE,
            parent_id=parent_id,
            metadata=metadata or {"request": request},
            services=services,
            sandbox_pid=sandbox_pid
        )
        
        self.apps[app.id] = app
        self.focused_app_id = app.id
        
        logger.info(
            f"Spawned app: {app.title} (id={app.id}, "
            f"services={len(services)}, sandbox_pid={sandbox_pid})"
        )
        if parent_id:
            logger.info(f"  ↳ Spawned by app: {parent_id}")
        
        return app
    
    def get_app(self, app_id: str) -> Optional[AppInstance]:
        """Get app by ID."""
        return self.apps.get(app_id)
    
    def get_focused_app(self) -> Optional[AppInstance]:
        """Get the currently focused app."""
        if self.focused_app_id:
            return self.apps.get(self.focused_app_id)
        return None
    
    def list_apps(self, state: Optional[AppState] = None) -> List[AppInstance]:
        """
        List all apps, optionally filtered by state.
        
        Args:
            state: Optional state filter
            
        Returns:
            List of app instances
        """
        apps = list(self.apps.values())
        if state:
            apps = [app for app in apps if app.state == state]
        return apps
    
    def focus_app(self, app_id: str) -> bool:
        """
        Focus an app (bring to foreground).
        
        Args:
            app_id: App ID to focus
            
        Returns:
            True if successful, False if app not found
        """
        app = self.apps.get(app_id)
        if not app:
            logger.warning(f"Cannot focus app: {app_id} not found")
            return False
        
        # Unfocus current app
        if self.focused_app_id and self.focused_app_id != app_id:
            current = self.apps.get(self.focused_app_id)
            if current and current.state == AppState.ACTIVE:
                current.state = AppState.BACKGROUND
        
        # Focus new app
        app.state = AppState.ACTIVE
        self.focused_app_id = app_id
        
        logger.info(f"Focused app: {app.title} (id={app_id})")
        return True
    
    def close_app(self, app_id: str) -> bool:
        """
        Close and destroy an app.
        
        Args:
            app_id: App ID to close
            
        Returns:
            True if successful, False if app not found
        """
        app = self.apps.get(app_id)
        if not app:
            logger.warning(f"Cannot close app: {app_id} not found")
            return False
        
        # Close all child apps first
        children = [a for a in self.apps.values() if a.parent_id == app_id]
        for child in children:
            logger.info(f"  ↳ Closing child app: {child.title}")
            self.close_app(child.id)
        
        # Mark as destroyed and remove
        app.state = AppState.DESTROYED
        del self.apps[app_id]
        
        # Update focus
        if self.focused_app_id == app_id:
            self.focused_app_id = None
            # Auto-focus another app if available
            active_apps = [a for a in self.apps.values() if a.state != AppState.DESTROYED]
            if active_apps:
                self.focus_app(active_apps[0].id)
        
        logger.info(f"Closed app: {app.title} (id={app_id})")
        return True
    
    def get_stats(self) -> Dict:
        """Get app manager statistics."""
        return {
            "total_apps": len(self.apps),
            "active_apps": len([a for a in self.apps.values() if a.state == AppState.ACTIVE]),
            "background_apps": len([a for a in self.apps.values() if a.state == AppState.BACKGROUND]),
            "focused_app": self.get_focused_app().title if self.get_focused_app() else None,
        }


# Global singleton instance
_app_manager: Optional[AppManager] = None


def get_app_manager() -> AppManager:
    """Get or create the global AppManager instance."""
    global _app_manager
    if _app_manager is None:
        _app_manager = AppManager()
    return _app_manager


__all__ = ["AppManager", "AppInstance", "AppState", "get_app_manager"]

