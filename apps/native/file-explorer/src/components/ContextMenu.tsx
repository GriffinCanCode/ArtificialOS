/**
 * Context Menu Component
 * Right-click menu for file operations
 */

import { useEffect, useRef } from 'react';
import type { ContextMenuProps, ContextAction } from '../types';

export function ContextMenu({
  x,
  y,
  entry,
  selectedCount,
  onClose,
  onAction,
}: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  // Close on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    // Close on escape
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [onClose]);

  const handleAction = (action: ContextAction) => {
    onAction(action);
    onClose();
  };

  const hasSelection = selectedCount > 0;

  return (
    <div
      ref={menuRef}
      className="context-menu"
      style={{
        left: `${x}px`,
        top: `${y}px`,
      }}
    >
      {entry && (
        <>
          <button
            className="context-menu-item"
            onClick={() => handleAction('open')}
          >
            <span className="menu-icon">📂</span>
            <span className="menu-label">Open</span>
          </button>
          <div className="context-menu-divider" />
        </>
      )}

      {hasSelection && (
        <>
          <button
            className="context-menu-item"
            onClick={() => handleAction('copy')}
          >
            <span className="menu-icon">📋</span>
            <span className="menu-label">Copy</span>
            <span className="menu-shortcut">Ctrl+C</span>
          </button>
          <button
            className="context-menu-item"
            onClick={() => handleAction('cut')}
          >
            <span className="menu-icon">✂️</span>
            <span className="menu-label">Cut</span>
            <span className="menu-shortcut">Ctrl+X</span>
          </button>
        </>
      )}

      <button
        className="context-menu-item"
        onClick={() => handleAction('paste')}
      >
        <span className="menu-icon">📄</span>
        <span className="menu-label">Paste</span>
        <span className="menu-shortcut">Ctrl+V</span>
      </button>

      {hasSelection && (
        <>
          <div className="context-menu-divider" />
          <button
            className="context-menu-item"
            onClick={() => handleAction('rename')}
          >
            <span className="menu-icon">✏️</span>
            <span className="menu-label">Rename</span>
            <span className="menu-shortcut">F2</span>
          </button>
          <button
            className="context-menu-item context-menu-item-danger"
            onClick={() => handleAction('delete')}
          >
            <span className="menu-icon">🗑️</span>
            <span className="menu-label">Delete</span>
            <span className="menu-shortcut">Del</span>
          </button>
        </>
      )}

      <div className="context-menu-divider" />

      <button
        className="context-menu-item"
        onClick={() => handleAction('new-folder')}
      >
        <span className="menu-icon">📁</span>
        <span className="menu-label">New Folder</span>
        <span className="menu-shortcut">Ctrl+Shift+N</span>
      </button>

      <button
        className="context-menu-item"
        onClick={() => handleAction('refresh')}
      >
        <span className="menu-icon">↻</span>
        <span className="menu-label">Refresh</span>
        <span className="menu-shortcut">F5</span>
      </button>

      {entry && (
        <>
          <div className="context-menu-divider" />
          <button
            className="context-menu-item"
            onClick={() => handleAction('properties')}
          >
            <span className="menu-icon">ℹ️</span>
            <span className="menu-label">Properties</span>
          </button>
        </>
      )}
    </div>
  );
}
