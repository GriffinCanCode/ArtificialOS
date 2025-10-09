/**
 * Search Bar Component
 * Intelligent search with filters
 */

import { useState } from 'react';
import type { SearchFilters } from '../types';
import './SearchBar.css';

interface SearchBarProps {
  query: string;
  onQueryChange: (query: string) => void;
  onSearch: () => void;
  filters: SearchFilters;
  onFiltersChange: (filters: SearchFilters) => void;
}

export function SearchBar({
  query,
  onQueryChange,
  onSearch,
  filters,
  onFiltersChange,
}: SearchBarProps) {
  const [showFilters, setShowFilters] = useState(false);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      onSearch();
    }
  };

  return (
    <div className="search-bar">
      <div className="search-input-container">
        <span className="search-icon">🔍</span>
        <input
          type="text"
          className="search-input"
          placeholder="Search files..."
          value={query}
          onChange={e => onQueryChange(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        {query && (
          <button
            className="clear-button"
            onClick={() => onQueryChange('')}
            title="Clear"
          >
            ×
          </button>
        )}
        <button
          className={`filter-button ${showFilters ? 'active' : ''}`}
          onClick={() => setShowFilters(!showFilters)}
          title="Filters"
        >
          ⚙️
        </button>
      </div>

      {showFilters && (
        <div className="search-filters">
          <div className="filter-group">
            <label className="filter-label">Type</label>
            <select
              className="filter-select"
              value={filters.type || 'all'}
              onChange={e => onFiltersChange({ ...filters, type: e.target.value as any })}
            >
              <option value="all">All</option>
              <option value="files">Files</option>
              <option value="folders">Folders</option>
              <option value="images">Images</option>
              <option value="documents">Documents</option>
              <option value="code">Code</option>
            </select>
          </div>

          <div className="filter-group">
            <label className="filter-label">Size</label>
            <div className="filter-row">
              <input
                type="number"
                className="filter-input"
                placeholder="Min (bytes)"
                value={filters.minSize || ''}
                onChange={e => onFiltersChange({ ...filters, minSize: e.target.value ? Number(e.target.value) : undefined })}
              />
              <span className="filter-separator">to</span>
              <input
                type="number"
                className="filter-input"
                placeholder="Max (bytes)"
                value={filters.maxSize || ''}
                onChange={e => onFiltersChange({ ...filters, maxSize: e.target.value ? Number(e.target.value) : undefined })}
              />
            </div>
          </div>

          <button
            className="filter-reset"
            onClick={() => onFiltersChange({})}
          >
            Reset Filters
          </button>
        </div>
      )}
    </div>
  );
}

