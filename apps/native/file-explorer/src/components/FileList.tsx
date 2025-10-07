/**
 * File List Component
 * Virtualized file list with multiple view modes
 */

import { useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { FileItem } from './FileItem';
import type { FileEntry, ViewMode } from '../types';

interface FileListProps {
  entries: FileEntry[];
  viewMode: ViewMode;
  selectedPaths: Set<string>;
  onEntryClick: (entry: FileEntry, event: React.MouseEvent) => void;
  onEntryDoubleClick: (entry: FileEntry) => void;
  onEntryContextMenu: (entry: FileEntry, event: React.MouseEvent) => void;
  onKeyDown: (event: React.KeyboardEvent) => void;
}

export function FileList({
  entries,
  viewMode,
  selectedPaths,
  onEntryClick,
  onEntryDoubleClick,
  onEntryContextMenu,
  onKeyDown,
}: FileListProps) {
  const parentRef = useRef<HTMLDivElement>(null);

  // Calculate item size based on view mode
  const estimateSize = () => {
    switch (viewMode) {
      case 'grid':
        return 120; // Grid item height
      case 'compact':
        return 28; // Compact row height
      case 'list':
      default:
        return 48; // List row height
    }
  };

  // Virtualize rows
  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => parentRef.current,
    estimateSize,
    overscan: 10,
  });

  const virtualItems = virtualizer.getVirtualItems();

  if (entries.length === 0) {
    return (
      <div className="file-list-empty">
        <div className="empty-icon">📂</div>
        <div className="empty-text">This folder is empty</div>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className={`file-list file-list-${viewMode}`}
      onKeyDown={onKeyDown}
      tabIndex={0}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualItems.map((virtualItem) => {
          const entry = entries[virtualItem.index];
          const isSelected = selectedPaths.has(entry.path);

          return (
            <div
              key={virtualItem.key}
              data-index={virtualItem.index}
              ref={virtualizer.measureElement}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <FileItem
                entry={entry}
                isSelected={isSelected}
                viewMode={viewMode}
                onClick={onEntryClick}
                onDoubleClick={onEntryDoubleClick}
                onContextMenu={onEntryContextMenu}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
