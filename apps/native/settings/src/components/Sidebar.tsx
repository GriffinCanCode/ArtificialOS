/**
 * Settings Sidebar Navigation
 */

import type { Page } from '../App';
import './Sidebar.css';

interface SidebarProps {
  activePage: Page;
  onNavigate: (page: Page) => void;
  searchQuery: string;
  onSearch: (query: string) => void;
}

export function Sidebar({ activePage, onNavigate, searchQuery, onSearch }: SidebarProps) {
  const navItems: Array<{ id: Page; label: string; icon: string }> = [
    { id: 'general', label: 'General', icon: '⚙️' },
    { id: 'system', label: 'System', icon: '💻' },
    { id: 'appearance', label: 'Appearance', icon: '🎨' },
    { id: 'network', label: 'Network', icon: '🌐' },
    { id: 'permissions', label: 'Permissions', icon: '🔒' },
    { id: 'storage', label: 'Storage', icon: '💾' },
    { id: 'developer', label: 'Developer', icon: '🛠️' },
    { id: 'about', label: 'About', icon: 'ℹ️' },
  ];

  return (
    <div className="settings-sidebar">
      <div className="sidebar-header">
        <h2>Settings</h2>
      </div>

      <div className="sidebar-search">
        <input
          type="text"
          placeholder="Search settings..."
          value={searchQuery}
          onChange={(e) => onSearch(e.target.value)}
          className="search-input"
        />
      </div>

      <nav className="sidebar-nav">
        {navItems.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${activePage === item.id ? 'active' : ''}`}
            onClick={() => onNavigate(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            <span className="nav-label">{item.label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}

