/**
 * ConnectionStatus Component
 * Shows WebSocket connection status to help users understand when the system is ready
 */

import React from 'react';
import { Wifi, WifiOff, RefreshCw } from 'lucide-react';
import { useWebSocket } from '../../../ui/contexts/WebSocketContext';
import './ConnectionStatus.css';

export const ConnectionStatus: React.FC = () => {
  const { isConnected } = useWebSocket();
  const [isReconnecting, setIsReconnecting] = React.useState(false);

  // Show reconnecting state briefly when connection drops
  React.useEffect(() => {
    if (!isConnected) {
      setIsReconnecting(true);
      const timer = setTimeout(() => setIsReconnecting(false), 3000);
      return () => clearTimeout(timer);
    } else {
      setIsReconnecting(false);
    }
  }, [isConnected]);

  if (isConnected) {
    return (
      <div className="connection-status connection-status--connected" title="Connected to backend">
        <Wifi size={16} />
        <span className="connection-status__text">Connected</span>
      </div>
    );
  }

  return (
    <div
      className={`connection-status connection-status--disconnected ${isReconnecting ? 'connection-status--reconnecting' : ''}`}
      title="Disconnected from backend - reconnecting..."
    >
      {isReconnecting ? <RefreshCw size={16} className="spin" /> : <WifiOff size={16} />}
      <span className="connection-status__text">
        {isReconnecting ? 'Reconnecting...' : 'Disconnected'}
      </span>
    </div>
  );
};

