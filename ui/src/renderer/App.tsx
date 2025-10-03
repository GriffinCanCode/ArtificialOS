/**
 * Main Application Component
 */

import { useCallback, useEffect } from "react";
import ThoughtStream from "../components/ThoughtStream";
import DynamicRenderer from "../components/DynamicRenderer";
import ChatInterface from "../components/ChatInterface";
import TitleBar from "../components/TitleBar";
import { WebSocketProvider, useWebSocket } from "../contexts/WebSocketContext";
import { useAppActions } from "../store/appStore";
import { useSessionManager } from "../hooks/useSessionManager";
import { ServerMessage } from "../types/api";
import "./App.css";

function App() {
  return (
    <WebSocketProvider>
      <AppContent />
    </WebSocketProvider>
  );
}

function AppContent() {
  const { client } = useWebSocket();
  const { addMessage, addThought, appendToLastMessage } = useAppActions();
  
  // Initialize session manager with auto-save every 30s
  const sessionManager = useSessionManager({
    autoSaveInterval: 30,
    enableAutoSave: true,
    restoreOnMount: true,
  });

  // Handle incoming WebSocket messages with type safety
  const handleMessage = useCallback(
    (message: ServerMessage) => {
      switch (message.type) {
        case "system":
          addMessage({
            type: "system",
            content: message.message,
            timestamp: Date.now(),
          });
          break;

        case "token":
          appendToLastMessage(message.content);
          break;

        case "thought":
          addThought({
            content: message.content,
            timestamp: message.timestamp,
          });
          break;

        case "complete":
          console.log("✨ Generation complete");
          break;

        case "error":
          console.error("❌ Error from server:", message.message);
          addMessage({
            type: "system",
            content: `Error: ${message.message}`,
            timestamp: Date.now(),
          });
          break;

        case "history_update":
          console.log("📜 History updated");
          break;

        default:
          // Other message types handled by DynamicRenderer
          break;
      }
    },
    [addMessage, addThought, appendToLastMessage]
  );

  // Subscribe to WebSocket messages
  useEffect(() => {
    if (!client) return;

    const unsubscribe = client.onMessage(handleMessage);
    return unsubscribe;
  }, [client, handleMessage]);

  return (
    <div className="app">
      <TitleBar 
        sessionManager={sessionManager}
      />

      <div className="app-layout">
        {/* Left Panel: Chat Interface */}
        <div className="panel chat-panel">
          <ChatInterface />
        </div>

        {/* Center Panel: Dynamic App Renderer */}
        <div className="panel renderer-panel">
          <DynamicRenderer />
        </div>

        {/* Right Panel: Thought Stream */}
        <div className="panel thoughts-panel">
          <ThoughtStream />
        </div>
      </div>
      
      {/* Session Status Indicator */}
      {sessionManager.isSaving && (
        <div className="session-status saving">
          💾 Saving...
        </div>
      )}
      {sessionManager.lastSaveTime && !sessionManager.isSaving && (
        <div className="session-status saved">
          ✅ Saved {formatTimeSince(sessionManager.lastSaveTime)}
        </div>
      )}
    </div>
  );
}

// Helper to format time since last save
function formatTimeSince(date: Date): string {
  const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}

export default App;
