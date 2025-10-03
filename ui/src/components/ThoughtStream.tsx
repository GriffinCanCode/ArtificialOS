/**
 * Thought Stream Component
 * Visualizes AI thinking process in real-time
 */

import React, { useRef, useEffect } from "react";
import { useThoughts, useAppActions } from "../store/appStore";
import { useWebSocket } from "../contexts/WebSocketContext";
import "./ThoughtStream.css";

interface ThoughtStreamProps {
  isVisible: boolean;
  onToggle: () => void;
}

const ThoughtStream: React.FC<ThoughtStreamProps> = ({ isVisible, onToggle }) => {
  const thoughts = useThoughts();
  const { addThought } = useAppActions();
  const { client } = useWebSocket();
  const streamEndRef = useRef<HTMLDivElement>(null);
  const prevThoughtsLength = useRef(0);

  useEffect(() => {
    scrollToBottom();
  }, [thoughts]);

  // Auto-open panel when new thoughts arrive (optional - can be disabled)
  useEffect(() => {
    // Only auto-open on first thought
    if (thoughts.length === 1 && prevThoughtsLength.current === 0 && !isVisible) {
      onToggle();
    }
    prevThoughtsLength.current = thoughts.length;
  }, [thoughts.length, isVisible, onToggle]);

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
    <>
      {/* Toggle Button - Fixed in top-right corner */}
      <button
        className={`thought-toggle ${thoughts.length > 0 ? "has-thoughts" : ""}`}
        onClick={onToggle}
        title="Toggle thought stream"
      >
        <span className="thought-icon">💭</span>
        {thoughts.length > 0 && <span className="thought-badge">{thoughts.length}</span>}
      </button>

      {/* Slide-out Panel */}
      <div className={`thought-stream-panel ${isVisible ? "visible" : ""}`}>
        <div className="thought-stream">
          <div className="thought-header">
            <h3>💭 Thought Stream</h3>
            <div className="thought-header-actions">
              <span className="thought-count">{thoughts.length} steps</span>
              <button className="thought-close" onClick={onToggle}>
                ✕
              </button>
            </div>
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
      </div>

      {/* Backdrop */}
      {isVisible && <div className="thought-backdrop" onClick={onToggle} />}
    </>
  );
};

export default ThoughtStream;
