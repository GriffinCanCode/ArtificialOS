/**
 * Sidebar Component
 * Category navigation and filters
 */

import React from 'react';
import type { CategoryFilter, AppStats } from '../types';

interface SidebarProps {
  selectedCategory: CategoryFilter;
  onSelectCategory: (category: CategoryFilter) => void;
  stats: AppStats | null;
  favoritesCount: number;
  recentsCount: number;
}

export const Sidebar: React.FC<SidebarProps> = ({
  selectedCategory,
  onSelectCategory,
  stats,
  favoritesCount,
  recentsCount,
}) => {
  const categories = stats?.categories || {};
  const allCategories = Object.keys(categories).sort();

  return (
    <div className="sidebar">
      <div className="sidebar-section">
        <h3 className="sidebar-title">Collections</h3>
        <div className="sidebar-items">
          <button
            className={`sidebar-item ${selectedCategory === 'all' ? 'active' : ''}`}
            onClick={() => onSelectCategory('all')}
          >
            <span className="sidebar-icon">📦</span>
            <span className="sidebar-label">All Apps</span>
            <span className="sidebar-count">{stats?.total_packages || 0}</span>
          </button>

          <button
            className={`sidebar-item ${selectedCategory === 'favorites' ? 'active' : ''}`}
            onClick={() => onSelectCategory('favorites')}
          >
            <span className="sidebar-icon">⭐</span>
            <span className="sidebar-label">Favorites</span>
            <span className="sidebar-count">{favoritesCount}</span>
          </button>

          <button
            className={`sidebar-item ${selectedCategory === 'recent' ? 'active' : ''}`}
            onClick={() => onSelectCategory('recent')}
          >
            <span className="sidebar-icon">🕐</span>
            <span className="sidebar-label">Recent</span>
            <span className="sidebar-count">{recentsCount}</span>
          </button>
        </div>
      </div>

      {allCategories.length > 0 && (
        <div className="sidebar-section">
          <h3 className="sidebar-title">Categories</h3>
          <div className="sidebar-items">
            {allCategories.map((cat) => (
              <button
                key={cat}
                className={`sidebar-item ${selectedCategory === cat ? 'active' : ''}`}
                onClick={() => onSelectCategory(cat)}
              >
                <span className="sidebar-icon">{getCategoryIcon(cat)}</span>
                <span className="sidebar-label">{capitalize(cat)}</span>
                <span className="sidebar-count">{categories[cat]}</span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

function getCategoryIcon(category: string): string {
  const icons: Record<string, string> = {
    system: '🖥️',
    productivity: '📝',
    utilities: '🔧',
    developer: '💻',
    general: '📦',
  };
  return icons[category] || '📦';
}

function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

