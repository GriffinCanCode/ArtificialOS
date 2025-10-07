/**
 * File Item Component
 * Individual file/folder display
 */

import { memo } from 'react';
import type { FileItemProps } from '../types';
import { getFileIcon, formatFileSize, formatDate } from '../utils';

export const FileItem = memo(function FileItem({
  entry,
  isSelected,
  viewMode,
  onClick,
  onDoubleClick,
  onContextMenu,
}: FileItemProps) {
  const icon = getFileIcon(entry);

  const handleClick = (e: React.MouseEvent) => {
    e.preventDefault();
    onClick(entry, e);
  };

  const handleDoubleClick = () => {
    onDoubleClick(entry);
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    onContextMenu(entry, e);
  };

  if (viewMode === 'grid') {
    return (
      <div
        className={`file-item file-item-grid ${isSelected ? 'selected' : ''}`}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        onContextMenu={handleContextMenu}
        title={entry.name}
      >
        <div className="file-icon" style={{ color: icon.color }}>
          {icon.emoji}
        </div>
        <div className="file-name">{entry.name}</div>
      </div>
    );
  }

  if (viewMode === 'compact') {
    return (
      <div
        className={`file-item file-item-compact ${isSelected ? 'selected' : ''}`}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        onContextMenu={handleContextMenu}
        title={entry.name}
      >
        <span className="file-icon" style={{ color: icon.color }}>
          {icon.emoji}
        </span>
        <span className="file-name">{entry.name}</span>
      </div>
    );
  }

  // List view (default)
  return (
    <div
      className={`file-item file-item-list ${isSelected ? 'selected' : ''}`}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onContextMenu={handleContextMenu}
    >
      <div className="file-icon-cell" style={{ color: icon.color }}>
        {icon.emoji}
      </div>
      <div className="file-name-cell">{entry.name}</div>
      <div className="file-size-cell">
        {entry.is_dir ? '—' : formatFileSize(entry.size)}
      </div>
      <div className="file-date-cell">{formatDate(entry.modified)}</div>
    </div>
  );
});
