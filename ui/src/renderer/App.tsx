/**
 * Main Application Component
 */

import React, { useCallback, useEffect } from "react";
import { useForm } from "react-hook-form";
import { Save, Sparkles, CheckCircle } from "lucide-react";
import { QueryClientProvider } from "@tanstack/react-query";
import ThoughtStream from "../components/chat/ThoughtStream";
import DynamicRenderer from "../components/DynamicRenderer";
import TitleBar from "../components/layout/TitleBar";
import { WebSocketProvider, useWebSocket } from "../contexts/WebSocketContext";
import { useAppActions } from "../store/appStore";
import { useSessionManager } from "../hooks/useSessionManager";
import { ServerMessage } from "../types/api";
import { useLogger } from "../utils/monitoring/useLogger";
import { useFadeIn, useSlideInUp } from "../hooks/useGSAP";
import { queryClient } from "../lib/queryClient";
import "./App.css";

interface SpotlightFormData {
  prompt: string;
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <WebSocketProvider>
        <AppContent />
      </WebSocketProvider>
    </QueryClientProvider>
  );
}

function AppContent() {
  const log = useLogger("AppContent");
  const { client, generateUI } = useWebSocket();
  const { addMessage, addThought, appendToLastMessage } = useAppActions();

  // Initialize session manager with auto-save every 30s
  // Memoize options to prevent hooks order issues during HMR
  const sessionManagerOptions = React.useMemo(() => ({
    autoSaveInterval: 30,
    enableAutoSave: true,
    restoreOnMount: true,
  }), []);
  
  const sessionManager = useSessionManager(sessionManagerOptions);

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
          log.info("Generation complete");
          break;

        case "error":
          log.error("Error from server", undefined, { message: message.message });
          addMessage({
            type: "system",
            content: `Error: ${message.message}`,
            timestamp: Date.now(),
          });
          break;

        case "history_update":
          log.debug("History updated");
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
  const [inputFocused, setInputFocused] = React.useState(false);
  
  // React Hook Form for spotlight input
  const { register, handleSubmit: handleFormSubmit, reset, watch } = useForm<SpotlightFormData>({
    defaultValues: { prompt: "" },
  });

  const inputValue = watch("prompt");
  
  // Get ref from register for keyboard shortcut access
  const { ref: inputRefCallback, ...inputRegisterProps } = register("prompt", {
    required: true,
    validate: (value) => value.trim().length > 0,
    onBlur: (e) => {
      // Don't blur if clicking the send button
      if (e.relatedTarget?.classList.contains("spotlight-send")) {
        return;
      }
      setInputFocused(false);
    },
  });
  
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  
  // Combine refs for both react-hook-form and keyboard shortcut
  const setInputRef = React.useCallback(
    (node: HTMLInputElement | null) => {
      inputRefCallback(node);
      if (inputRef) {
        inputRef.current = node;
      }
    },
    [inputRefCallback]
  );
  
  // GSAP Animation hooks
  const spotlightContainerRef = useFadeIn<HTMLDivElement>({ duration: 0.3 });
  const hintRef = useFadeIn<HTMLDivElement>({ duration: 0.3 });
  const sessionStatusRef = useSlideInUp<HTMLDivElement>({ duration: 0.3 });

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

  const onSubmitSpotlight = useCallback((data: SpotlightFormData) => {
    const message = data.prompt.trim();
    if (message) {
      // Send to AI for UI generation using the context method
      generateUI(message, {});
      reset(); // Clear form after submission
    }
  }, [generateUI, reset]);

  const formatTimeSinceMemo = useCallback((date: Date): string => {
    const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
    if (seconds < 60) return "just now";
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ago`;
  }, []);

  return (
    <div className="app os-interface">
      {/* Minimal Title Bar - just window controls */}
      <TitleBar sessionManager={sessionManager} />

      {/* Full-screen App Canvas */}
      <div className="os-canvas">
        <DynamicRenderer />
      </div>

      {/* Floating Spotlight-style Input Bar */}
      <div ref={spotlightContainerRef} className={`spotlight-input-container ${inputFocused ? "focused" : ""}`}>
        <form className="spotlight-input-wrapper" onSubmit={handleFormSubmit(onSubmitSpotlight)}>
          <div className="spotlight-icon"><Sparkles size={20} /></div>
          <input
            type="text"
            className="spotlight-input"
            placeholder="Ask AI to create something... (⌘K)"
            onFocus={() => setInputFocused(true)}
            ref={setInputRef}
            {...inputRegisterProps}
          />
          {inputValue && (
            <button
              type="submit"
              className="spotlight-send"
              aria-label="Send message"
            >
              →
            </button>
          )}
        </form>
        {inputFocused && (
          <div ref={hintRef} className="spotlight-hint">Press Enter to generate • Esc to close</div>
        )}
      </div>

      {/* Thought Stream - Slide-out Notification Panel */}
      <ThoughtStream isVisible={showThoughts} onToggle={() => setShowThoughts(!showThoughts)} />

      {/* Session Status Indicator */}
      {sessionManager.isSaving && <div ref={sessionStatusRef} className="session-status saving"><Save size={14} style={{ marginRight: '6px', verticalAlign: 'middle' }} />Saving...</div>}
      {sessionManager.lastSaveTime && !sessionManager.isSaving && (
        <div ref={sessionStatusRef} className="session-status saved">
          <CheckCircle size={14} style={{ marginRight: '6px', verticalAlign: 'middle' }} />Saved {formatTimeSinceMemo(sessionManager.lastSaveTime)}
        </div>
      )}
    </div>
  );
}

export default App;
