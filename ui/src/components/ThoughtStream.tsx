/**
 * Thought Stream Component
 * Visualizes AI thinking process in real-time
 */

import React, { useRef, useEffect } from "react";
import { useThoughts, useAppActions } from "../store/appStore";
import { useWebSocket } from "../contexts/WebSocketContext";
import "./ThoughtStream.css";

const ThoughtStream: React.FC = () => {
  const thoughts = useThoughts();
  const { addThought } = useAppActions();
  const { client } = useWebSocket();
  const streamEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollToBottom();
  }, [thoughts]);

  // Listen for thought messages from WebSocket
  useEffect(() => {
    if (!client) return;

    const unsubscribe = client.onMessage((message) => {
      // Add thoughts to the stream
      if (message.type === "thought" || message.type === "reasoning") {
        addThought({
          content: message.content || "",
          timestamp: Date.now() / 1000, // Convert to seconds
        });
      }
    });

    return () => unsubscribe();
  }, [client, addThought]);

  const scrollToBottom = () => {
    streamEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return (
    <div className="thought-stream">
      <div className="thought-header">
        <h3>💭 Thought Stream</h3>
        <span className="thought-count">{thoughts.length} steps</span>
      </div>

      <div className="thoughts-container">
        {thoughts.length === 0 ? (
          <div className="empty-state">
            <span className="empty-icon">🧠</span>
            <p>AI thoughts will appear here...</p>
          </div>
        ) : (
          thoughts.map((thought, idx) => (
            <div key={idx} className="thought-item">
              <div className="thought-index">{idx + 1}</div>
              <div className="thought-content">
                <div className="thought-text">{thought.content}</div>
                <div className="thought-time">{formatTime(thought.timestamp)}</div>
              </div>
            </div>
          ))
        )}
        <div ref={streamEndRef} />
      </div>
    </div>
  );
};

export default ThoughtStream;
