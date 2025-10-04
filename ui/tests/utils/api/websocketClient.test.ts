/**
 * WebSocket Client Tests
 * Tests WebSocket connection, messaging, and reconnection logic
 */

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { WebSocketClient } from "../../../src/utils/api/websocketClient";
import { MockWebSocket } from "../../setup/utils";

describe("WebSocketClient", () => {
  let client: WebSocketClient;
  let mockWs: MockWebSocket;

  beforeEach(() => {
    // Mock WebSocket globally
    global.WebSocket = MockWebSocket as any;
    client = new WebSocketClient({
      url: "ws://localhost:8000/ws",
      autoReconnect: false, // Disable for most tests
    });
  });

  afterEach(() => {
    client.destroy();
    MockWebSocket.reset();
  });

  describe("Connection", () => {
    it("connects to WebSocket server", () => {
      client.connect();

      expect(MockWebSocket.instance).toBeDefined();
      expect(MockWebSocket.instance?.url).toBe("ws://localhost:8000/ws");
    });

    it("reports connected state", (done) => {
      client.onConnection((connected) => {
        if (connected) {
          expect(client.isConnected()).toBe(true);
          done();
        }
      });

      client.connect();
    });

    it("disconnects properly", () => {
      client.connect();
      client.disconnect();

      expect(client.isConnected()).toBe(false);
    });

    it("calls connection handlers on connect", (done) => {
      const handler = vi.fn((connected) => {
        if (connected) {
          expect(handler).toHaveBeenCalledWith(true);
          done();
        }
      });

      client.onConnection(handler);
      client.connect();
    });

    it("calls connection handlers on disconnect", (done) => {
      const handler = vi.fn((connected) => {
        if (!connected) {
          expect(handler).toHaveBeenCalledWith(false);
          done();
        }
      });

      client.onConnection(handler);
      client.connect();

      setTimeout(() => {
        client.disconnect();
      }, 10);
    });
  });

  describe("Messaging", () => {
    beforeEach(() => {
      client.connect();
    });

    it("sends chat message", () => {
      client.sendChat("Hello", { userId: "123" });

      expect(MockWebSocket.instance?.sentMessages).toHaveLength(1);
      expect(MockWebSocket.instance?.sentMessages[0]).toMatchObject({
        type: "chat",
        message: "Hello",
        context: { userId: "123" },
      });
    });

    it("sends generate UI message", () => {
      client.generateUI("Create a calculator");

      expect(MockWebSocket.instance?.sentMessages).toHaveLength(1);
      expect(MockWebSocket.instance?.sentMessages[0]).toMatchObject({
        type: "generate_ui",
        message: "Create a calculator",
      });
    });

    it("sends ping message", () => {
      client.ping();

      expect(MockWebSocket.instance?.sentMessages).toHaveLength(1);
      expect(MockWebSocket.instance?.sentMessages[0]).toMatchObject({
        type: "ping",
      });
    });

    it("receives and parses messages", (done) => {
      client.onMessage((message) => {
        expect(message.type).toBe("system");
        expect(message.content).toBe("Connected");
        done();
      });

      MockWebSocket.instance?.simulateMessage({
        type: "system",
        content: "Connected",
      });
    });

    it("handles multiple message handlers", () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      client.onMessage(handler1);
      client.onMessage(handler2);

      MockWebSocket.instance?.simulateMessage({
        type: "system",
        content: "Test",
      });

      expect(handler1).toHaveBeenCalled();
      expect(handler2).toHaveBeenCalled();
    });

    it("unsubscribes message handlers", () => {
      const handler = vi.fn();
      const unsubscribe = client.onMessage(handler);

      MockWebSocket.instance?.simulateMessage({
        type: "system",
        content: "Test 1",
      });

      expect(handler).toHaveBeenCalledTimes(1);

      unsubscribe();

      MockWebSocket.instance?.simulateMessage({
        type: "system",
        content: "Test 2",
      });

      expect(handler).toHaveBeenCalledTimes(1); // Not called again
    });

    it("does not send messages when disconnected", () => {
      client.disconnect();

      expect(() => {
        client.sendChat("Hello");
      }).not.toThrow();

      expect(MockWebSocket.instance?.sentMessages).toHaveLength(0);
    });
  });

  describe("Error Handling", () => {
    it("calls error handlers on WebSocket error", (done) => {
      client.onError((error) => {
        expect(error).toBeInstanceOf(Error);
        done();
      });

      client.connect();
      MockWebSocket.instance?.simulateError();
    });

    it("handles invalid message format gracefully", () => {
      const handler = vi.fn();
      client.onMessage(handler);
      client.connect();

      // Simulate invalid JSON
      const messageEvent = new MessageEvent("message", {
        data: "invalid json",
      });

      MockWebSocket.instance?.onmessage?.(messageEvent);

      expect(handler).not.toHaveBeenCalled();
    });
  });

  describe("Reconnection", () => {
    it("attempts to reconnect on disconnect", (done) => {
      const reconnectClient = new WebSocketClient({
        url: "ws://localhost:8000/ws",
        autoReconnect: true,
        reconnectDelay: 100,
      });

      reconnectClient.connect();

      // Simulate disconnect
      MockWebSocket.instance?.close();

      setTimeout(() => {
        expect(reconnectClient.getReconnectAttempts()).toBeGreaterThan(0);
        reconnectClient.destroy();
        done();
      }, 200);
    });

    it("does not reconnect when manually disconnected", (done) => {
      const reconnectClient = new WebSocketClient({
        url: "ws://localhost:8000/ws",
        autoReconnect: true,
        reconnectDelay: 100,
      });

      reconnectClient.connect();
      reconnectClient.disconnect();

      setTimeout(() => {
        expect(reconnectClient.getReconnectAttempts()).toBe(0);
        reconnectClient.destroy();
        done();
      }, 200);
    });
  });

  describe("Message Types", () => {
    beforeEach(() => {
      client.connect();
    });

    it("handles token messages", (done) => {
      client.onMessage((message) => {
        if (message.type === "token") {
          expect(message.content).toBe("Hello");
          done();
        }
      });

      MockWebSocket.instance?.simulateMessage({
        type: "token",
        content: "Hello",
      });
    });

    it("handles thought messages", (done) => {
      client.onMessage((message) => {
        if (message.type === "thought") {
          expect(message.content).toBe("Analyzing request...");
          done();
        }
      });

      MockWebSocket.instance?.simulateMessage({
        type: "thought",
        content: "Analyzing request...",
      });
    });

    it("handles UI generated messages", (done) => {
      client.onMessage((message) => {
        if (message.type === "ui_generated") {
          expect(message.ui_spec).toBeDefined();
          expect(message.app_id).toBe("app-123");
          done();
        }
      });

      MockWebSocket.instance?.simulateMessage({
        type: "ui_generated",
        app_id: "app-123",
        ui_spec: { type: "app", title: "Test", layout: "vertical", components: [] },
      });
    });

    it("handles error messages", (done) => {
      client.onMessage((message) => {
        if (message.type === "error") {
          expect(message.error).toBe("Something went wrong");
          done();
        }
      });

      MockWebSocket.instance?.simulateMessage({
        type: "error",
        error: "Something went wrong",
      });
    });

    it("handles pong messages", (done) => {
      client.onMessage((message) => {
        if (message.type === "pong") {
          expect(message.timestamp).toBeGreaterThan(0);
          done();
        }
      });

      MockWebSocket.instance?.simulateMessage({
        type: "pong",
        timestamp: Date.now(),
      });
    });
  });

  describe("Cleanup", () => {
    it("destroys client and cleans up resources", () => {
      client.connect();
      const handler = vi.fn();
      client.onMessage(handler);

      client.destroy();

      expect(client.isConnected()).toBe(false);
      
      // Try to send message after destroy - should not throw
      expect(() => {
        client.sendChat("Test");
      }).not.toThrow();
    });
  });
});

