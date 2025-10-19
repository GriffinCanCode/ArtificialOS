/**
 * Main Application Component
 */

import React, { useCallback, useEffect } from "react";
import { useForm } from "react-hook-form";
import { Sparkles } from "lucide-react";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "sonner";
import ThoughtStream from "../ui/components/chat/ThoughtStream";
import TitleBar from "../ui/components/layout/TitleBar";
import { Desktop } from "../ui/components/layout/Desktop";
import { WindowManager } from "../ui/components/layout/WindowManager";
import { Taskbar } from "../ui/components/layout/Taskbar";
import DynamicRenderer from "../features/dynamics/core/DynamicRenderer";
import { Spotlight } from "../features/search/components/Spotlight";
import { WebSocketProvider, useWebSocket } from "../ui/contexts/WebSocketContext";
import { useAppActions } from "../core/store/appStore";
import { useActions, useStore as useWindowStore } from "../features/windows";
import { useSessionManager } from "../core/hooks/useSessionManager";
import { ServerMessage } from "../core/types/api";
import { useLogger } from "../core/monitoring/hooks/useLogger";
import { useJourney } from "../core/monitoring";
import { useFadeIn } from "../ui/hooks/useGSAP";
import { queryClient } from "../core/lib/queryClient";
import { useScope, useShortcuts } from "../features/input";
import { Tooltip } from "../features/floating";
import { AnimatedTitle } from "../ui/components/typography";
import { TypewriterText } from "../ui/components/typography/TypewriterText";
import "./App.css";
import "../core/toast/styles.css";
import { initWebVitals, MonitorProvider } from "../core/monitoring";

// Expose window store globally for native apps
if (typeof window !== "undefined") {
  (window as any).useWindowStore = useWindowStore;
}

interface SpotlightFormData {
  prompt: string;
}

function App() {
  return (
    <MonitorProvider
      autoStart={true}
      desktopContext={{
        environment: process.env.NODE_ENV as 'development' | 'production',
      }}
    >
      <QueryClientProvider client={queryClient}>
        <WebSocketProvider>
          <AppContent />
        </WebSocketProvider>
      </QueryClientProvider>
    </MonitorProvider>
  );
}

