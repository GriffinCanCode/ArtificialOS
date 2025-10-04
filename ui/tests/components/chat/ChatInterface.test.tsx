/**
 * ChatInterface Component Tests
 * Tests user input, message rendering, and WebSocket integration
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, userEvent, waitFor, MockWebSocket } from "../../setup/utils";
import ChatInterface from "../../../src/components/chat/ChatInterface";
import { useAppStore } from "../../../src/store/appStore";

// Mock the WebSocket client
vi.mock("../../../src/contexts/WebSocketContext", () => ({
  useWebSocket: () => ({
    client: null,
    isConnected: true,
    sendChat: vi.fn(),
  }),
}));

describe("ChatInterface", () => {
  beforeEach(() => {
    // Reset store state before each test
    useAppStore.setState({
      messages: [],
      thoughts: [],
      uiSpec: null,
      partialUISpec: null,
      isLoading: false,
      isStreaming: false,
      error: null,
      generationThoughts: [],
      generationPreview: "",
      buildProgress: 0,
      appId: null,
    });
  });

  it("renders chat interface with input and connection status", () => {
    render(<ChatInterface />);

    expect(screen.getByText("Chat")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Type a message...")).toBeInTheDocument();
    expect(screen.getByText("● Connected")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /send/i })).toBeInTheDocument();
  });

  it("displays disconnected state when not connected", () => {
    const { useWebSocket } = require("../../../src/contexts/WebSocketContext");
    vi.mocked(useWebSocket).mockReturnValue({
      client: null,
      isConnected: false,
      sendChat: vi.fn(),
    });

    render(<ChatInterface />);

    expect(screen.getByText("○ Disconnected")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Connecting...")).toBeInTheDocument();
  });

  it("disables send button when input is empty", () => {
    render(<ChatInterface />);

    const sendButton = screen.getByRole("button", { name: /send/i });
    expect(sendButton).toBeDisabled();
  });

  it("enables send button when input has text", async () => {
    const user = userEvent.setup();
    render(<ChatInterface />);

    const input = screen.getByPlaceholderText("Type a message...");
    const sendButton = screen.getByRole("button", { name: /send/i });

    await user.type(input, "Hello");

    expect(sendButton).not.toBeDisabled();
  });

  it("sends message and clears input on submit", async () => {
    const user = userEvent.setup();
    const mockSendChat = vi.fn();
    const { useWebSocket } = require("../../../src/contexts/WebSocketContext");
    vi.mocked(useWebSocket).mockReturnValue({
      client: null,
      isConnected: true,
      sendChat: mockSendChat,
    });

    render(<ChatInterface />);

    const input = screen.getByPlaceholderText("Type a message...");
    const sendButton = screen.getByRole("button", { name: /send/i });

    await user.type(input, "Test message");
    await user.click(sendButton);

    expect(mockSendChat).toHaveBeenCalledWith("Test message", {});
    expect(input).toHaveValue("");
  });

  it("displays messages from store", () => {
    useAppStore.setState({
      messages: [
        {
          type: "user",
          content: "User message",
          timestamp: Date.now(),
        },
        {
          type: "assistant",
          content: "Assistant response",
          timestamp: Date.now(),
        },
      ],
    });

    render(<ChatInterface />);

    expect(screen.getByText("User message")).toBeInTheDocument();
    expect(screen.getByText("Assistant response")).toBeInTheDocument();
  });

  it("renders messages with correct styling based on type", () => {
    useAppStore.setState({
      messages: [
        {
          type: "user",
          content: "User message",
          timestamp: Date.now(),
        },
        {
          type: "assistant",
          content: "Assistant message",
          timestamp: Date.now(),
        },
        {
          type: "system",
          content: "System message",
          timestamp: Date.now(),
        },
      ],
    });

    render(<ChatInterface />);

    const userMessage = screen.getByText("User message").closest(".message");
    const assistantMessage = screen.getByText("Assistant message").closest(".message");
    const systemMessage = screen.getByText("System message").closest(".message");

    expect(userMessage).toHaveClass("message-user");
    expect(assistantMessage).toHaveClass("message-assistant");
    expect(systemMessage).toHaveClass("message-system");
  });

  it("prevents sending message when disconnected", async () => {
    const user = userEvent.setup();
    const mockSendChat = vi.fn();
    const { useWebSocket } = require("../../../src/contexts/WebSocketContext");
    vi.mocked(useWebSocket).mockReturnValue({
      client: null,
      isConnected: false,
      sendChat: mockSendChat,
    });

    render(<ChatInterface />);

    const input = screen.getByPlaceholderText("Connecting...");
    const sendButton = screen.getByRole("button", { name: /send/i });

    expect(input).toBeDisabled();
    expect(sendButton).toBeDisabled();
  });

  it("formats timestamps correctly", () => {
    const now = new Date("2024-01-01T12:30:00");
    useAppStore.setState({
      messages: [
        {
          type: "user",
          content: "Test",
          timestamp: now.getTime(),
        },
      ],
    });

    render(<ChatInterface />);

    // Check if time is displayed (format: 12:30 PM)
    expect(screen.getByText(/12:30/)).toBeInTheDocument();
  });

  it("trims whitespace from messages before sending", async () => {
    const user = userEvent.setup();
    const mockSendChat = vi.fn();
    const { useWebSocket } = require("../../../src/contexts/WebSocketContext");
    vi.mocked(useWebSocket).mockReturnValue({
      client: null,
      isConnected: true,
      sendChat: mockSendChat,
    });

    render(<ChatInterface />);

    const input = screen.getByPlaceholderText("Type a message...");
    const sendButton = screen.getByRole("button", { name: /send/i });

    await user.type(input, "   Test message   ");
    await user.click(sendButton);

    expect(mockSendChat).toHaveBeenCalledWith("Test message", {});
  });
});

