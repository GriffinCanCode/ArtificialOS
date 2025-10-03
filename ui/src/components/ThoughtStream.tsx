/**
 * Thought Stream Component
 * Visualizes AI thinking process in real-time
 */

import React, { useRef, useEffect } from "react";
import { MessageCircle, Brain, X } from "lucide-react";
import { useThoughts, useAppActions } from "../store/appStore";
import { useWebSocket } from "../contexts/WebSocketContext";
import { usePulse, useFadeIn, useStaggerSlideUp } from "../hooks/useGSAP";
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
  
  // GSAP Animation hooks
  const toggleButtonRef = usePulse<HTMLButtonElement>(thoughts.length > 0);
  const backdropRef = useFadeIn<HTMLDivElement>({ duration: 0.3 });
  const thoughtsListRef = useStaggerSlideUp<HTMLDivElement>('.thought-item', { stagger: 0.05, distance: 20 });

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
        ref={toggleButtonRef}
        className={`thought-toggle ${thoughts.length > 0 ? "has-thoughts" : ""}`}
        onClick={onToggle}
        title="Toggle thought stream"
      >
        <span className="thought-icon"><MessageCircle size={20} /></span>
        {thoughts.length > 0 && <span className="thought-badge">{thoughts.length}</span>}
      </button>

      {/* Slide-out Panel */}
      <div className={`thought-stream-panel ${isVisible ? "visible" : ""}`}>
        <div className="thought-stream">
          <div className="thought-header">
            <h3><MessageCircle size={18} style={{ display: 'inline-block', marginRight: '8px', verticalAlign: 'middle' }} />Thought Stream</h3>
            <div className="thought-header-actions">
              <span className="thought-count">{thoughts.length} steps</span>
              <button className="thought-close" onClick={onToggle}>
                <X size={16} />
              </button>
            </div>
          </div>

          <div ref={thoughtsListRef} className="thoughts-container">
            {thoughts.length === 0 ? (
              <div className="empty-state">
                <span className="empty-icon"><Brain size={48} /></span>
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
      {isVisible && <div ref={backdropRef} className="thought-backdrop" onClick={onToggle} />}
    </>
  );
};

export default ThoughtStream;
