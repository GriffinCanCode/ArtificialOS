/**
 * Network Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface NetworkPageProps {
  executor: any;
  state: any;
}

export function NetworkPage({ executor }: NetworkPageProps) {
  const [proxyEnabled, setProxyEnabled] = useState(false);
  const [proxyUrl, setProxyUrl] = useState('');
  const [networkStats, setNetworkStats] = useState<any>(null);

  useEffect(() => {
    loadSettings();
    loadNetworkStats();
  }, []);

  const loadSettings = async () => {
    try {
      const proxyResult = await executor.execute('settings.get', { key: 'network.proxy_enabled' });
      if (proxyResult?.value !== undefined) setProxyEnabled(proxyResult.value);

      const urlResult = await executor.execute('settings.get', { key: 'network.proxy_url' });
      if (urlResult?.value) setProxyUrl(urlResult.value);
    } catch (error) {
      console.error('Failed to load network settings:', error);
    }
  };

  const loadNetworkStats = async () => {
    try {
      const stats = await executor.execute('monitor.network', {});
      setNetworkStats(stats);
    } catch (error) {
      console.error('Failed to load network stats:', error);
    }
  };

  const handleProxyToggle = async () => {
    const newValue = !proxyEnabled;
    setProxyEnabled(newValue);
    try {
      await executor.execute('settings.set', { key: 'network.proxy_enabled', value: newValue });
    } catch (error) {
      console.error('Failed to update proxy setting:', error);
    }
  };

  const handleProxyUrlChange = async () => {
    try {
      await executor.execute('settings.set', { key: 'network.proxy_url', value: proxyUrl });
    } catch (error) {
      console.error('Failed to update proxy URL:', error);
    }
  };

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>Network Settings</h1>
        <p className="page-description">Configure network and connectivity options</p>
      </div>

      {networkStats && (
        <div className="settings-section">
          <h2 className="section-title">Network Statistics</h2>
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-label">Bytes Sent</div>
              <div className="stat-value">{formatBytes(networkStats.bytes_sent)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Bytes Received</div>
              <div className="stat-value">{formatBytes(networkStats.bytes_received)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Packets Sent</div>
              <div className="stat-value">{networkStats.packets_sent || 0}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Packets Received</div>
              <div className="stat-value">{networkStats.packets_received || 0}</div>
            </div>
          </div>
        </div>
      )}

      <div className="settings-section">
        <h2 className="section-title">Proxy Configuration</h2>
        <SettingRow
          label="Enable Proxy"
          description="Route network traffic through a proxy server"
        >
          <div
            className={`toggle-switch ${proxyEnabled ? 'active' : ''}`}
            onClick={handleProxyToggle}
          />
        </SettingRow>
        {proxyEnabled && (
          <SettingRow
            label="Proxy URL"
            description="Proxy server address (e.g., http://proxy.example.com:8080)"
          >
            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                value={proxyUrl}
                onChange={(e) => setProxyUrl(e.target.value)}
                placeholder="http://proxy.example.com:8080"
                style={{ width: '300px' }}
              />
              <button className="btn" onClick={handleProxyUrlChange}>
                Save
              </button>
            </div>
          </SettingRow>
        )}
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

