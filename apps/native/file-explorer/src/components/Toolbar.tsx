/**
 * Toolbar Component
 * Action buttons and view controls
 */

import { type ViewMode, type SortField, type SortDirection } from '../types';

interface ToolbarProps {
  viewMode: ViewMode;
  sortField: SortField;
  sortDirection: SortDirection;
  searchQuery: string;
  canGoBack: boolean;
  canGoForward: boolean;
  onGoBack: () => void;
  onGoForward: () => void;
  onGoUp: () => void;
  onRefresh: () => void;
  onNewFolder: () => void;
  onViewModeChange: (mode: ViewMode) => void;
  onSortChange: (field: SortField) => void;
  onSearchChange: (query: string) => void;
}

export function Toolbar({
  viewMode,
  sortField,
  searchQuery,
  canGoBack,
  canGoForward,
  onGoBack,
  onGoForward,
  onGoUp,
  onRefresh,
  onNewFolder,
  onViewModeChange,
  onSortChange,
  onSearchChange,
}: ToolbarProps) {
  return (
    <div className="toolbar">
      {/* Navigation */}
      <div className="toolbar-group">
        <button
          className="toolbar-btn"
          onClick={onGoBack}
          disabled={!canGoBack}
          title="Back (Alt+Left)"
        >
          ←
        </button>
        <button
          className="toolbar-btn"
          onClick={onGoForward}
          disabled={!canGoForward}
          title="Forward (Alt+Right)"
        >
          →
        </button>
        <button className="toolbar-btn" onClick={onGoUp} title="Up (Alt+Up)">
          ↑
        </button>
        <button className="toolbar-btn" onClick={onRefresh} title="Refresh (F5)">
          ↻
        </button>
      </div>

      {/* Actions */}
      <div className="toolbar-group">
        <button className="toolbar-btn" onClick={onNewFolder} title="New Folder (Ctrl+Shift+N)">
          📁+
        </button>
      </div>

      {/* View Mode */}
      <div className="toolbar-group">
        <button
          className={`toolbar-btn ${viewMode === 'list' ? 'active' : ''}`}
          onClick={() => onViewModeChange('list')}
          title="List View"
        >
          ≡
        </button>
        <button
          className={`toolbar-btn ${viewMode === 'grid' ? 'active' : ''}`}
          onClick={() => onViewModeChange('grid')}
          title="Grid View"
        >
          ⊞
        </button>
        <button
          className={`toolbar-btn ${viewMode === 'compact' ? 'active' : ''}`}
          onClick={() => onViewModeChange('compact')}
          title="Compact View"
        >
          ☰
        </button>
      </div>

      {/* Sort */}
      <div className="toolbar-group">
        <select
          className="toolbar-select"
          value={sortField}
          onChange={(e) => onSortChange(e.target.value as SortField)}
          title="Sort By"
        >
          <option value="name">Name</option>
          <option value="size">Size</option>
          <option value="modified">Modified</option>
          <option value="type">Type</option>
        </select>
      </div>

      {/* Search */}
      <div className="toolbar-group toolbar-search">
        <input
          type="text"
          className="toolbar-input"
          placeholder="Search..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>
    </div>
  );
}
