/**
 * Chat Interface Component
 * User input and message history
 */

import React, { useState, useRef, useEffect } from 'react';
import { useMessages, useAppActions } from '../store/appStore';
import { useWebSocket } from '../contexts/WebSocketContext';
import { useLogger } from '../utils/useLogger';
import './ChatInterface.css';

const ChatInterface: React.FC = () => {
  const log = useLogger('ChatInterface');
  const messages = useMessages();
  const { addMessage } = useAppActions();
  const { sendChat, isConnected } = useWebSocket();
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    log.info('Connection status changed', { isConnected });
  }, [isConnected, log]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim() && isConnected) {
      const message = input.trim();
      
      log.info('User sending message', { 
        messageLength: message.length,
        messagePreview: message.substring(0, 50) 
      });
      
      // Add user message to state immediately
      addMessage({
        type: 'user',
        content: message,
        timestamp: Date.now()
      });
      
      // Send via WebSocket
      try {
        sendChat(message, {});
        log.debug('Message sent successfully');
      } catch (error) {
        log.error('Failed to send message', error as Error);
      }
      
      setInput('');
    } else if (!isConnected) {
      log.warn('Attempted to send message while disconnected');
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  return (
    <div className="chat-interface">
      <div className="chat-header">
        <h3>Chat</h3>
        <div className={`connection-status ${isConnected ? 'connected' : 'disconnected'}`}>
          {isConnected ? '● Connected' : '○ Disconnected'}
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

      <form className="chat-input-form" onSubmit={handleSubmit}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={isConnected ? "Type a message..." : "Connecting..."}
          disabled={!isConnected}
          className="chat-input"
        />
        <button
          type="submit"
          disabled={!isConnected || !input.trim()}
          className="send-button"
        >
          Send
        </button>
      </form>
    </div>
  );
};

export default ChatInterface;

