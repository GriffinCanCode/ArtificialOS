/**
 * Main Application Component
 */

import React, { useCallback, useEffect } from "react";
import { useForm } from "react-hook-form";
import { Save, Sparkles, CheckCircle } from "lucide-react";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "sonner";
import ThoughtStream from "../ui/components/chat/ThoughtStream";
import TitleBar from "../ui/components/layout/TitleBar";
import { Desktop } from "../ui/components/layout/Desktop";
import { WindowManager } from "../ui/components/layout/WindowManager";
import { Taskbar } from "../ui/components/layout/Taskbar";
import DynamicRenderer from "../features/dynamics/core/DynamicRenderer";
import { WebSocketProvider, useWebSocket } from "../ui/contexts/WebSocketContext";
import { useAppActions } from "../core/store/appStore";
import { useActions } from "../features/windows";
import { useSessionManager } from "../core/hooks/useSessionManager";
import { ServerMessage } from "../core/types/api";
import { useLogger } from "../core/utils/monitoring/useLogger";
import { formatRelativeTime } from "../core/utils/dates";
import { useFadeIn, useSlideInUp } from "../ui/hooks/useGSAP";
import { queryClient } from "../core/lib/queryClient";
import { shouldIgnoreKeyboardEvent } from "../features/input";
import "./App.css";
import "../core/toast/styles.css";
import { initWebVitals } from "../core/monitoring";

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
  const { open: openWindow } = useActions();

  // Initialize performance monitoring
  useEffect(() => {
    initWebVitals();
    log.info("Performance monitoring initialized");
    log.info("Access metrics via window.agentOSMetrics in console");
  }, [log]);

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

  // Window keyboard shortcuts are now handled automatically by the windows module

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
      // Don't trigger shortcuts if user is typing in an input/textarea (except Escape)
      if (shouldIgnoreKeyboardEvent(e, { allowedKeys: ["Escape"] })) {
        return; // Let the input handle the keystroke (but allow Escape to close creator)
      }

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
        // Start generation immediately - this will trigger the builder UI
        generateUI(message, {});
        reset(); // Clear form after submission
        setShowCreator(false); // Hide creator - builder screen will take over
      }
    },
    [generateUI, reset]
  );

  const handleLaunchApp = useCallback(
    async (appId: string) => {
      try {
        const response = await fetch(`http://localhost:8000/registry/apps/${appId}/launch`, {
          method: "POST",
        });
        const data = await response.json();
        if (data.error) {
          log.error("Failed to launch app", undefined, { appId, error: data.error });
        } else if (data.blueprint) {
          // Successfully launched - open in a window
          log.info("App launched successfully", { appId, appInstanceId: data.app_id });

          // Get app metadata for icon
          const metaResponse = await fetch(`http://localhost:8000/registry/apps/${appId}`);
          const metaData = await metaResponse.json();
          const icon = metaData.icon || "📦";

          openWindow(data.app_id, data.blueprint.title, data.blueprint, icon);
        }
      } catch (error) {
        log.error("Failed to launch app", error as Error, { appId });
      }
    },
    [log, openWindow]
  );

  const formatTimeSinceMemo = useCallback((date: Date): string => {
    return formatRelativeTime(date);
  }, []);

  return (
    <div className="app os-interface">
      {/* Toast Container */}
      <Toaster
        position="bottom-right"
        expand={false}
        richColors
        closeButton
        duration={4000}
        theme="dark"
      />

      {/* Minimal Title Bar - just window controls */}
      <TitleBar sessionManager={sessionManager} />

      {/* Welcome Screen with Animation */}
      {showWelcome && (
        <div className={`welcome-screen ${!showWelcome ? "exiting" : ""}`}>
          <div className="welcome-content">
            <div className="welcome-icon">AgentOS</div>
            <h1 className="welcome-title">Welcome to Griffin's AgentOS</h1>
            <p className="welcome-subtitle">
              Press <kbd>⌘K</kbd> or click below to create something
            </p>
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

      {/* Dynamic Renderer (handles AI-generated UI) */}
      <DynamicRenderer />

      {/* Window Manager (PRIMARY app container) */}
      <WindowManager />

      {/* Taskbar (shows open windows) */}
      <Taskbar />

      {/* Creator Overlay (⌘K) */}
      {showCreator && (
        <div className="creator-overlay">
          <div className="creator-backdrop" onClick={() => setShowCreator(false)} />
          <div className="creator-content">
            <div className="welcome-icon">AgentOS</div>
            <h1 className="welcome-title">What would you like to create?</h1>
            <div
              ref={spotlightContainerRef}
              className={`spotlight-input-container creator ${inputFocused ? "focused" : ""}`}
            >
              <form
                className="spotlight-input-wrapper"
                onSubmit={handleFormSubmit(onSubmitSpotlight)}
              >
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
                  <Tooltip content={!client ? "Connecting..." : "Generate app (Enter)"} delay={700}>
                    <button type="submit" className="spotlight-send" aria-label="Generate app">
                      →
                    </button>
                  </Tooltip>
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
