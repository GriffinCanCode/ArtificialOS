/**
 * Revolutionary File Explorer - Miller Columns with Intelligence
 *
 * Key Innovations:
 * - Miller Columns for spatial navigation and context
 * - Command Palette (Cmd+P) for power users
 * - Smart inline previews (images, code, text)
 * - Intelligent search with multiple filters
 * - Tag system with colors
 * - Quick access to favorites and recent
 * - Beautiful animations and polish
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import type { NativeAppProps } from './sdk';
import type { FileEntry, Column } from './types';
import { useFileSystem } from './hooks/useFileSystem';
import { useKeyboard } from './hooks/useKeyboard';
import { usePreferences } from './hooks/usePreferences';
import { useSearch } from './hooks/useSearch';
import { getFileIcon, formatFileSize, formatDate } from './utils';
import { MillerColumns } from './components/MillerColumns';
import { CommandPalette } from './components/CommandPalette';
import { QuickAccess } from './components/QuickAccess';
import { PreviewPanel } from './components/PreviewPanel';
import { SearchBar } from './components/SearchBar';
import { StatusBar } from './components/StatusBar';
import './styles/App.css';

export default function FileExplorerApp({ context }: NativeAppProps) {
  const { window: win, executor } = context;

  // Core state
  const fs = useFileSystem(context);
  const preferences = usePreferences(context);
  const search = useSearch(context, fs);

  // UI state
  const [columns, setColumns] = useState<Column[]>([]);
  const [selectedEntry, setSelectedEntry] = useState<FileEntry | null>(null);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showQuickAccess, setShowQuickAccess] = useState(false);
  const [previewCollapsed, setPreviewCollapsed] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [viewMode, setViewMode] = useState<'columns' | 'list' | 'grid'>('columns');

  // Refs
  const columnsRef = useRef<HTMLDivElement>(null);

  // ============================================================================
  // Lifecycle
  // ============================================================================

  useEffect(() => {
    // Initialize with home directory
    const initialPath = preferences.data.lastPath || '/storage';
    fs.navigate(initialPath);
    win.setTitle('📁 Explorer');
  }, []);

  useEffect(() => {
    // Build Miller columns from current path
    if (fs.currentPath && !fs.loading) {
      buildColumns(fs.currentPath);
    }
  }, [fs.currentPath, fs.entries]);

  // ============================================================================
  // Column Management
  // ============================================================================

  const buildColumns = useCallback((path: string) => {
    const parts = path.split('/').filter(Boolean);
    const newColumns: Column[] = [];

    // Root column
    newColumns.push({
      id: 'root',
      path: '/',
      name: 'Root',
      entries: [{ name: 'storage', path: '/storage', size: 0, modified: '', is_dir: true }],
      selectedIndex: parts.length > 0 ? 0 : -1,
    });

    // Build each level
    let currentPath = '';
    for (let i = 0; i < parts.length; i++) {
      currentPath += '/' + parts[i];
      const isLast = i === parts.length - 1;

      newColumns.push({
        id: currentPath,
        path: currentPath,
        name: parts[i],
        entries: isLast ? fs.entries : [],
        selectedIndex: -1,
        loading: isLast && fs.loading,
      });
    }

    setColumns(newColumns);
  }, [fs.entries, fs.loading]);

  const handleColumnSelect = useCallback(async (columnIndex: number, entry: FileEntry) => {
    setSelectedEntry(entry);

    if (entry.is_dir) {
      // Navigate into directory
      await fs.navigate(entry.path);

      // Save to recent
      preferences.addRecent(entry.path);
    } else {
      // Select file for preview
      setPreviewCollapsed(false);
    }
  }, [fs, preferences]);

  const handleColumnNavigate = useCallback(async (path: string) => {
    await fs.navigate(path);
    preferences.addRecent(path);
  }, [fs, preferences]);

  // ============================================================================
  // Command Palette
  // ============================================================================

  const handleCommand = useCallback(async (command: string, payload?: any) => {
    switch (command) {
      case 'go-to':
        await fs.navigate(payload.path);
        break;

      case 'search':
        search.setQuery(payload.query);
        search.execute();
        break;

      case 'new-folder':
        const name = payload.name;
        await fs.createFolder(name);
        break;

      case 'toggle-preview':
        setPreviewCollapsed(prev => !prev);
        break;

      case 'toggle-hidden':
        preferences.toggleShowHidden();
        break;

      case 'add-favorite':
        if (selectedEntry) {
          preferences.addFavorite(selectedEntry.path);
        }
        break;

      case 'copy-path':
        if (selectedEntry) {
          await executor.execute('clipboard.write', { text: selectedEntry.path });
        }
        break;

      case 'open-terminal':
        await executor.execute('app.spawn', {
          request: `Terminal at ${fs.currentPath}`,
          appId: 'terminal',
          params: { cwd: fs.currentPath },
        });
        break;

      case 'quick-access':
        setShowQuickAccess(true);
        break;

      default:
        console.log('Unknown command:', command);
    }

    setShowCommandPalette(false);
  }, [fs, search, selectedEntry, preferences, executor]);

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  const keyboard = useKeyboard({
    onCommandP: () => setShowCommandPalette(true),
    onCommandO: () => setShowQuickAccess(true),
    onEscape: () => {
      setShowCommandPalette(false);
      setShowQuickAccess(false);
    },
    onCommandEnter: () => {
      if (selectedEntry && !selectedEntry.is_dir) {
        openFile(selectedEntry);
      }
    },
    onSpace: () => {
      setPreviewCollapsed(prev => !prev);
    },
  });

  // ============================================================================
  // File Operations
  // ============================================================================

  const openFile = async (entry: FileEntry) => {
    const ext = entry.name.split('.').pop()?.toLowerCase() || '';

    // Smart file opening based on type
    const handlers: Record<string, () => Promise<void>> = {
      // Text files - open in editor
      txt: async () => {
        const content = await executor.execute('filesystem.read', { path: entry.path });
        await executor.execute('app.spawn', {
          request: `Edit ${entry.name}`,
          blueprint: {
            title: entry.name,
            components: [
              {
                type: 'container',
                props: { style: { padding: '20px', height: '100%' } },
                children: [
                  {
                    type: 'textarea',
                    props: {
                      value: content?.content || '',
                      style: {
                        width: '100%',
                        height: '100%',
                        fontFamily: 'monospace',
                        fontSize: '14px',
                        background: '#1e1e1e',
                        color: '#d4d4d4',
                        border: 'none',
                        outline: 'none',
                        resize: 'none',
                      }
                    }
                  }
                ]
              }
            ]
          }
        });
      },

      // Images - open in viewer
      jpg: async () => await openImage(entry),
      jpeg: async () => await openImage(entry),
      png: async () => await openImage(entry),
      gif: async () => await openImage(entry),
      webp: async () => await openImage(entry),

      // Code files - open with syntax highlighting
      js: async () => await openCode(entry),
      ts: async () => await openCode(entry),
      tsx: async () => await openCode(entry),
      jsx: async () => await openCode(entry),
      py: async () => await openCode(entry),
      go: async () => await openCode(entry),
      rs: async () => await openCode(entry),
      json: async () => await openCode(entry),
      yaml: async () => await openCode(entry),
      yml: async () => await openCode(entry),
    };

    const handler = handlers[ext];
    if (handler) {
      await handler();
    } else {
      // Default: show file info
      const stat = await executor.execute('filesystem.stat', { path: entry.path });
      console.log('File info:', stat);
    }
  };

  const openImage = async (entry: FileEntry) => {
    await executor.execute('app.spawn', {
      request: `View ${entry.name}`,
      blueprint: {
        title: entry.name,
        components: [
          {
            type: 'container',
            props: {
              style: {
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                height: '100%',
                background: '#000',
              }
            },
            children: [
              {
                type: 'img',
                props: {
                  src: `file://${entry.path}`,
                  alt: entry.name,
                  style: {
                    maxWidth: '100%',
                    maxHeight: '100%',
                    objectFit: 'contain',
                  }
                }
              }
            ]
          }
        ]
      }
    });
  };

  const openCode = async (entry: FileEntry) => {
    const content = await executor.execute('filesystem.read', { path: entry.path });
    await executor.execute('app.spawn', {
      request: `Edit ${entry.name}`,
      blueprint: {
        title: entry.name,
        components: [
          {
            type: 'container',
            props: {
              style: {
                padding: '20px',
                height: '100%',
                background: '#1e1e1e',
              }
            },
            children: [
              {
                type: 'pre',
                props: {
                  style: {
                    fontFamily: 'monospace',
                    fontSize: '14px',
                    color: '#d4d4d4',
                    margin: 0,
                    whiteSpace: 'pre-wrap',
                  }
                },
                content: content?.content || '',
              }
            ]
          }
        ]
      }
    });
  };

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="file-explorer-v2" onKeyDown={keyboard.handleKeyDown} tabIndex={0}>
      {/* Top Toolbar */}
      <div className="explorer-toolbar">
        {/* Navigation Controls */}
        <div className="toolbar-nav">
          <button
            className="toolbar-btn"
            onClick={() => fs.goBack()}
            disabled={!fs.canGoBack}
            title="Back"
          >
            ←
          </button>
          <button
            className="toolbar-btn"
            onClick={() => fs.goForward()}
            disabled={!fs.canGoForward}
            title="Forward"
          >
            →
          </button>
          <button
            className="toolbar-btn"
            onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
            title="Toggle Sidebar"
          >
            {sidebarCollapsed ? '☰' : '⋮'}
          </button>
        </div>

        {/* Search Bar */}
        <SearchBar
          query={search.query}
          onQueryChange={search.setQuery}
          onSearch={search.execute}
          filters={search.filters}
          onFiltersChange={search.setFilters}
        />

        {/* View Controls */}
        <div className="toolbar-actions">
          <div className="view-switcher">
            <button
              className={`view-btn ${viewMode === 'columns' ? 'active' : ''}`}
              onClick={() => setViewMode('columns')}
              title="Columns View"
            >
              ▦
            </button>
            <button
              className={`view-btn ${viewMode === 'list' ? 'active' : ''}`}
              onClick={() => setViewMode('list')}
              title="List View"
            >
              ☰
            </button>
            <button
              className={`view-btn ${viewMode === 'grid' ? 'active' : ''}`}
              onClick={() => setViewMode('grid')}
              title="Grid View"
            >
              ▦▦
            </button>
          </div>

          <button
            className="toolbar-btn"
            onClick={() => setShowCommandPalette(true)}
            title="Command Palette (⌘P)"
          >
            ⌘
          </button>
          <button
            className="toolbar-btn"
            onClick={() => setPreviewCollapsed(prev => !prev)}
            title="Toggle Preview (Space)"
          >
            {previewCollapsed ? '👁️' : '👁️‍🗨️'}
          </button>
        </div>
      </div>

      {/* Main Layout */}
      <div className="explorer-main">
        {/* Sidebar */}
        {!sidebarCollapsed && (
          <div className="explorer-sidebar">
            <div className="sidebar-section">
              <div className="sidebar-title">Favorites</div>
              <div className="sidebar-items">
                {preferences.data.favorites.length === 0 ? (
                  <div className="sidebar-empty">No favorites yet</div>
                ) : (
                  preferences.data.favorites.map((path, index) => (
                    <button
                      key={`fav-${index}`}
                      className={`sidebar-item ${fs.currentPath === path ? 'active' : ''}`}
                      onClick={() => fs.navigate(path)}
                    >
                      <span className="sidebar-icon">⭐</span>
                      <span className="sidebar-label">{path.split('/').pop() || path}</span>
                    </button>
                  ))
                )}
              </div>
            </div>

            <div className="sidebar-section">
              <div className="sidebar-title">Quick Access</div>
              <div className="sidebar-items">
                <button
                  className={`sidebar-item ${fs.currentPath === '/storage' ? 'active' : ''}`}
                  onClick={() => fs.navigate('/storage')}
                >
                  <span className="sidebar-icon">💾</span>
                  <span className="sidebar-label">Storage</span>
                </button>
              </div>
            </div>

            <div className="sidebar-section">
              <div className="sidebar-title">Recent</div>
              <div className="sidebar-items">
                {preferences.data.recent.slice(0, 5).length === 0 ? (
                  <div className="sidebar-empty">No recent locations</div>
                ) : (
                  preferences.data.recent.slice(0, 5).map((path, index) => (
                    <button
                      key={`recent-${index}`}
                      className={`sidebar-item ${fs.currentPath === path ? 'active' : ''}`}
                      onClick={() => fs.navigate(path)}
                    >
                      <span className="sidebar-icon">🕐</span>
                      <span className="sidebar-label">{path.split('/').pop() || path}</span>
                    </button>
                  ))
                )}
              </div>
            </div>
          </div>
        )}

        {/* Content Area */}
        <div className="explorer-content">
          {/* Breadcrumb Navigation */}
          <div className="breadcrumb-bar">
            <div className="breadcrumbs">
              {fs.currentPath.split('/').filter(Boolean).map((part, index, arr) => {
                const path = '/' + arr.slice(0, index + 1).join('/');
                return (
                  <div key={path} className="breadcrumb">
                    <button
                      className="breadcrumb-btn"
                      onClick={() => fs.navigate(path)}
                    >
                      {part}
                    </button>
                    {index < arr.length - 1 && <span className="breadcrumb-sep">›</span>}
                  </div>
                );
              })}
              {fs.currentPath === '/' && (
                <div className="breadcrumb">
                  <button className="breadcrumb-btn" onClick={() => fs.navigate('/')}>
                    Root
                  </button>
                </div>
              )}
            </div>

            <div className="breadcrumb-actions">
              <button
                className="toolbar-btn"
                onClick={() => fs.refresh()}
                title="Refresh"
              >
                ↻
              </button>
              <button
                className="toolbar-btn"
                onClick={() => {
                  const name = prompt('New folder name:');
                  if (name) fs.createFolder(name);
                }}
                title="New Folder"
              >
                📁+
              </button>
            </div>
          </div>

          {/* File Display Area */}
          <div className="explorer-body">
            {viewMode === 'columns' ? (
              <div ref={columnsRef} className="columns-container">
                <MillerColumns
                  columns={columns}
                  onSelect={handleColumnSelect}
                  onNavigate={handleColumnNavigate}
                  selectedEntry={selectedEntry}
                  showHidden={preferences.data.showHidden}
                />
              </div>
            ) : (
              <div className="files-container">
                {fs.loading ? (
                  <div className="loading-state">
                    <div className="spinner">↻</div>
                    <div>Loading...</div>
                  </div>
                ) : fs.entries.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-icon">📂</div>
                    <div className="empty-title">This folder is empty</div>
                    <div className="empty-hint">
                      <button
                        className="empty-action"
                        onClick={() => {
                          const name = prompt('New folder name:');
                          if (name) fs.createFolder(name);
                        }}
                      >
                        Create a folder
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className={`files-${viewMode}`}>
                    {fs.entries
                      .filter(entry => preferences.data.showHidden || !entry.name.startsWith('.'))
                      .map((entry) => {
                        const icon = getFileIcon(entry);
                        const isSelected = selectedEntry?.path === entry.path;

                        return (
                          <div
                            key={entry.path}
                            className={`file-item ${isSelected ? 'selected' : ''} ${entry.is_dir ? 'directory' : ''}`}
                            onClick={() => setSelectedEntry(entry)}
                            onDoubleClick={() => {
                              if (entry.is_dir) {
                                fs.navigate(entry.path);
                              } else {
                                openFile(entry);
                              }
                            }}
                          >
                            <div className="file-item-icon" style={{ color: icon.color }}>
                              {icon.emoji}
                            </div>
                            <div className="file-item-name">{entry.name}</div>
                            {viewMode === 'list' && (
                              <>
                                <div className="file-item-size">
                                  {entry.is_dir ? '—' : formatFileSize(entry.size)}
                                </div>
                                <div className="file-item-modified">
                                  {formatDate(entry.modified)}
                                </div>
                              </>
                            )}
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>
            )}

            {/* Preview Panel */}
            {!previewCollapsed && selectedEntry && viewMode !== 'columns' && (
              <PreviewPanel
                entry={selectedEntry}
                context={context}
                onOpen={() => openFile(selectedEntry)}
              />
            )}
          </div>
        </div>
      </div>

      {/* Status Bar */}
      <StatusBar
        currentPath={fs.currentPath}
        itemCount={fs.entries.length}
        selectedEntry={selectedEntry}
        loading={fs.loading}
      />

      {/* Command Palette */}
      {showCommandPalette && (
        <CommandPalette
          onCommand={handleCommand}
          onClose={() => setShowCommandPalette(false)}
          recentPaths={preferences.data.recent}
          favorites={preferences.data.favorites}
        />
      )}

      {/* Quick Access */}
      {showQuickAccess && (
        <QuickAccess
          favorites={preferences.data.favorites}
          recent={preferences.data.recent}
          onSelect={(path) => {
            fs.navigate(path);
            setShowQuickAccess(false);
          }}
          onClose={() => setShowQuickAccess(false)}
        />
      )}
    </div>
  );
}
