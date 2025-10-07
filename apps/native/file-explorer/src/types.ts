/**
 * File Explorer Types
 * Comprehensive TypeScript definitions
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
}

/**
 * View mode for file list
 */
export type ViewMode = 'list' | 'grid' | 'compact';

/**
 * Sort field
 */
export type SortField = 'name' | 'size' | 'modified' | 'type';

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * Sort configuration
 */
export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

/**
 * Selection mode
 */
export type SelectionMode = 'single' | 'multiple';

/**
 * Navigation history entry
 */
export interface HistoryEntry {
  path: string;
  timestamp: number;
}

/**
 * Clipboard operation
 */
export type ClipboardOperation = 'copy' | 'cut' | null;

/**
 * Clipboard state
 */
export interface ClipboardState {
  operation: ClipboardOperation;
  paths: string[];
}

// ============================================================================
// Component Props
// ============================================================================

/**
 * File list item props
 */
export interface FileItemProps {
  entry: FileEntry;
  isSelected: boolean;
  viewMode: ViewMode;
  onClick: (entry: FileEntry, event: React.MouseEvent) => void;
  onDoubleClick: (entry: FileEntry) => void;
  onContextMenu: (entry: FileEntry, event: React.MouseEvent) => void;
}

/**
 * Context menu props
 */
export interface ContextMenuProps {
  x: number;
  y: number;
  entry: FileEntry | null;
  selectedCount: number;
  onClose: () => void;
  onAction: (action: ContextAction) => void;
}

/**
 * Context menu action
 */
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

/**
 * Path breadcrumb segment
 */
export interface PathSegment {
  name: string;
  path: string;
}

// ============================================================================
// Hook Return Types
// ============================================================================

/**
 * File system hook return
 */
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

/**
 * Selection hook return
 */
export interface UseSelectionReturn {
  selected: Set<string>;
  isSelected: (path: string) => boolean;
  toggle: (path: string, event?: React.MouseEvent) => void;
  selectRange: (start: string, end: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  getSelectedEntries: () => FileEntry[];
}

/**
 * Keyboard navigation hook return
 */
export interface UseKeyboardReturn {
  focusedIndex: number;
  handleKeyDown: (event: React.KeyboardEvent) => void;
}

/**
 * Clipboard hook return
 */
export interface UseClipboardReturn {
  clipboard: ClipboardState;
  copy: (paths: string[]) => void;
  cut: (paths: string[]) => void;
  paste: (targetPath: string) => Promise<void>;
  canPaste: boolean;
  clear: () => void;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * File type category
 */
export type FileCategory =
  | 'folder'
  | 'document'
  | 'image'
  | 'video'
  | 'audio'
  | 'archive'
  | 'code'
  | 'unknown';

/**
 * File icon mapping
 */
export interface FileIcon {
  emoji: string;
  color: string;
}
