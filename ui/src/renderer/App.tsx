/**
 * Main Application Component
 */

import React, { useState, useEffect } from 'react';
import ThoughtStream from '../components/ThoughtStream';
import DynamicRenderer from '../components/DynamicRenderer';
import ChatInterface from '../components/ChatInterface';
import TitleBar from '../components/TitleBar';
import './App.css';

interface Message {
  type: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

interface ThoughtStep {
  content: string;
  timestamp: number;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [thoughts, setThoughts] = useState<ThoughtStep[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [ws, setWs] = useState<WebSocket | null>(null);

  useEffect(() => {
    // Connect to AI service WebSocket
    connectWebSocket();

    return () => {
      ws?.close();
    };
  }, []);

  const connectWebSocket = () => {
    const websocket = new WebSocket('ws://localhost:8000/stream');

    websocket.onopen = () => {
      console.log('✅ Connected to AI service');
      setIsConnected(true);
      setMessages(prev => [...prev, {
        type: 'system',
        content: 'Connected to AI service',
        timestamp: Date.now()
      }]);
    };

    websocket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      switch (data.type) {
        case 'token':
          // Append token to the latest assistant message
          setMessages(prev => {
            const last = prev[prev.length - 1];
            if (last && last.type === 'assistant') {
              return [
                ...prev.slice(0, -1),
                { ...last, content: last.content + data.content }
              ];
            } else {
              return [...prev, {
                type: 'assistant',
                content: data.content,
                timestamp: Date.now()
              }];
            }
          });
          break;

        case 'thought':
          setThoughts(prev => [...prev, {
            content: data.content,
            timestamp: data.timestamp
          }]);
          break;

        case 'complete':
          // Message generation complete
          break;

        case 'system':
          setMessages(prev => [...prev, {
            type: 'system',
            content: data.message,
            timestamp: Date.now()
          }]);
          break;
      }
    };

    websocket.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
      setIsConnected(false);
    };

    websocket.onclose = () => {
      console.log('🔌 Disconnected from AI service');
      setIsConnected(false);
      // Auto-reconnect after 3 seconds
      setTimeout(connectWebSocket, 3000);
    };

    setWs(websocket);
  };

  const sendMessage = (message: string) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      console.error('WebSocket not connected');
      return;
    }

    // Add user message to UI
    setMessages(prev => [...prev, {
      type: 'user',
      content: message,
      timestamp: Date.now()
    }]);

    // Clear previous thoughts
    setThoughts([]);

    // Send to AI service
    ws.send(JSON.stringify({
      type: 'chat',
      message: message,
      context: {}
    }));
  };

  return (
    <div className="app">
      <TitleBar />
      
      <div className="app-layout">
        {/* Left Panel: Chat Interface */}
        <div className="panel chat-panel">
          <ChatInterface
            messages={messages}
            onSendMessage={sendMessage}
            isConnected={isConnected}
          />
        </div>

        {/* Center Panel: Dynamic App Renderer */}
        <div className="panel renderer-panel">
          <DynamicRenderer />
        </div>

        {/* Right Panel: Thought Stream */}
        <div className="panel thoughts-panel">
          <ThoughtStream thoughts={thoughts} />
        </div>
      </div>
    </div>
  );
}

export default App;

