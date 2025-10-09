/**
 * Storage Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface StoragePageProps {
  executor: any;
  state: any;
}

export function StoragePage({ executor }: StoragePageProps) {
  const [storageItems, setStorageItems] = useState<string[]>([]);

  useEffect(() => {
    loadStorageInfo();
  }, []);

  const loadStorageInfo = async () => {
    try {
      const result = await executor.execute('storage.list', {});
      if (result?.keys) {
        setStorageItems(result.keys);
      }
    } catch (error) {
      console.error('Failed to load storage info:', error);
    }
  };

  const handleClearAll = async () => {
    if (!confirm('Are you sure you want to clear all storage? This action cannot be undone.')) {
      return;
    }

    try {
      await executor.execute('storage.clear', {});
      setStorageItems([]);
    } catch (error) {
      console.error('Failed to clear storage:', error);
    }
  };

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>Storage Management</h1>
        <p className="page-description">Manage application data and cache</p>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Storage Usage</h2>
        <div className="storage-info">
          <div className="storage-stat">
            <div className="stat-label">Total Items</div>
            <div className="stat-value">{storageItems.length}</div>
          </div>
        </div>
      </div>

      {storageItems.length > 0 && (
        <div className="settings-section">
          <h2 className="section-title">Stored Keys ({storageItems.length})</h2>
          <div className="storage-list">
            {storageItems.map((key) => (
              <div key={key} className="storage-item">
                <div className="storage-key">{key}</div>
                <button
                  className="btn-secondary"
                  onClick={async () => {
                    try {
                      await executor.execute('storage.remove', { key });
                      loadStorageInfo();
                    } catch (error) {
                      console.error('Failed to remove item:', error);
                    }
                  }}
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="settings-section">
        <h2 className="section-title">Maintenance</h2>
        <SettingRow
          label="Clear All Storage"
          description="Remove all stored data and cache"
        >
          <button className="btn-secondary" onClick={handleClearAll}>
            Clear All
          </button>
        </SettingRow>
      </div>
    </div>
  );
}

