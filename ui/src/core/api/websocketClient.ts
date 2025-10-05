/**
 * Type-Safe WebSocket Client
 * Provides validated WebSocket communication with automatic reconnection
 */

import {
  ClientMessage,
  ServerMessage,
  parseServerMessage,
  createChatMessage,
  createGenerateUIMessage,
  createPingMessage,
} from "../types/api";
import { logger } from "../utils/monitoring/logger";

export type MessageHandler = (message: ServerMessage) => void;
export type ConnectionHandler = (connected: boolean) => void;
export type ErrorHandler = (error: Error) => void;

// ============================================================================
// WebSocket Client Options
// ============================================================================

export interface WebSocketClientOptions {
  url?: string;
  reconnectDelay?: number;
  maxReconnectDelay?: number;
  reconnectBackoff?: number;
  autoReconnect?: boolean;
}

const DEFAULT_OPTIONS: Required<WebSocketClientOptions> = {
  url: "ws://localhost:8000/stream",
  reconnectDelay: 1000,
  maxReconnectDelay: 10000,
  reconnectBackoff: 2,
  autoReconnect: true,
};

// ============================================================================
// WebSocket Client Class
// ============================================================================

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private options: Required<WebSocketClientOptions>;
  private reconnectAttempts = 0;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private shouldReconnect = true;
  private isManualClose = false;

  // Event handlers
  private messageHandlers: Set<MessageHandler> = new Set();
  private connectionHandlers: Set<ConnectionHandler> = new Set();
  private errorHandlers: Set<ErrorHandler> = new Set();

  constructor(options: WebSocketClientOptions = {}) {
    this.options = { ...DEFAULT_OPTIONS, ...options };
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  /**
   * Connect to the WebSocket server
   */
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      logger.warn("WebSocket already connected", { component: "WebSocketClient" });
      return;
    }

    this.reconnectAttempts++;
    logger.info("Connecting to WebSocket", {
      component: "WebSocketClient",
      url: this.options.url,
      attempt: this.reconnectAttempts,
    });

    try {
      this.ws = new WebSocket(this.options.url);
      this.setupEventHandlers();
    } catch (error) {
      logger.error("Failed to create WebSocket", error as Error, { component: "WebSocketClient" });
      this.handleReconnect();
    }
  }

  /**
   * Close the WebSocket connection
   */
  disconnect(): void {
    this.isManualClose = true;
    this.shouldReconnect = false;

    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    logger.info("WebSocket disconnected", { component: "WebSocketClient" });
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Get current reconnect attempts
   */
  getReconnectAttempts(): number {
    return this.reconnectAttempts;
  }

  // ============================================================================
  // Event Handlers Setup
  // ============================================================================

  private setupEventHandlers(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      logger.info("WebSocket connected", { component: "WebSocketClient" });
      this.reconnectAttempts = 0;
      this.isManualClose = false;
      this.notifyConnection(true);
    };

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data);
        const message = parseServerMessage(data);

        if (message) {
          this.notifyMessage(message);
        } else {
          logger.warn("Invalid WebSocket message format", {
            component: "WebSocketClient",
            data: JSON.stringify(data).substring(0, 100),
          });
        }
      } catch (error) {
        logger.error("Failed to parse WebSocket message", error as Error, {
          component: "WebSocketClient",
        });
        this.notifyError(error instanceof Error ? error : new Error("Failed to parse message"));
      }
    };

    this.ws.onerror = (event: Event) => {
      logger.error("WebSocket error", undefined, {
        component: "WebSocketClient",
        event: String(event),
      });
      this.notifyError(new Error("WebSocket error"));
    };

    this.ws.onclose = (event: CloseEvent) => {
      logger.info("WebSocket connection closed", {
        component: "WebSocketClient",
        code: event.code,
        reason: event.reason,
      });
      this.notifyConnection(false);
      this.ws = null;

      if (!this.isManualClose && this.shouldReconnect && this.options.autoReconnect) {
        this.handleReconnect();
      }
    };
  }

  // ============================================================================
  // Reconnection Logic
  // ============================================================================

  private handleReconnect(): void {
    if (!this.shouldReconnect || !this.options.autoReconnect) {
      return;
    }

    const delay = Math.min(
      this.options.reconnectDelay *
        Math.pow(this.options.reconnectBackoff, this.reconnectAttempts - 1),
      this.options.maxReconnectDelay
    );

    logger.info("WebSocket reconnecting", {
      component: "WebSocketClient",
      delaySeconds: delay / 1000,
      attempt: this.reconnectAttempts,
    });

    this.reconnectTimeout = setTimeout(() => {
      this.connect();
    }, delay);
  }

  // ============================================================================
  // Message Sending
  // ============================================================================

  /**
   * Send a raw message (for advanced use)
   */
  send(message: ClientMessage): void {
    if (!this.isConnected()) {
      logger.error("Cannot send WebSocket message: not connected", undefined, {
        component: "WebSocketClient",
      });
      throw new Error("WebSocket not connected");
    }

    this.ws!.send(JSON.stringify(message));
  }

  /**
   * Send a chat message
   */
  sendChat(message: string, context?: Record<string, any>): void {
    const chatMessage = createChatMessage(message, context);
    this.send(chatMessage);
  }

  /**
   * Request UI generation
   */
  generateUI(message: string, context?: Record<string, any>): void {
    const uiMessage = createGenerateUIMessage(message, context);
    this.send(uiMessage);
  }

  /**
   * Send ping
   */
  ping(): void {
    const pingMessage = createPingMessage();
    this.send(pingMessage);
  }

  // ============================================================================
  // Event Subscription
  // ============================================================================

  /**
   * Subscribe to incoming messages
   */
  onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  /**
   * Subscribe to connection status changes
   */
  onConnection(handler: ConnectionHandler): () => void {
    this.connectionHandlers.add(handler);
    // Immediately notify of current state
    handler(this.isConnected());
    return () => this.connectionHandlers.delete(handler);
  }

  /**
   * Subscribe to errors
   */
  onError(handler: ErrorHandler): () => void {
    this.errorHandlers.add(handler);
    return () => this.errorHandlers.delete(handler);
  }

  // ============================================================================
  // Event Notification (Internal)
  // ============================================================================

  private notifyMessage(message: ServerMessage): void {
    this.messageHandlers.forEach((handler) => {
      try {
        handler(message);
      } catch (error) {
        logger.error("Error in WebSocket message handler", error as Error, {
          component: "WebSocketClient",
        });
      }
    });
  }

  private notifyConnection(connected: boolean): void {
    this.connectionHandlers.forEach((handler) => {
      try {
        handler(connected);
      } catch (error) {
        logger.error("Error in WebSocket connection handler", error as Error, {
          component: "WebSocketClient",
        });
      }
    });
  }

  private notifyError(error: Error): void {
    this.errorHandlers.forEach((handler) => {
      try {
        handler(error);
      } catch (handlerError) {
        logger.error("Error in WebSocket error handler", handlerError as Error, {
          component: "WebSocketClient",
        });
      }
    });
  }

  // ============================================================================
  // Cleanup
  // ============================================================================

  /**
   * Clean up all handlers and disconnect
   */
  destroy(): void {
    this.disconnect();
    this.messageHandlers.clear();
    this.connectionHandlers.clear();
    this.errorHandlers.clear();
  }
}

// ============================================================================
// Export convenience function
// ============================================================================

/**
 * Create a new WebSocket client instance
 */
export function createWebSocketClient(options?: WebSocketClientOptions): WebSocketClient {
  return new WebSocketClient(options);
}

export default WebSocketClient;
