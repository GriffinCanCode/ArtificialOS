/**
 * Tab Bar Component
 * Multi-tab management UI
 */

import type { BrowserTab } from '../types';
import './TabBar.css';

interface TabBarProps {
  tabs: BrowserTab[];
  activeTabId: string | null;
  onTabClick: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onNewTab: () => void;
}

export function TabBar({
  tabs,
  activeTabId,
  onTabClick,
  onTabClose,
  onNewTab,
}: TabBarProps) {
  return (
    <div className="tab-bar">
      <div className="tabs-container">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`browser-tab ${tab.id === activeTabId ? 'active' : ''} ${
              tab.isPinned ? 'pinned' : ''
            }`}
            onClick={() => onTabClick(tab.id)}
          >
            <div className="tab-content">
              {tab.favicon ? (
                <img src={tab.favicon} alt="" className="tab-favicon" />
              ) : (
                <span className="tab-icon">🌐</span>
              )}
              <span className="tab-title">{tab.title}</span>
              {tab.loading && <span className="tab-loading">⟳</span>}
            </div>
            <button
              className="tab-close-btn"
              onClick={(e) => {
                e.stopPropagation();
                onTabClose(tab.id);
              }}
              title="Close tab (Ctrl+W)"
            >
              ×
            </button>
          </div>
        ))}
      </div>

      <button className="new-tab-btn" onClick={onNewTab} title="New tab (Ctrl+T)">
        +
      </button>
    </div>
  );
}

