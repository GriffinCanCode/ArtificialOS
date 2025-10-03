/**
 * WebSocket Context
 * Provides a shared WebSocket connection across the application
 * Eliminates duplicate connections and centralizes state management
 */

import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { WebSocketClient } from '../utils/websocketClient';
import { logger } from '../utils/logger';

interface WebSocketContextType {
  client: WebSocketClient | null;
  isConnected: boolean;
  sendChat: (message: string, context?: Record<string, any>) => void;
  generateUI: (message: string, context?: Record<string, any>) => void;
}

export const WebSocketContext = createContext<WebSocketContextType>({
  client: null,
  isConnected: false,
  sendChat: () => {},
  generateUI: () => {}
});

export const useWebSocket = () => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within WebSocketProvider');
  }
  return context;
};

interface WebSocketProviderProps {
  children: ReactNode;
}

export const WebSocketProvider: React.FC<WebSocketProviderProps> = ({ 
  children
}) => {
  const [client] = useState(() => {
    logger.info('Initializing WebSocket client', { component: 'WebSocketProvider' });
    return new WebSocketClient();
  });
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    logger.info('WebSocketProvider mounted, setting up connection', { component: 'WebSocketProvider' });
    
    // Enhanced connection status handler with logging
    const handleConnectionChange = (connected: boolean) => {
      setIsConnected(connected);
      if (connected) {
        logger.info('WebSocket connected successfully', { component: 'WebSocketProvider' });
      } else {
        logger.warn('WebSocket disconnected', { component: 'WebSocketProvider' });
      }
    };

    // Subscribe to connection status
    const unsubscribeConnection = client.onConnection(handleConnectionChange);

    // Connect only if not already connected (handles React Strict Mode)
    if (!client.isConnected()) {
      try {
        client.connect();
        logger.debug('WebSocket connection initiated', { component: 'WebSocketProvider' });
      } catch (error) {
        logger.error('Failed to initiate WebSocket connection', error as Error, { component: 'WebSocketProvider' });
      }
    } else {
      logger.debug('WebSocket already connected, skipping reconnect', { component: 'WebSocketProvider' });
    }

    // Cleanup - only unsubscribe, don't destroy
    // The client persists for the lifetime of the component
    return () => {
      logger.debug('Unsubscribing from WebSocket connection status', { component: 'WebSocketProvider' });
      unsubscribeConnection();
    };
  }, [client]);

  // Cleanup on unmount - destroy the client only on final unmount
  useEffect(() => {
    return () => {
      logger.info('WebSocketProvider unmounting, destroying client', { component: 'WebSocketProvider' });
      client.destroy();
    };
    // Empty deps array means this only runs on mount/unmount, not re-renders
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const sendChat = (message: string, context?: Record<string, any>) => {
    if (!isConnected) {
      logger.error('Cannot send chat: WebSocket not connected', undefined, { 
        component: 'WebSocketProvider',
        messageLength: message.length 
      });
      return;
    }
    logger.debug('Sending chat message', { 
      component: 'WebSocketProvider',
      messageLength: message.length,
      hasContext: !!context 
    });
    client.sendChat(message, context);
  };

  const generateUI = (message: string, context?: Record<string, any>) => {
    if (!isConnected) {
      logger.error('Cannot generate UI: WebSocket not connected', undefined, { 
        component: 'WebSocketProvider',
        messageLength: message.length 
      });
      return;
    }
    logger.info('Generating UI', { 
      component: 'WebSocketProvider',
      messageLength: message.length,
      hasContext: !!context 
    });
    client.generateUI(message, context);
  };

  return (
    <WebSocketContext.Provider value={{ client, isConnected, sendChat, generateUI }}>
      {children}
    </WebSocketContext.Provider>
  );
};

