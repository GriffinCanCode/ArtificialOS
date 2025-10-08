/**
 * Hub App
 * Application launcher and manager
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import type { NativeAppProps } from './sdk';
import type { AppMetadata, CategoryFilter } from './types';
import { useApps } from './hooks/useApps';
import { useFavorites } from './hooks/useFavorites';
import { useRecents } from './hooks/useRecents';
import { useKeyboard } from './hooks/useKeyboard';
import { SearchBar } from './components/SearchBar';
import { Sidebar } from './components/Sidebar';
import { AppGrid } from './components/AppGrid';
import { EmptyState } from './components/EmptyState';
import { launchApp } from './lib/api';
import './styles/App.css';

export default function HubApp({ context }: NativeAppProps) {
  const { window: win } = context;

  // State management
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<CategoryFilter>('all');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [stats, setStats] = useState<any>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Hooks
  const { filteredApps, loading, error, searchApps, filterByCategory, reload } = useApps();
  const { favorites, toggleFavorite } = useFavorites();
  const { recents, addRecent } = useRecents();

  /**
   * Launch an app
   */
  const handleLaunch = useCallback(
    async (app: AppMetadata) => {
      try {
        console.log('[Hub] Launching app:', app.name);

        const response = await launchApp(app.id);
        console.log('[Hub] App launched:', response.app_id);

        // Track in recents
        addRecent(app.id);

        // Close hub after launching
        setTimeout(() => {
          win.close();
        }, 300);
      } catch (err) {
        console.error('[Hub] Failed to launch app:', err);
        alert(`Failed to launch ${app.name}: ${(err as Error).message}`);
      }
    },
    [addRecent, win]
  );

  /**
   * Handle search input change
   */
  const handleSearch = useCallback(
    (query: string) => {
      setSearchQuery(query);
      searchApps(query);
      setSelectedIndex(0); // Reset selection
    },
    [searchApps]
  );

  /**
   * Handle category selection
   */
  const handleCategoryChange = useCallback(
    (category: CategoryFilter) => {
      setSelectedCategory(category);
      setSearchQuery(''); // Clear search
      filterByCategory(category, favorites, recents);
      setSelectedIndex(0); // Reset selection
    },
    [filterByCategory, favorites, recents]
  );

  /**
   * Handle keyboard navigation
   */
  const handleNavigate = useCallback(
    (direction: 'up' | 'down' | 'left' | 'right') => {
      const cols = 4; // Grid columns (update if CSS changes)
      const maxIndex = filteredApps.length - 1;

      setSelectedIndex((current) => {
        let next = current;

        if (direction === 'left') {
          next = Math.max(0, current - 1);
        } else if (direction === 'right') {
          next = Math.min(maxIndex, current + 1);
        } else if (direction === 'up') {
          next = Math.max(0, current - cols);
        } else if (direction === 'down') {
          next = Math.min(maxIndex, current + cols);
        }

        return next;
      });
    },
    [filteredApps.length]
  );

  /**
   * Handle keyboard selection
   */
  const handleSelect = useCallback(() => {
    if (filteredApps[selectedIndex]) {
      handleLaunch(filteredApps[selectedIndex]);
    }
  }, [filteredApps, selectedIndex, handleLaunch]);

  /**
   * Focus search input
   */
  const focusSearch = useCallback(() => {
    if (searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, []);

  /**
   * Close hub
   */
  const handleClose = useCallback(() => {
    win.close();
  }, [win]);

  // Keyboard shortcuts
  useKeyboard({
    onSearch: focusSearch,
    onNavigate: handleNavigate,
    onSelect: handleSelect,
    onEscape: handleClose,
  });

  // Initialize
  useEffect(() => {
    win.setTitle('🚀 App Hub');

    // Load initial stats (simulated from apps data)
    // In real implementation, stats come from backend
    const timer = setTimeout(() => {
      setStats({
        total_packages: filteredApps.length,
        categories: {},
      });
    }, 100);

    return () => clearTimeout(timer);
  }, [win, filteredApps.length]);

  // Update stats when apps change
  useEffect(() => {
    if (filteredApps.length > 0) {
      const categories: Record<string, number> = {};
      filteredApps.forEach((app) => {
        categories[app.category] = (categories[app.category] || 0) + 1;
      });
      setStats({
        total_packages: filteredApps.length,
        categories,
      });
    }
  }, [filteredApps]);

  return (
    <div className="hub-app">
      {/* Header */}
      <div className="hub-header">
        <SearchBar
          value={searchQuery}
          onChange={handleSearch}
          onClear={() => handleSearch('')}
          autoFocus={false}
        />
      </div>

      {/* Main content */}
      <div className="hub-content">
        {/* Sidebar */}
        <Sidebar
          selectedCategory={selectedCategory}
          onSelectCategory={handleCategoryChange}
          stats={stats}
          favoritesCount={favorites.size}
          recentsCount={recents.size}
        />

        {/* App grid */}
        <div className="hub-main">
          {loading && (
            <div className="hub-loading">
              <div className="loading-spinner">⏳</div>
              <div className="loading-text">Loading apps...</div>
            </div>
          )}

          {error && (
            <div className="hub-error">
              <div className="error-icon">⚠️</div>
              <div className="error-text">{error}</div>
              <button className="error-retry" onClick={reload}>
                Retry
              </button>
            </div>
          )}

          {!loading && !error && filteredApps.length === 0 && (
            <EmptyState
              icon={searchQuery ? '🔍' : '📦'}
              message={
                searchQuery
                  ? `No apps match "${searchQuery}"`
                  : 'No apps available in this category'
              }
            />
          )}

          {!loading && !error && filteredApps.length > 0 && (
            <AppGrid
              apps={filteredApps}
              favorites={favorites}
              onLaunch={handleLaunch}
              onToggleFavorite={toggleFavorite}
              selectedIndex={selectedIndex}
            />
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="hub-footer">
        <div className="hub-shortcuts">
          <kbd>/</kbd> Search • <kbd>↑↓←→</kbd> Navigate • <kbd>Enter</kbd> Launch •{' '}
          <kbd>Esc</kbd> Close
        </div>
        <div className="hub-stats">
          {filteredApps.length} {filteredApps.length === 1 ? 'app' : 'apps'}
        </div>
      </div>
    </div>
  );
}

