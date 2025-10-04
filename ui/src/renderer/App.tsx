/**
 * Main Application Component
 */

import React, { useCallback, useEffect } from "react";
import { useForm } from "react-hook-form";
import { Save, Sparkles, CheckCircle } from "lucide-react";
import { QueryClientProvider } from "@tanstack/react-query";
import ThoughtStream from "../components/chat/ThoughtStream";
import DynamicRenderer from "../components/dynamics/DynamicRenderer";
import TitleBar from "../components/layout/TitleBar";
import { Desktop } from "../components/layout/Desktop";
import { WindowManager } from "../components/layout/WindowManager";
import { Taskbar } from "../components/layout/Taskbar";
import { WebSocketProvider, useWebSocket } from "../contexts/WebSocketContext";
import { useAppActions } from "../store/appStore";
import { useWindowActions } from "../store/windowStore";
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
  const { openWindow } = useWindowActions();

  // Initialize session manager with auto-save every 30s
  // Memoize options to prevent hooks order issues during HMR
  const sessionManagerOptions = React.useMemo(
    () => ({
      autoSaveInterval: 30,
      enableAutoSave: true,
      restoreOnMount: true,
    }),
    []
  );

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
  const [showWelcome, setShowWelcome] = React.useState(true);
  const [showCreator, setShowCreator] = React.useState(false);

  // React Hook Form for spotlight input
  const {
    register,
    handleSubmit: handleFormSubmit,
    reset,
    watch,
  } = useForm<SpotlightFormData>({
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

  // Welcome screen animation - hide after 2 seconds
  useEffect(() => {
    const timer = setTimeout(() => {
      setShowWelcome(false);
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  // Global keyboard shortcut: Cmd/Ctrl + K to toggle creator
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setShowCreator(!showCreator);
        if (!showCreator) {
          setInputFocused(true);
          setTimeout(() => inputRef.current?.focus(), 50);
        }
      }
      // Escape to close creator
      if (e.key === "Escape" && showCreator) {
        setShowCreator(false);
        setInputFocused(false);
        inputRef.current?.blur();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showCreator, inputFocused]);

  const onSubmitSpotlight = useCallback(
    (data: SpotlightFormData) => {
      const message = data.prompt.trim();
      if (message) {
        // Send to AI for UI generation using the context method
        generateUI(message, {});
        reset(); // Clear form after submission
        setShowCreator(false); // Hide creator after submission
      }
    },
    [generateUI, reset]
  );

  const handleLaunchApp = useCallback(async (appId: string) => {
    try {
      const response = await fetch(`http://localhost:8000/registry/apps/${appId}/launch`, {
        method: "POST",
      });
      const data = await response.json();
      if (data.error) {
        log.error("Failed to launch app", undefined, { appId, error: data.error });
      } else if (data.ui_spec) {
        // Successfully launched - open in a window
        log.info("App launched successfully", { appId, appInstanceId: data.app_id });
        
        // Get app metadata for icon
        const metaResponse = await fetch(`http://localhost:8000/registry/apps/${appId}`);
        const metaData = await metaResponse.json();
        const icon = metaData.icon || "📦";
        
        openWindow(data.app_id, data.ui_spec.title, data.ui_spec, icon);
      }
    } catch (error) {
      log.error("Failed to launch app", error as Error, { appId });
    }
  }, [log, openWindow]);

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

      {/* Welcome Screen with Animation */}
      {showWelcome && (
        <div className={`welcome-screen ${!showWelcome ? "exiting" : ""}`}>
          <div className="welcome-content">
            <div className="welcome-icon">✨</div>
            <h1 className="welcome-title">Welcome to Griffin's AgentOS</h1>
            <p className="welcome-subtitle">Press <kbd>⌘K</kbd> or click below to create something</p>
          </div>
        </div>
      )}

      {/* Desktop (always rendered, revealed when welcome fades) */}
      <div className={`desktop-container ${showWelcome ? "hidden" : "visible"}`}>
        <Desktop
          onLaunchApp={handleLaunchApp}
          onOpenHub={() => handleLaunchApp("hub")}
          onOpenCreator={() => setShowCreator(true)}
        />
      </div>

      {/* Full-screen App Canvas (for AI-generated apps) */}
      <div className="os-canvas">
        <DynamicRenderer />
      </div>

      {/* Window Manager (for windowed apps) */}
      <WindowManager />

      {/* Taskbar (shows open windows) */}
      <Taskbar />

      {/* Creator Overlay (⌘K) */}
      {showCreator && (
        <div className="creator-overlay">
          <div className="creator-backdrop" onClick={() => setShowCreator(false)} />
          <div className="creator-content">
            <div className="welcome-icon">✨</div>
            <h1 className="welcome-title">What would you like to create?</h1>
            <div
              ref={spotlightContainerRef}
              className={`spotlight-input-container creator ${inputFocused ? "focused" : ""}`}
            >
              <form className="spotlight-input-wrapper" onSubmit={handleFormSubmit(onSubmitSpotlight)}>
                <div className="spotlight-icon">
                  <Sparkles size={20} />
                </div>
                <input
                  type="text"
                  className="spotlight-input"
                  placeholder="Ask AI to create something..."
                  onFocus={() => setInputFocused(true)}
                  ref={setInputRef}
                  {...inputRegisterProps}
                  autoFocus
                />
                {inputValue && (
                  <button type="submit" className="spotlight-send" aria-label="Send message">
                    →
                  </button>
                )}
              </form>
              {inputFocused && (
                <div ref={hintRef} className="spotlight-hint">
                  Press Enter to generate • Esc to close
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Thought Stream - Slide-out Notification Panel */}
      <ThoughtStream isVisible={showThoughts} onToggle={() => setShowThoughts(!showThoughts)} />

      {/* Session Status Indicator */}
      {sessionManager.isSaving && (
        <div ref={sessionStatusRef} className="session-status saving">
          <Save size={14} style={{ marginRight: "6px", verticalAlign: "middle" }} />
          Saving...
        </div>
      )}
      {sessionManager.lastSaveTime && !sessionManager.isSaving && (
        <div ref={sessionStatusRef} className="session-status saved">
          <CheckCircle size={14} style={{ marginRight: "6px", verticalAlign: "middle" }} />
          Saved {formatTimeSinceMemo(sessionManager.lastSaveTime)}
        </div>
      )}
    </div>
  );
}

export default App;
