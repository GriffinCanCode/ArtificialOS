/**
 * File Explorer App
 * Main component orchestrating file management
 */

import { useEffect, useState, useCallback } from 'react';
import type { NativeAppProps } from './sdk';
import type { ViewMode, SortConfig, ContextAction, FileEntry } from './types';
import { useFileSystem } from './hooks/useFileSystem';
import { useSelection } from './hooks/useSelection';
import { useKeyboard } from './hooks/useKeyboard';
import { useClipboard } from './hooks/useClipboard';
import { sortEntries, filterEntries } from './utils';
import { Toolbar } from './components/Toolbar';
import { PathBar } from './components/PathBar';
import { FileList } from './components/FileList';
import { ContextMenu } from './components/ContextMenu';
import { InputDialog } from './components/InputDialog';
import { ConfirmDialog } from './components/ConfirmDialog';
import './styles/App.css';

export default function FileExplorerApp({ context }: NativeAppProps) {
  const { window: win } = context;

  // File system management
  const fs = useFileSystem(context);

  // View state
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [sortConfig, setSortConfig] = useState<SortConfig>({
    field: 'name',
    direction: 'asc',
  });
  const [searchQuery, setSearchQuery] = useState('');

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    entry: FileEntry | null;
  } | null>(null);

  // Dialog states
  const [inputDialog, setInputDialog] = useState<{
    title: string;
    label: string;
    defaultValue?: string;
    onConfirm: (value: string) => void;
  } | null>(null);

  const [confirmDialog, setConfirmDialog] = useState<{
    title: string;
    message: string;
    variant?: 'default' | 'danger';
    onConfirm: () => void;
  } | null>(null);

  // Process entries (sort & filter)
  const processedEntries = sortEntries(
    filterEntries(fs.entries, searchQuery),
    sortConfig
  );

  // Selection management
  const selection = useSelection(processedEntries);

  // Clipboard management
  const clipboard = useClipboard(fs.copyEntry, fs.moveEntry);

  // ============================================================================
  // Lifecycle
  // ============================================================================

  useEffect(() => {
    // Load initial directory
    fs.navigate(fs.currentPath);
    win.setTitle('File Explorer');
  }, []);

  useEffect(() => {
    // Update window title with current path
    const dirname = fs.currentPath.split('/').pop() || 'File Explorer';
    win.setTitle(`📁 ${dirname}`);
  }, [fs.currentPath, win]);

  // ============================================================================
  // Handlers
  // ============================================================================

  /**
   * Open file in appropriate viewer based on file type
   */
  const openFile = async (entry: FileEntry) => {
    const ext = entry.name.split('.').pop()?.toLowerCase() || '';

    // Categorize file types
    const textExtensions = ['txt', 'md', 'json', 'xml', 'html', 'css', 'js', 'ts', 'tsx', 'jsx', 'py', 'go', 'rs', 'c', 'cpp', 'h', 'java', 'sh', 'yaml', 'yml', 'toml', 'ini', 'cfg', 'conf', 'log'];
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'ico'];
    const videoExtensions = ['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv'];
    const audioExtensions = ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'];

    try {
      if (textExtensions.includes(ext)) {
        // Read and display text files
        const content = await context.executor.execute('filesystem.read', { path: entry.path });

        // Create a simple text viewer by spawning a new app
        await context.executor.execute('app.spawn', {
          request: `Text Viewer - ${entry.name}`,
          blueprint: {
            title: entry.name,
            components: [
              {
                type: 'container',
                props: {
                  style: {
                    padding: '20px',
                    height: '100%',
                    overflow: 'auto',
                    backgroundColor: '#1e1e1e',
                  },
                },
                children: [
                  {
                    type: 'pre',
                    props: {
                      style: {
                        whiteSpace: 'pre-wrap',
                        wordWrap: 'break-word',
                        fontFamily: 'monospace',
                        fontSize: '14px',
                        color: '#d4d4d4',
                        margin: 0,
                      },
                    },
                    content: typeof content === 'string' ? content : content?.data || 'Unable to read file',
                  },
                ],
              },
            ],
          },
        });
      } else if (imageExtensions.includes(ext)) {
        // Display image files
        await context.executor.execute('app.spawn', {
          request: `Image Viewer - ${entry.name}`,
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
                    backgroundColor: '#000',
                  },
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
                      },
                    },
                  },
                ],
              },
            ],
          },
        });
      } else if (videoExtensions.includes(ext)) {
        // Display video files
        await context.executor.execute('app.spawn', {
          request: `Video Player - ${entry.name}`,
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
                    backgroundColor: '#000',
                  },
                },
                children: [
                  {
                    type: 'video',
                    props: {
                      src: `file://${entry.path}`,
                      controls: true,
                      style: {
                        maxWidth: '100%',
                        maxHeight: '100%',
                      },
                    },
                  },
                ],
              },
            ],
          },
        });
      } else if (audioExtensions.includes(ext)) {
        // Display audio files
        await context.executor.execute('app.spawn', {
          request: `Audio Player - ${entry.name}`,
          blueprint: {
            title: entry.name,
            components: [
              {
                type: 'container',
                props: {
                  style: {
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'center',
                    alignItems: 'center',
                    height: '100%',
                    gap: '20px',
                  },
                },
                children: [
                  {
                    type: 'text',
                    props: {
                      style: {
                        fontSize: '18px',
                        fontWeight: 'bold',
                      },
                    },
                    content: entry.name,
                  },
                  {
                    type: 'audio',
                    props: {
                      src: `file://${entry.path}`,
                      controls: true,
                      autoPlay: true,
                    },
                  },
                ],
              },
            ],
          },
        });
      } else {
        // For unknown file types, show file info and allow download/copy path
        const stats = await context.executor.execute('filesystem.stat', { path: entry.path });

        await context.executor.execute('app.spawn', {
          request: `File Info - ${entry.name}`,
          blueprint: {
            title: entry.name,
            components: [
              {
                type: 'container',
                props: {
                  style: {
                    padding: '20px',
                  },
                },
                children: [
                  {
                    type: 'text',
                    props: {
                      style: {
                        fontSize: '18px',
                        fontWeight: 'bold',
                        marginBottom: '20px',
                      },
                    },
                    content: entry.name,
                  },
                  {
                    type: 'text',
                    content: `Path: ${entry.path}`,
                  },
                  {
                    type: 'text',
                    content: `Size: ${entry.size} bytes`,
                  },
                  {
                    type: 'text',
                    content: `Type: ${ext || 'unknown'}`,
                  },
                  {
                    type: 'text',
                    content: `Modified: ${entry.modified ? new Date(entry.modified).toLocaleString() : 'N/A'}`,
                  },
                ],
              },
            ],
          },
        });
      }
    } catch (err) {
      alert(`Failed to open file: ${(err as Error).message}`);
    }
  };

  /**
   * Open entry (navigate to folder or open file)
   */
  const handleEntryOpen = useCallback(
    async (entry: FileEntry) => {
      if (entry.is_dir) {
        await fs.navigate(entry.path);
        selection.clearSelection();
      } else {
        // Open file in appropriate viewer based on file type
        await openFile(entry);
      }
    },
    [fs, selection, context]
  );

  /**
   * Handle entry click
   */
  const handleEntryClick = useCallback(
    (entry: FileEntry, event: React.MouseEvent) => {
      if (event.shiftKey && selection.selected.size > 0) {
        // Shift+click: range select
        const lastSelected = Array.from(selection.selected)[selection.selected.size - 1];
        selection.selectRange(lastSelected, entry.path);
      } else {
        selection.toggle(entry.path, event);
      }
    },
    [selection]
  );

  /**
   * Handle entry double-click
   */
  const handleEntryDoubleClick = useCallback(
    (entry: FileEntry) => {
      handleEntryOpen(entry);
    },
    [handleEntryOpen]
  );

  /**
   * Handle entry context menu
   */
  const handleEntryContextMenu = useCallback(
    (entry: FileEntry, event: React.MouseEvent) => {
      event.preventDefault();

      // Select if not already selected
      if (!selection.isSelected(entry.path)) {
        selection.toggle(entry.path);
      }

      setContextMenu({
        x: event.clientX,
        y: event.clientY,
        entry,
      });
    },
    [selection]
  );

  /**
   * Handle sort change
   */
  const handleSortChange = useCallback((field: string) => {
    setSortConfig((prev) => ({
      field: field as SortConfig['field'],
      direction: prev.field === field && prev.direction === 'asc' ? 'desc' : 'asc',
    }));
  }, []);

  /**
   * Handle new folder
   */
  const handleNewFolder = useCallback(async () => {
    setInputDialog({
      title: 'New Folder',
      label: 'Folder name:',
      onConfirm: async (name: string) => {
        setInputDialog(null);
        try {
          await fs.createFolder(name);
        } catch (err) {
          alert((err as Error).message);
        }
      },
    });
  }, [fs]);

  /**
   * Handle copy
   */
  const handleCopy = useCallback(() => {
    const paths = Array.from(selection.selected);
    if (paths.length > 0) {
      clipboard.copy(paths);
    }
  }, [selection, clipboard]);

  /**
   * Handle cut
   */
  const handleCut = useCallback(() => {
    const paths = Array.from(selection.selected);
    if (paths.length > 0) {
      clipboard.cut(paths);
    }
  }, [selection, clipboard]);

  /**
   * Handle delete
   */
  const handleDelete = useCallback(async () => {
    const paths = Array.from(selection.selected);
    if (paths.length === 0) return;

    setConfirmDialog({
      title: 'Delete Items',
      message: `Delete ${paths.length} item${paths.length > 1 ? 's' : ''}?`,
      variant: 'danger',
      onConfirm: async () => {
        setConfirmDialog(null);
        try {
          for (const path of paths) {
            await fs.deleteEntry(path);
          }
          selection.clearSelection();
        } catch (err) {
          alert((err as Error).message);
        }
      },
    });
  }, [selection, fs]);

  /**
   * Handle paste
   */
  const handlePaste = useCallback(async () => {
    if (clipboard.canPaste) {
      try {
        await clipboard.paste(fs.currentPath);
      } catch (err) {
        alert((err as Error).message);
      }
    }
  }, [clipboard, fs]);

  // Keyboard navigation (moved here after handlers are defined)
  const keyboard = useKeyboard(
    processedEntries.length,
    (index) => handleEntryOpen(processedEntries[index]),
    handleDelete,
    selection.selectAll,
    handleCopy,
    handleCut,
    handlePaste
  );

  /**
   * Handle context menu action
   */
  const handleContextAction = useCallback(
    async (action: ContextAction) => {
      switch (action) {
        case 'open':
          if (contextMenu?.entry) {
            await handleEntryOpen(contextMenu.entry);
          }
          break;

        case 'copy':
          handleCopy();
          break;

        case 'cut':
          handleCut();
          break;

        case 'paste':
          await handlePaste();
          break;

        case 'delete':
          await handleDelete();
          break;

        case 'rename':
          if (contextMenu?.entry) {
            const entryToRename = contextMenu.entry;
            setInputDialog({
              title: 'Rename',
              label: 'New name:',
              defaultValue: entryToRename.name,
              onConfirm: async (newName: string) => {
                setInputDialog(null);
                try {
                  await fs.renameEntry(entryToRename.path, newName);
                } catch (err) {
                  alert((err as Error).message);
                }
              },
            });
          }
          break;

        case 'new-folder':
          await handleNewFolder();
          break;

        case 'refresh':
          await fs.refresh();
          break;

        case 'properties':
          if (contextMenu?.entry) {
            alert(`Properties:\n${JSON.stringify(contextMenu.entry, null, 2)}`);
          }
          break;
      }
    },
    [contextMenu, handleEntryOpen, handleCopy, handleCut, handlePaste, handleDelete, handleNewFolder, fs]
  );

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="file-explorer">
      <Toolbar
        viewMode={viewMode}
        sortField={sortConfig.field}
        sortDirection={sortConfig.direction}
        searchQuery={searchQuery}
        canGoBack={fs.canGoBack}
        canGoForward={fs.canGoForward}
        onGoBack={fs.goBack}
        onGoForward={fs.goForward}
        onGoUp={fs.goUp}
        onRefresh={fs.refresh}
        onNewFolder={handleNewFolder}
        onViewModeChange={setViewMode}
        onSortChange={handleSortChange}
        onSearchChange={setSearchQuery}
      />

      <PathBar currentPath={fs.currentPath} onNavigate={fs.navigate} />

      <div className="file-explorer-content">
        {fs.loading ? (
          <div className="file-explorer-loading">
            <div className="loading-spinner">↻</div>
            <div className="loading-text">Loading...</div>
          </div>
        ) : fs.error ? (
          <div className="file-explorer-error">
            <div className="error-icon">⚠️</div>
            <div className="error-text">{fs.error}</div>
            <button className="error-button" onClick={fs.refresh}>
              Retry
            </button>
          </div>
        ) : (
          <FileList
            entries={processedEntries}
            viewMode={viewMode}
            selectedPaths={selection.selected}
            onEntryClick={handleEntryClick}
            onEntryDoubleClick={handleEntryDoubleClick}
            onEntryContextMenu={handleEntryContextMenu}
            onKeyDown={keyboard.handleKeyDown}
          />
        )}
      </div>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          entry={contextMenu.entry}
          selectedCount={selection.selected.size}
          onClose={() => setContextMenu(null)}
          onAction={handleContextAction}
        />
      )}

      {/* Status bar */}
      <div className="file-explorer-status">
        <span className="status-text">
          {processedEntries.length} item{processedEntries.length !== 1 ? 's' : ''}
          {selection.selected.size > 0 && ` • ${selection.selected.size} selected`}
          {clipboard.canPaste && ` • ${clipboard.clipboard.paths.length} in clipboard`}
        </span>
      </div>

      {/* Input dialog */}
      {inputDialog && (
        <InputDialog
          title={inputDialog.title}
          label={inputDialog.label}
          defaultValue={inputDialog.defaultValue}
          onConfirm={inputDialog.onConfirm}
          onCancel={() => setInputDialog(null)}
        />
      )}

      {/* Confirm dialog */}
      {confirmDialog && (
        <ConfirmDialog
          title={confirmDialog.title}
          message={confirmDialog.message}
          variant={confirmDialog.variant}
          confirmText="Delete"
          onConfirm={confirmDialog.onConfirm}
          onCancel={() => setConfirmDialog(null)}
        />
      )}
    </div>
  );
}
