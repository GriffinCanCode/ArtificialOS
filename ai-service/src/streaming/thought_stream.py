"""
Thought Stream Manager
Handles WebSocket connections and real-time thought streaming
"""

from typing import Dict
from fastapi import WebSocket


class ThoughtStreamManager:
    """Manages active WebSocket connections for thought streaming"""
    
    def __init__(self):
        self.active_connections: Dict[int, WebSocket] = {}
    
    def add_connection(self, connection_id: int, websocket: WebSocket):
        """Register a new WebSocket connection"""
        self.active_connections[connection_id] = websocket
    
    def remove_connection(self, connection_id: int):
        """Remove a WebSocket connection"""
        if connection_id in self.active_connections:
            del self.active_connections[connection_id]
    
    async def broadcast(self, message: dict):
        """Broadcast a message to all connected clients"""
        for connection_id, websocket in list(self.active_connections.items()):
            try:
                await websocket.send_json(message)
            except Exception as e:
                print(f"Error broadcasting to {connection_id}: {e}")
                self.remove_connection(connection_id)
    
    async def send_to(self, connection_id: int, message: dict):
        """Send a message to a specific connection"""
        if connection_id in self.active_connections:
            websocket = self.active_connections[connection_id]
            await websocket.send_json(message)

