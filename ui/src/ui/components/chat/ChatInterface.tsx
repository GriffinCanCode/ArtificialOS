/**
 * Chat Interface Component
 * User input and message history
 */

import React, { useRef, useEffect, useCallback } from "react";
import { useForm } from "react-hook-form";
import { useMessages, useAppActions } from "../../../core/store/appStore";
import { useWebSocket } from "../../contexts/WebSocketContext";
import { useLogger } from "../../../core/utils/monitoring/useLogger";
import "./ChatInterface.css";

interface ChatFormData {
  message: string;
}

const ChatInterface: React.FC = React.memo(() => {
  const log = useLogger("ChatInterface");
  const messages = useMessages();
  const { addMessage, appendToLastMessage } = useAppActions();
  const { client, sendChat, isConnected } = useWebSocket();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const { register, handleSubmit, reset, watch } = useForm<ChatFormData>({
    defaultValues: { message: "" },
  });

  const inputValue = watch("message");

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  const formatTime = useCallback((timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  useEffect(() => {
    log.info("Connection status changed", { isConnected });
  }, [isConnected, log]);

  // Listen for streaming tokens from WebSocket
  useEffect(() => {
    if (!client) return;

    const unsubscribe = client.onMessage((message) => {
      // Handle streaming tokens for chat responses
      if (message.type === "token") {
        log.verboseThrottled("Received token", {
          contentLength: message.content?.length || 0,
        });
        appendToLastMessage(message.content || "");
      } else if (message.type === "chat_response") {
        // Handle complete chat responses (non-streaming fallback)
        addMessage({
          type: "assistant",
          content: message.content || "",
          timestamp: Date.now(),
        });
      }
    });

    return () => unsubscribe();
  }, [client, appendToLastMessage, addMessage, log]);

  const onSubmit = useCallback(
    (data: ChatFormData) => {
      const message = data.message.trim();
      if (message && isConnected) {
        log.info("User sending message", {
          messageLength: message.length,
          messagePreview: message.substring(0, 50),
        });

        // Add user message to state immediately
        addMessage({
          type: "user",
          content: message,
          timestamp: Date.now(),
        });

        // Send via WebSocket
        try {
          sendChat(message, {});
          log.debug("Message sent successfully");
          reset(); // Clear form after successful send
        } catch (error) {
          log.error("Failed to send message", error as Error);
        }
      } else if (!isConnected) {
        log.warn("Attempted to send message while disconnected");
      }
    },
    [isConnected, log, addMessage, sendChat, reset]
  );

  return (
    <div className="chat-interface">
      <div className="chat-header">
        <h3>Chat</h3>
        <div className={`connection-status ${isConnected ? "connected" : "disconnected"}`}>
          {isConnected ? "● Connected" : "○ Disconnected"}
        </div>
      </div>

      <div className="messages-container">
        {messages.map((msg, idx) => (
          <div key={idx} className={`message message-${msg.type}`}>
            <div className="message-header">
              <span className="message-type">{msg.type}</span>
              <span className="message-time">{formatTime(msg.timestamp)}</span>
            </div>
            <div className="message-content">{msg.content}</div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      <form className="chat-input-form" onSubmit={handleSubmit(onSubmit)}>
        <input
          type="text"
          placeholder={isConnected ? "Type a message..." : "Connecting..."}
          disabled={!isConnected}
          className="chat-input"
          {...register("message", {
            required: true,
            validate: (value) => value.trim().length > 0,
          })}
        />
        <button
          type="submit"
          disabled={!isConnected || !inputValue?.trim()}
          className="send-button"
        >
          Send
        </button>
      </form>
    </div>
  );
});

ChatInterface.displayName = "ChatInterface";

export default ChatInterface;
