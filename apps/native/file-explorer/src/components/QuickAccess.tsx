/**
 * Quick Access Component
 * Favorites and recent locations (Cmd+O)
 */

import { useState } from 'react';
import './QuickAccess.css';

interface QuickAccessProps {
  favorites: string[];
  recent: string[];
  onSelect: (path: string) => void;
  onClose: () => void;
}

export function QuickAccess({ favorites, recent, onSelect, onClose }: QuickAccessProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);

  const allItems = [
    ...favorites.map(path => ({ path, type: 'favorite' as const })),
    ...recent.map(path => ({ path, type: 'recent' as const })),
  ];

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, allItems.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (allItems[selectedIndex]) {
          onSelect(allItems[selectedIndex].path);
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  };

  const getName = (path: string) => {
    return path.split('/').pop() || path;
  };

  return (
    <div className="quick-access-overlay" onClick={onClose}>
      <div className="quick-access" onClick={e => e.stopPropagation()} onKeyDown={handleKeyDown}>
        {/* Header */}
        <div className="quick-access-header">
          <span className="header-icon">⚡</span>
          <h3 className="header-title">Quick Access</h3>
          <button className="close-button" onClick={onClose}>×</button>
        </div>

        {/* Content */}
        <div className="quick-access-content">
          {favorites.length > 0 && (
            <div className="access-section">
              <div className="section-label">Favorites</div>
              {favorites.map((path, index) => {
                const globalIndex = index;
                const isSelected = globalIndex === selectedIndex;

                return (
                  <button
                    key={`fav-${index}`}
                    className={`access-item ${isSelected ? 'selected' : ''}`}
                    onClick={() => onSelect(path)}
                    onMouseEnter={() => setSelectedIndex(globalIndex)}
                  >
                    <span className="item-icon">⭐</span>
                    <div className="item-info">
                      <div className="item-name">{getName(path)}</div>
                      <div className="item-path">{path}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          )}

          {recent.length > 0 && (
            <div className="access-section">
              <div className="section-label">Recent</div>
              {recent.slice(0, 10).map((path, index) => {
                const globalIndex = favorites.length + index;
                const isSelected = globalIndex === selectedIndex;

                return (
                  <button
                    key={`recent-${index}`}
                    className={`access-item ${isSelected ? 'selected' : ''}`}
                    onClick={() => onSelect(path)}
                    onMouseEnter={() => setSelectedIndex(globalIndex)}
                  >
                    <span className="item-icon">🕐</span>
                    <div className="item-info">
                      <div className="item-name">{getName(path)}</div>
                      <div className="item-path">{path}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          )}

          {favorites.length === 0 && recent.length === 0 && (
            <div className="access-empty">
              <div className="empty-icon">📁</div>
              <div className="empty-text">No quick access items yet</div>
              <div className="empty-hint">Add favorites from the command palette</div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="quick-access-footer">
          <span className="footer-hint">
            <kbd>↑</kbd> <kbd>↓</kbd> to navigate · <kbd>↵</kbd> to select · <kbd>esc</kbd> to close
          </span>
        </div>
      </div>
    </div>
  );
}

