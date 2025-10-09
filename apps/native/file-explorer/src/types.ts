/**
 * File Explorer Types
 * Enhanced types for the revolutionary file explorer
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * File system entry (file or directory)
 */
export interface FileEntry {
  name: string;
  path: string;
  size: number;
  modified: string;
  is_dir: boolean;
  mime_type?: string;
  extension?: string;
  permissions?: string;
  tags?: string[];
  color?: string;
}

/**
 * Miller Column
 */
export interface Column {
  id: string;
  path: string;
  name: string;
  entries: FileEntry[];
  selectedIndex: number;
  loading?: boolean;
}

/**
 * Search filters
 */
export interface SearchFilters {
  type?: 'all' | 'files' | 'folders' | 'images' | 'documents' | 'code';
  minSize?: number;
  maxSize?: number;
  modifiedAfter?: string;
  modifiedBefore?: string;
  extensions?: string[];
  tags?: string[];
}

/**
 * User preferences
 */
export interface Preferences {
  showHidden: boolean;
  lastPath: string;
  favorites: string[];
  recent: string[];
  tags: { [path: string]: string[] };
  colors: { [path: string]: string };
}

/**
 * Command for palette
 */
export interface Command {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  shortcut?: string;
  category?: 'navigation' | 'file' | 'view' | 'search' | 'system';
  handler: (payload?: any) => void | Promise<void>;
}

// ============================================================================
// Navigation History
// ============================================================================

export interface HistoryEntry {
  path: string;
  timestamp: number;
}

// ============================================================================
// Clipboard
// ============================================================================

export type ClipboardOperation = 'copy' | 'cut' | null;

export interface ClipboardState {
  operation: ClipboardOperation;
  paths: string[];
}

// ============================================================================
// Hook Return Types
// ============================================================================

export interface UseFileSystemReturn {
  entries: FileEntry[];
  currentPath: string;
  loading: boolean;
  error: string | null;
  navigate: (path: string) => Promise<void>;
  refresh: () => Promise<void>;
  goUp: () => Promise<void>;
  goBack: () => Promise<void>;
  goForward: () => Promise<void>;
  canGoBack: boolean;
  canGoForward: boolean;
  createFolder: (name: string) => Promise<void>;
  deleteEntry: (path: string) => Promise<void>;
  renameEntry: (oldPath: string, newName: string) => Promise<void>;
  copyEntry: (source: string, dest: string) => Promise<void>;
  moveEntry: (source: string, dest: string) => Promise<void>;
}

export interface UsePreferencesReturn {
  data: Preferences;
  toggleShowHidden: () => void;
  addFavorite: (path: string) => void;
  removeFavorite: (path: string) => void;
  addRecent: (path: string) => void;
  setTag: (path: string, tags: string[]) => void;
  setColor: (path: string, color: string) => void;
}

export interface UseSearchReturn {
  query: string;
  setQuery: (query: string) => void;
  filters: SearchFilters;
  setFilters: (filters: SearchFilters) => void;
  results: FileEntry[];
  loading: boolean;
  execute: () => Promise<void>;
  clear: () => void;
}

// ============================================================================
// Utility Types
// ============================================================================

export type FileCategory =
  | 'folder'
  | 'document'
  | 'image'
  | 'video'
  | 'audio'
  | 'archive'
  | 'code'
  | 'unknown';

export interface FileIcon {
  emoji: string;
  color: string;
}

export interface PathSegment {
  name: string;
  path: string;
}

// ============================================================================
// Old Types (for backward compatibility)
// ============================================================================

export type ViewMode = 'list' | 'grid' | 'compact';
export type SortField = 'name' | 'size' | 'modified' | 'type';
export type SortDirection = 'asc' | 'desc';

export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

export interface FileItemProps {
  entry: FileEntry;
  isSelected: boolean;
  viewMode: ViewMode;
  onClick: (entry: FileEntry, event: React.MouseEvent) => void;
  onDoubleClick: (entry: FileEntry) => void;
  onContextMenu: (entry: FileEntry, event: React.MouseEvent) => void;
}

export interface ContextMenuProps {
  x: number;
  y: number;
  entry: FileEntry | null;
  selectedCount: number;
  onClose: () => void;
  onAction: (action: ContextAction) => void;
}

export type ContextAction =
  | 'open'
  | 'copy'
  | 'cut'
  | 'paste'
  | 'delete'
  | 'rename'
  | 'properties'
  | 'new-folder'
  | 'refresh';

export interface UseSelectionReturn {
  selected: Set<string>;
  isSelected: (path: string) => boolean;
  toggle: (path: string, event?: React.MouseEvent) => void;
  selectRange: (start: string, end: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  getSelectedEntries: () => FileEntry[];
}

export interface UseClipboardReturn {
  clipboard: ClipboardState;
  copy: (paths: string[]) => void;
  cut: (paths: string[]) => void;
  paste: (targetPath: string) => Promise<void>;
  canPaste: boolean;
  clear: () => void;
}