function AppContent() {
  const log = useLogger("AppContent");
  const journey = useJourney("AppContent", true, "User opened AgentOS");
  const [showAbout, setShowAbout] = React.useState(false);
  const [showSpotlight, setShowSpotlight] = React.useState(false);
  const [showLaunchpad, setShowLaunchpad] = React.useState(false);
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
    [addMessage, addThought, appendToLastMessage, log]
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

  // Welcome screen animation - hide after 2 seconds
  useEffect(() => {
    const timer = setTimeout(() => {
      setShowWelcome(false);
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  // Global keyboard shortcuts
  useScope("global");

  useShortcuts([
    {
      id: "app.creator.toggle",
      sequence: "$mod+k",
      label: "Toggle Creator",
      description: "Open or close the app creator",
      category: "system",
      scope: "global",
      priority: "critical",
      handler: () => {
        setShowCreator(!showCreator);
        if (!showCreator) {
          setInputFocused(true);
          setTimeout(() => inputRef.current?.focus(), 50);
        }
      },
    },
    {
      id: "app.creator.close",
      sequence: "Escape",
      label: "Close Creator",
      description: "Close the creator modal",
      category: "system",
      scope: "creator",
      priority: "high",
      allowInInput: true,
      enabled: showCreator,
      handler: () => {
        if (showCreator) {
          setShowCreator(false);
          setInputFocused(false);
          inputRef.current?.blur();
        }
      },
    },
    {
      id: "app.spotlight.toggle",
      sequence: "$mod+shift+k",
      label: "Toggle Spotlight",
      description: "Open or close Spotlight search",
      category: "system",
      scope: "global",
      priority: "critical",
      handler: () => {
        setShowSpotlight(!showSpotlight);
      },
    },
    {
      id: "app.spotlight.close",
      sequence: "Escape",
      label: "Close Spotlight",
      description: "Close Spotlight search",
      category: "system",
      scope: "global",
      priority: "high",
      allowInInput: true,
      enabled: showSpotlight,
      handler: () => {
        setShowSpotlight(false);
      },
    },
    {
      id: "app.launchpad.close",
      sequence: "Escape",
      label: "Close Launchpad",
      description: "Close Launchpad",
      category: "system",
      scope: "global",
      priority: "high",
      allowInInput: false,
      enabled: showLaunchpad,
      handler: () => {
        setShowLaunchpad(false);
      },
    },
  ]);

  const onSubmitSpotlight = useCallback(
    (data: SpotlightFormData) => {
      const message = data.prompt.trim();
      if (message) {
        // Track form submission
        journey.trackInteraction('spotlight-form', 'submit');
        journey.addStep('user_action', `User submitted prompt: "${message.substring(0, 50)}${message.length > 50 ? '...' : ''}"`, {
          promptLength: message.length,
          action: 'submit_spotlight_form',
        });

        // Start generation immediately - this will trigger the builder UI
        generateUI(message, {});
        reset(); // Clear form after submission
        setShowCreator(false); // Hide creator - builder screen will take over
      }
    },
    [generateUI, reset, journey]
  );

  const handleLaunchApp = useCallback(
    async (appId: string) => {
      // Track app launch
      journey.trackInteraction('app-launcher', `launch-${appId}`);
      journey.addStep('user_action', `User launching app: ${appId}`, {
        appId,
        action: 'launch_app',
      });

      try {
        const response = await fetch(`http://localhost:8000/registry/apps/${appId}/launch`, {
          method: "POST",
        });
        const data = await response.json();
        if (data.error) {
          log.error("Failed to launch app", undefined, { appId, error: data.error });
          journey.trackError(new Error(`Failed to launch app: ${data.error}`), { appId });
        } else if (data.type === "native_web") {
          // Native web app launched
          log.info("Native web app launched successfully", {
            appId,
            appInstanceId: data.app_id,
            packageId: data.package_id,
            bundlePath: data.bundle_path,
          });

          journey.trackResponse('launch_app', performance.now(), true);
          journey.addStep('navigation', `Opening window for native app: ${data.title}`, {
            appId: data.app_id,
            appType: 'native_web',
            packageId: data.package_id,
          });

          // Open window with native app metadata
          // Pass empty blueprint and metadata as 5th parameter
          openWindow(
            data.app_id,
            data.title,
            {
              type: "native",
              title: data.title,
              layout: "vertical",
              components: [],
            },
            data.icon || "📦",
            {
              appType: "native_web",
              packageId: data.package_id,
              bundlePath: data.bundle_path,
              services: data.services,
              permissions: data.permissions,
            }
          );
        } else if (data.blueprint) {
          // Blueprint app launched - open in a window
          log.info("App launched successfully", { appId, appInstanceId: data.app_id });

          journey.trackResponse('launch_app', performance.now(), true);
          journey.addStep('navigation', `Opening window for blueprint app: ${data.blueprint.title}`, {
            appId: data.app_id,
            appType: 'blueprint',
          });

          // Get app metadata for icon
          const metaResponse = await fetch(`http://localhost:8000/registry/apps/${appId}`);
          const metaData = await metaResponse.json();
          const icon = metaData.icon || "📦";

          openWindow(data.app_id, data.blueprint.title, data.blueprint, icon);
        }
      } catch (error) {
        log.error("Failed to launch app", error as Error, { appId });
        journey.trackError(error as Error, { appId, action: 'launch_app' });
      }
    },
    [log, openWindow, journey]
  );

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
      <TitleBar
        sessionManager={sessionManager}
        showAbout={showAbout}
        onOpenAbout={() => setShowAbout(true)}
        onCloseAbout={() => setShowAbout(false)}
        onOpenSpotlight={() => setShowSpotlight(true)}
      />

      {/* Welcome Screen with Animation */}
      {showWelcome && (
        <div className={`welcome-screen ${!showWelcome ? "exiting" : ""}`}>
          <div className="welcome-content">
            <div className="welcome-icon">✨</div>
            <h1 className="welcome-title">
              <AnimatedTitle text="AgentOS" preset="hero" effect="gradient" animationDelay={200} />
            </h1>
            <p className="welcome-subtitle">
              <TypewriterText text="Press ⌘K to create something amazing" speed={40} delay={800} />
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
          onOpenAbout={() => setShowAbout(true)}
          showLaunchpad={showLaunchpad}
          onToggleLaunchpad={() => setShowLaunchpad(!showLaunchpad)}
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
            <div className="welcome-icon">✨</div>
            <h1 className="welcome-title">
              <AnimatedTitle
                text="What would you like to create?"
                preset="title"
                effect="glow"
                animationDelay={0}
              />
            </h1>
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

      {/* Spotlight Search */}
      <Spotlight
        isOpen={showSpotlight}
        onClose={() => setShowSpotlight(false)}
        onSelect={(item) => {
          log.info("Spotlight item selected", { item });
          setShowSpotlight(false);
        }}
      />
    </div>
  );
}

export default App;
