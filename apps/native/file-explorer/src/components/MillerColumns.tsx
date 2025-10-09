/**
 * Miller Columns Component
 * Spatial navigation with multiple columns showing hierarchy
 */

import { useRef, useEffect } from 'react';
import type { Column, FileEntry } from '../types';
import { getFileIcon, formatFileSize, formatDate } from '../utils';
import './MillerColumns.css';

interface MillerColumnsProps {
  columns: Column[];
  onSelect: (columnIndex: number, entry: FileEntry) => void;
  onNavigate: (path: string) => void;
  selectedEntry: FileEntry | null;
  showHidden: boolean;
}

export function MillerColumns({
  columns,
  onSelect,
  onNavigate,
  selectedEntry,
  showHidden,
}: MillerColumnsProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const lastColumnRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to show the last column
  useEffect(() => {
    if (lastColumnRef.current && containerRef.current) {
      const container = containerRef.current;
      const lastColumn = lastColumnRef.current;
      const scrollLeft = lastColumn.offsetLeft - container.offsetWidth / 3;

      container.scrollTo({
        left: Math.max(0, scrollLeft),
        behavior: 'smooth',
      });
    }
  }, [columns.length]);

  const filterEntries = (entries: FileEntry[]) => {
    if (showHidden) return entries;
    return entries.filter(entry => !entry.name.startsWith('.'));
  };

  return (
    <div ref={containerRef} className="miller-columns">
      {columns.map((column, columnIndex) => {
        const isLast = columnIndex === columns.length - 1;
        const filteredEntries = filterEntries(column.entries);

        return (
          <div
            key={column.id}
            ref={isLast ? lastColumnRef : null}
            className={`miller-column ${isLast ? 'active' : ''}`}
          >
            {/* Column Header */}
            <div className="column-header">
              <button
                className="column-title"
                onClick={() => onNavigate(column.path)}
                title={column.path}
              >
                {column.name}
              </button>
              <div className="column-count">
                {filteredEntries.length}
              </div>
            </div>

            {/* Column Content */}
            <div className="column-content">
              {column.loading ? (
                <div className="column-loading">
                  <div className="spinner">↻</div>
                  <div className="loading-text">Loading...</div>
                </div>
              ) : filteredEntries.length === 0 ? (
                <div className="column-empty">
                  <div className="empty-icon">📂</div>
                  <div className="empty-text">Empty folder</div>
                </div>
              ) : (
                <div className="column-items">
                  {filteredEntries.map((entry, index) => {
                    const icon = getFileIcon(entry);
                    const isSelected = selectedEntry?.path === entry.path;

                    return (
                      <div
                        key={entry.path}
                        className={`column-item ${isSelected ? 'selected' : ''} ${
                          entry.is_dir ? 'directory' : 'file'
                        }`}
                        onClick={() => onSelect(columnIndex, entry)}
                        onDoubleClick={() => {
                          if (entry.is_dir) {
                            onNavigate(entry.path);
                          }
                        }}
                      >
                        <div className="item-icon" style={{ color: icon.color }}>
                          {icon.emoji}
                        </div>
                        <div className="item-details">
                          <div className="item-name">
                            {entry.name}
                            {entry.tags && entry.tags.length > 0 && (
                              <div className="item-tags">
                                {entry.tags.map(tag => (
                                  <span key={tag} className="tag">
                                    {tag}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                          {!entry.is_dir && (
                            <div className="item-meta">
                              {formatFileSize(entry.size)} · {formatDate(entry.modified)}
                            </div>
                          )}
                        </div>
                        {entry.is_dir && (
                          <div className="item-arrow">›</div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

