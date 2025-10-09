/**
 * System Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface SystemPageProps {
  executor: any;
  state: any;
}

export function SystemPage({ executor }: SystemPageProps) {
  const [systemStats, setSystemStats] = useState<any>(null);
  const [processes, setProcesses] = useState<any[]>([]);
  const [memoryLimit, setMemoryLimit] = useState(1024);
  const [cpuCores, setCpuCores] = useState(0);
  const [logLevel, setLogLevel] = useState('info');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSettings();
    loadSystemStats();
    const interval = setInterval(loadSystemStats, 5000); // Update every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const loadSettings = async () => {
    try {
      const memResult = await executor.execute('settings.get', { key: 'system.memory_limit' });
      if (memResult?.value) setMemoryLimit(memResult.value);

      const cpuResult = await executor.execute('settings.get', { key: 'system.cpu_cores' });
      if (cpuResult?.value !== undefined) setCpuCores(cpuResult.value);

      const logResult = await executor.execute('settings.get', { key: 'system.log_level' });
      if (logResult?.value) setLogLevel(logResult.value);

      setLoading(false);
    } catch (error) {
      console.error('Failed to load settings:', error);
      setLoading(false);
    }
  };

  const loadSystemStats = async () => {
    try {
      const stats = await executor.execute('monitor.system', {});
      setSystemStats(stats);

      const procResult = await executor.execute('monitor.processes', {});
      if (procResult?.processes) {
        setProcesses(procResult.processes);
      }
    } catch (error) {
      console.error('Failed to load system stats:', error);
    }
  };

  const handleMemoryLimitChange = async (value: number) => {
    setMemoryLimit(value);
    try {
      await executor.execute('settings.set', { key: 'system.memory_limit', value });
    } catch (error) {
      console.error('Failed to update memory limit:', error);
    }
  };

  const handleLogLevelChange = async (value: string) => {
    setLogLevel(value);
    try {
      await executor.execute('settings.set', { key: 'system.log_level', value });
    } catch (error) {
      console.error('Failed to update log level:', error);
    }
  };

  if (loading) {
    return <div className="settings-page loading">Loading...</div>;
  }

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>System Settings</h1>
        <p className="page-description">Monitor and configure system resources</p>
      </div>

      {systemStats && (
        <div className="settings-section">
          <h2 className="section-title">System Status</h2>
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-label">Memory Usage</div>
              <div className="stat-value">
                {systemStats.memory?.usage_percent?.toFixed(1)}%
              </div>
              <div className="stat-detail">
                {formatBytes(systemStats.memory?.allocated_bytes)} / {formatBytes(systemStats.memory?.system_bytes)}
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-label">CPU Cores</div>
              <div className="stat-value">{systemStats.cpu?.cores}</div>
              <div className="stat-detail">{systemStats.cpu?.threads} threads</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Processes</div>
              <div className="stat-value">{systemStats.processes}</div>
              <div className="stat-detail">{systemStats.goroutines} goroutines</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Garbage Collection</div>
              <div className="stat-value">{systemStats.memory?.num_gc}</div>
              <div className="stat-detail">{systemStats.memory?.gc_pauses}ms total</div>
            </div>
          </div>
        </div>
      )}

      <div className="settings-section">
        <h2 className="section-title">Resource Limits</h2>
        <SettingRow
          label="Memory Limit"
          description="Maximum memory usage in MB (0 = unlimited)"
        >
          <input
            type="number"
            value={memoryLimit}
            onChange={(e) => handleMemoryLimitChange(Number(e.target.value))}
            min="0"
            step="256"
            style={{ width: '100px' }}
          />
        </SettingRow>
        <SettingRow
          label="CPU Cores"
          description="Number of CPU cores to use (0 = auto)"
        >
          <input
            type="number"
            value={cpuCores}
            onChange={(e) => setCpuCores(Number(e.target.value))}
            min="0"
            max={systemStats?.cpu?.cores || 16}
            style={{ width: '100px' }}
          />
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Logging</h2>
        <SettingRow
          label="Log Level"
          description="Set the system logging verbosity"
        >
          <select value={logLevel} onChange={(e) => handleLogLevelChange(e.target.value)}>
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warning</option>
            <option value="error">Error</option>
          </select>
        </SettingRow>
      </div>

      {processes.length > 0 && (
        <div className="settings-section">
          <h2 className="section-title">Running Processes ({processes.length})</h2>
          <div className="process-list">
            {processes.slice(0, 10).map((proc) => (
              <div key={proc.pid} className="process-item">
                <div className="process-info">
                  <div className="process-name">{proc.name || `PID ${proc.pid}`}</div>
                  <div className="process-state">{proc.state}</div>
                </div>
                <div className="process-pid">#{proc.pid}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

