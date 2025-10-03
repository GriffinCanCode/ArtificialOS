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
      console.warn("[WebSocketClient] Already connected");
      return;
    }

    this.reconnectAttempts++;
    console.log(
      `[WebSocketClient] Connecting to ${this.options.url} (attempt ${this.reconnectAttempts})...`
    );

    try {
      this.ws = new WebSocket(this.options.url);
      this.setupEventHandlers();
    } catch (error) {
      console.error("[WebSocketClient] Failed to create WebSocket:", error);
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

    console.log("[WebSocketClient] Disconnected");
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
      console.log("[WebSocketClient] ✅ Connected");
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
          console.warn("[WebSocketClient] Invalid message format:", data);
        }
      } catch (error) {
        console.error("[WebSocketClient] Failed to parse message:", error);
        this.notifyError(error instanceof Error ? error : new Error("Failed to parse message"));
      }
    };

    this.ws.onerror = (event: Event) => {
      console.error("[WebSocketClient] ❌ WebSocket error:", event);
      this.notifyError(new Error("WebSocket error"));
    };

    this.ws.onclose = (event: CloseEvent) => {
      console.log("[WebSocketClient] 🔌 Connection closed:", event.code, event.reason);
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

    console.log(`[WebSocketClient] ⏳ Reconnecting in ${delay / 1000}s...`);

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
      console.error("[WebSocketClient] Cannot send message: not connected");
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
        console.error("[WebSocketClient] Error in message handler:", error);
      }
    });
  }

  private notifyConnection(connected: boolean): void {
    this.connectionHandlers.forEach((handler) => {
      try {
        handler(connected);
      } catch (error) {
        console.error("[WebSocketClient] Error in connection handler:", error);
      }
    });
  }

  private notifyError(error: Error): void {
    this.errorHandlers.forEach((handler) => {
      try {
        handler(error);
      } catch (handlerError) {
        console.error("[WebSocketClient] Error in error handler:", handlerError);
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
