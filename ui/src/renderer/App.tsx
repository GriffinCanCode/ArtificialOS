/**
 * Main Application Component
 */

import React, { useCallback, useEffect } from "react";
import ThoughtStream from "../components/ThoughtStream";
import DynamicRenderer from "../components/DynamicRenderer";
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
  const { client, generateUI } = useWebSocket();
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

  const [showThoughts, setShowThoughts] = React.useState(false);
  const [inputValue, setInputValue] = React.useState("");
  const [inputFocused, setInputFocused] = React.useState(false);
  const inputRef = React.useRef<HTMLInputElement>(null);

  // Global keyboard shortcut: Cmd/Ctrl + K to focus input
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setInputFocused(true);
        // Focus the input field
        setTimeout(() => inputRef.current?.focus(), 50);
      }
      // Escape to blur input
      if (e.key === "Escape" && inputFocused) {
        setInputFocused(false);
        inputRef.current?.blur();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [inputFocused]);

  const handleSendMessage = (message: string) => {
    if (message.trim()) {
      // Send to AI for UI generation using the context method
      generateUI(message, {});
      setInputValue("");
    }
  };

  return (
    <div className="app os-interface">
      {/* Minimal Title Bar - just window controls */}
      <TitleBar sessionManager={sessionManager} />

      {/* Full-screen App Canvas */}
      <div className="os-canvas">
        <DynamicRenderer />
      </div>

      {/* Floating Spotlight-style Input Bar */}
      <div className={`spotlight-input-container ${inputFocused ? "focused" : ""}`}>
        <div className="spotlight-input-wrapper">
          <div className="spotlight-icon">✨</div>
          <input
            ref={inputRef}
            type="text"
            className="spotlight-input"
            placeholder="Ask AI to create something... (⌘K)"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onFocus={() => setInputFocused(true)}
            onBlur={(e) => {
              // Don't blur if clicking the send button
              if (e.relatedTarget?.classList.contains("spotlight-send")) {
                return;
              }
              setInputFocused(false);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && inputValue.trim()) {
                handleSendMessage(inputValue);
              }
            }}
          />
          {inputValue && (
            <button
              className="spotlight-send"
              onMouseDown={(e) => {
                // Prevent blur on click
                e.preventDefault();
                handleSendMessage(inputValue);
              }}
              aria-label="Send message"
            >
              →
            </button>
          )}
        </div>
        {inputFocused && (
          <div className="spotlight-hint">Press Enter to generate • Esc to close</div>
        )}
      </div>

      {/* Thought Stream - Slide-out Notification Panel */}
      <ThoughtStream isVisible={showThoughts} onToggle={() => setShowThoughts(!showThoughts)} />

      {/* Session Status Indicator */}
      {sessionManager.isSaving && <div className="session-status saving">💾 Saving...</div>}
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
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}

export default App;
