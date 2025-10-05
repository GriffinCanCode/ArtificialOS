/**
 * WebSocket Context
 * Provides a shared WebSocket connection across the application
 * Eliminates duplicate connections and centralizes state management
 */

import React, { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { WebSocketClient } from "../../core/api/websocketClient";
import { logger } from "../../core/utils/monitoring/logger";
import { useAppStore } from "../../core/store/appStore";
import { useStore as useWindowStore, useActions } from "../../features/windows";

interface WebSocketContextType {
  client: WebSocketClient | null;
  isConnected: boolean;
  sendChat: (message: string, context?: Record<string, any>) => void;
  generateUI: (message: string, context?: Record<string, any>) => void;
}

// Internal context - not exported to maintain Fast Refresh compatibility
const WebSocketContext = createContext<WebSocketContextType>({
  client: null,
  isConnected: false,
  sendChat: () => {},
  generateUI: () => {},
});

export const useWebSocket = () => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error("useWebSocket must be used within WebSocketProvider");
  }
  return context;
};

interface WebSocketProviderProps {
  children: ReactNode;
}

export const WebSocketProvider: React.FC<WebSocketProviderProps> = ({ children }) => {
  const [client] = useState(() => {
    logger.info("Initializing WebSocket client", { component: "WebSocketProvider" });
    return new WebSocketClient();
  });
  const [isConnected, setIsConnected] = useState(false);
  const { open: openWindow, close: closeWindow } = useActions();

  useEffect(() => {
    logger.info("WebSocketProvider mounted, setting up connection", {
      component: "WebSocketProvider",
    });

    // Enhanced connection status handler with logging
    const handleConnectionChange = (connected: boolean) => {
      setIsConnected(connected);
      if (connected) {
        logger.info("WebSocket connected successfully", { component: "WebSocketProvider" });
      } else {
        logger.warn("WebSocket disconnected", { component: "WebSocketProvider" });
      }
    };

    // Subscribe to connection status
    const unsubscribeConnection = client.onConnection(handleConnectionChange);

    // Connect only if not already connected (handles React Strict Mode)
    if (!client.isConnected()) {
      try {
        client.connect();
        logger.debug("WebSocket connection initiated", { component: "WebSocketProvider" });
      } catch (error) {
        logger.error("Failed to initiate WebSocket connection", error as Error, {
          component: "WebSocketProvider",
        });
      }
    } else {
      logger.debug("WebSocket already connected, skipping reconnect", {
        component: "WebSocketProvider",
      });
    }

    // Cleanup - unsubscribe and destroy on unmount
    return () => {
      logger.debug("WebSocketProvider effect cleanup", { component: "WebSocketProvider" });
      unsubscribeConnection();
      // Only destroy if we're actually unmounting (not React Strict Mode)
      // In Strict Mode, the client will persist via useState
      // On real unmount, the client will be garbage collected
    };
  }, [client]);

  const sendChat = React.useCallback((message: string, context?: Record<string, any>) => {
    // Check the actual client connection state, not React state
    if (!client.isConnected()) {
      logger.error("Cannot send chat: WebSocket not connected", undefined, {
        component: "WebSocketProvider",
        messageLength: message.length,
        reactStateConnected: isConnected,
        clientStateConnected: client.isConnected(),
      });
      return;
    }
    logger.debug("Sending chat message", {
      component: "WebSocketProvider",
      messageLength: message.length,
      hasContext: !!context,
    });
    client.sendChat(message, context);
  }, [client, isConnected]);

  const generateUI = React.useCallback((message: string, context?: Record<string, any>) => {
    // Prevent concurrent generations
    if ((window as any).__isGenerating) {
      logger.warn("Generation already in progress, ignoring duplicate call", {
        component: "WebSocketProvider",
      });
      return;
    }

    // Check the actual client connection state, not React state
    if (!client.isConnected()) {
      logger.error("Cannot generate UI: WebSocket not connected", undefined, {
        component: "WebSocketProvider",
        messageLength: message.length,
        reactStateConnected: isConnected,
        clientStateConnected: client.isConnected(),
      });
      return;
    }

    // Set generating flag
    (window as any).__isGenerating = true;

    logger.info("Generating UI", {
      component: "WebSocketProvider",
      messageLength: message.length,
      hasContext: !!context,
    });

    // CRITICAL: Open a builder window immediately
    // This shows the generation progress in a window instead of fullscreen
    const appStore = useAppStore.getState();
    const windowStore = useWindowStore.getState();

    // Reset generation state but DON'T set streaming (no fullscreen UI)
    // appStore.setStreaming(true); // REMOVED - we're using windows now!
    appStore.setBuildProgress(0);
    appStore.setError(null);
    appStore.setPartialBlueprint({ components: [] });

    // Close any existing builder window first to prevent duplicates
    const existingBuilderId = (window as any).__builderWindowId;
    if (existingBuilderId) {
      const existingBuilder = windowStore.windows.find(w => w.appId === existingBuilderId);
      if (existingBuilder) {
        logger.debug("Closing existing builder window before creating new one", {
          component: "WebSocketContext",
          existingBuilderId,
        });
        closeWindow(existingBuilder.id);
      }
    }

    // Create builder window - DynamicRenderer will handle showing build UI
    const builderId = `builder-${Date.now()}`;
    openWindow(
      builderId,
      "🔨 Building App...",
      {
        type: "app",
        title: "Building...",
        layout: "vertical",
        components: [],
        style: {},
        services: [],
        service_bindings: {},
        lifecycle_hooks: {},
      },
      "🔨"
    );

    // Store builder window ID for updates during streaming
    (window as any).__builderWindowId = builderId;

    client.generateUI(message, context);
  }, [client, isConnected, openWindow, closeWindow]);

  return (
    <WebSocketContext.Provider value={{ client, isConnected, sendChat, generateUI }}>
      {children}
    </WebSocketContext.Provider>
  );
};
