/**
 * Command Palette Component
 * Power user command interface (Cmd+P)
 */

import { useState, useEffect, useRef } from 'react';
import type { Command } from '../types';
import './CommandPalette.css';

interface CommandPaletteProps {
  onCommand: (command: string, payload?: any) => void;
  onClose: () => void;
  recentPaths: string[];
  favorites: string[];
}

export function CommandPalette({
  onCommand,
  onClose,
  recentPaths,
  favorites,
}: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Build available commands
  const commands: Command[] = [
    // Navigation
    { id: 'quick-access', name: 'Quick Access', icon: '⚡', category: 'navigation', handler: () => onCommand('quick-access') },

    // Recent paths
    ...recentPaths.slice(0, 5).map((path, i) => ({
      id: `recent-${i}`,
      name: `Recent: ${path.split('/').pop() || path}`,
      description: path,
      icon: '🕐',
      category: 'navigation' as const,
      handler: () => onCommand('go-to', { path }),
    })),

    // Favorites
    ...favorites.map((path, i) => ({
      id: `fav-${i}`,
      name: `Favorite: ${path.split('/').pop() || path}`,
      description: path,
      icon: '⭐',
      category: 'navigation' as const,
      handler: () => onCommand('go-to', { path }),
    })),

    // File operations
    { id: 'new-folder', name: 'New Folder', icon: '📁', category: 'file', handler: () => {
      const name = prompt('Folder name:');
      if (name) onCommand('new-folder', { name });
    }},
    { id: 'add-favorite', name: 'Add to Favorites', icon: '⭐', category: 'file', handler: () => onCommand('add-favorite') },
    { id: 'copy-path', name: 'Copy Path', icon: '📋', category: 'file', handler: () => onCommand('copy-path') },

    // View
    { id: 'toggle-preview', name: 'Toggle Preview', icon: '👁️', category: 'view', shortcut: 'Space', handler: () => onCommand('toggle-preview') },
    { id: 'toggle-hidden', name: 'Toggle Hidden Files', icon: '🙈', category: 'view', handler: () => onCommand('toggle-hidden') },

    // System
    { id: 'open-terminal', name: 'Open in Terminal', icon: '⌨️', category: 'system', handler: () => onCommand('open-terminal') },
  ];

  // Filter commands based on query
  const filteredCommands = query.trim()
    ? commands.filter(cmd =>
        cmd.name.toLowerCase().includes(query.toLowerCase()) ||
        cmd.description?.toLowerCase().includes(query.toLowerCase())
      )
    : commands;

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, filteredCommands.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          filteredCommands[selectedIndex].handler();
          onClose();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  };

  // Group commands by category
  const groupedCommands = filteredCommands.reduce((acc, cmd) => {
    const category = cmd.category || 'other';
    if (!acc[category]) acc[category] = [];
    acc[category].push(cmd);
    return acc;
  }, {} as Record<string, Command[]>);

  const categoryNames: Record<string, string> = {
    navigation: 'Navigation',
    file: 'File Operations',
    view: 'View',
    search: 'Search',
    system: 'System',
    other: 'Other',
  };

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={e => e.stopPropagation()}>
        {/* Search Input */}
        <div className="palette-search">
          <span className="search-icon">🔍</span>
          <input
            ref={inputRef}
            type="text"
            className="search-input"
            placeholder="Type a command or search..."
            value={query}
            onChange={e => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
          />
        </div>

        {/* Commands List */}
        <div className="palette-results">
          {Object.entries(groupedCommands).map(([category, cmds]) => (
            <div key={category} className="command-group">
              <div className="group-label">{categoryNames[category]}</div>
              {cmds.map((cmd, index) => {
                const globalIndex = filteredCommands.indexOf(cmd);
                const isSelected = globalIndex === selectedIndex;

                return (
                  <button
                    key={cmd.id}
                    className={`command-item ${isSelected ? 'selected' : ''}`}
                    onClick={() => {
                      cmd.handler();
                      onClose();
                    }}
                    onMouseEnter={() => setSelectedIndex(globalIndex)}
                  >
                    <span className="command-icon">{cmd.icon}</span>
                    <div className="command-info">
                      <div className="command-name">{cmd.name}</div>
                      {cmd.description && (
                        <div className="command-description">{cmd.description}</div>
                      )}
                    </div>
                    {cmd.shortcut && (
                      <div className="command-shortcut">{cmd.shortcut}</div>
                    )}
                  </button>
                );
              })}
            </div>
          ))}

          {filteredCommands.length === 0 && (
            <div className="no-results">
              <div className="no-results-icon">🔍</div>
              <div className="no-results-text">No commands found</div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="palette-footer">
          <span className="footer-hint">
            <kbd>↑</kbd> <kbd>↓</kbd> to navigate · <kbd>↵</kbd> to select · <kbd>esc</kbd> to close
          </span>
        </div>
      </div>
    </div>
  );
}

