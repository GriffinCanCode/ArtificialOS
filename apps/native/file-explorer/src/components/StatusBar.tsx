/**
 * Status Bar Component
 * Bottom status information
 */

import type { FileEntry } from '../types';
import './StatusBar.css';

interface StatusBarProps {
  currentPath: string;
  itemCount: number;
  selectedEntry: FileEntry | null;
  loading: boolean;
}

export function StatusBar({ currentPath, itemCount, selectedEntry, loading }: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-left">
        <span className="status-path">{currentPath}</span>
      </div>

      <div className="status-right">
        {loading ? (
          <span className="status-loading">Loading...</span>
        ) : (
          <>
            <span className="status-item">
              {itemCount} {itemCount === 1 ? 'item' : 'items'}
            </span>
            {selectedEntry && (
              <span className="status-item status-selected">
                {selectedEntry.name}
              </span>
            )}
          </>
        )}
      </div>
    </div>
  );
}

