/**
 * Shortcut Management Types
 * Type definitions for centralized keyboard shortcut system
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Keyboard modifier keys
 */
export type Modifier = "$mod" | "Control" | "Meta" | "Alt" | "Shift";

/**
 * Shortcut scope determines where the shortcut is active
 */
export type ShortcutScope =
  | "global"        // Active everywhere
  | "window"        // Active in focused window
  | "desktop"       // Active on desktop only
  | "creator"       // Active in creator mode
  | "hub"           // Active in hub
  | "terminal"      // Active in terminal
  | "app";          // Active in specific app

/**
 * Shortcut priority for conflict resolution
 */
export type ShortcutPriority = "low" | "normal" | "high" | "critical";

/**
 * Shortcut category for organization
 */
export type ShortcutCategory =
  | "system"        // System-level shortcuts
  | "window"        // Window management
  | "navigation"    // Navigation shortcuts
  | "editing"       // Text editing
  | "selection"     // Selection management
  | "clipboard"     // Clipboard operations
  | "app"           // App-specific
  | "developer"     // Developer tools
  | "custom";       // User-defined

/**
 * Platform types
 */
export type Platform = "mac" | "windows" | "linux" | "unknown";

// ============================================================================
// Configuration Types
// ============================================================================

/**
 * Shortcut configuration
 */
export interface ShortcutConfig {
  /** Unique identifier */
  id: string;

  /** Keyboard sequence (e.g., "$mod+k", "Control+Shift+p") */
  sequence: string;

  /** Human-readable label */
  label: string;

  /** Detailed description */
  description?: string;

  /** Handler function */
  handler: ShortcutHandler;

  /** Scope where shortcut is active */
  scope?: ShortcutScope;

  /** Priority for conflict resolution */
  priority?: ShortcutPriority;

  /** Category for organization */
  category?: ShortcutCategory;

  /** Whether shortcut can be customized */
  customizable?: boolean;

  /** Whether shortcut works in input fields */
  allowInInput?: boolean;

  /** Platform-specific overrides */
  platformOverrides?: Partial<Record<Platform, string>>;

  /** Whether shortcut is enabled */
  enabled?: boolean;

  /** Tags for searching */
  tags?: string[];

  /** Icon for display */
  icon?: string;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Shortcut handler function
 */
export type ShortcutHandler = (event: KeyboardEvent, context?: ShortcutContext) => void | boolean | Promise<void>;

/**
 * Context passed to shortcut handlers
 */
export interface ShortcutContext {
  /** Shortcut configuration */
  config: ShortcutConfig;

  /** Current scope */
  scope: ShortcutScope;

  /** Active scopes */
  activeScopes: Set<ShortcutScope>;

  /** Platform */
  platform: Platform;

  /** Additional context data */
  data?: Record<string, any>;
}

// ============================================================================
// Registry Types
// ============================================================================

/**
 * Registered shortcut with metadata
 */
export interface RegisteredShortcut {
  /** Configuration */
  config: ShortcutConfig;

  /** Registration timestamp */
  registeredAt: number;

  /** Last triggered timestamp */
  lastTriggered?: number;

  /** Trigger count */
  triggerCount: number;

  /** Whether shortcut is currently active */
  isActive: boolean;

  /** Cleanup function */
  cleanup?: () => void;
}

/**
 * Shortcut conflict
 */
export interface ShortcutConflict {
  /** Conflicting sequence */
  sequence: string;

  /** Conflicting shortcuts */
  shortcuts: ShortcutConfig[];

  /** Severity level */
  severity: "warning" | "error";

  /** Resolution strategy */
  resolution?: "priority" | "scope" | "manual";
}

/**
 * Shortcut statistics
 */
export interface ShortcutStats {
  /** Total registered shortcuts */
  total: number;

  /** Active shortcuts */
  active: number;

  /** Shortcuts by scope */
  byScope: Record<ShortcutScope, number>;

  /** Shortcuts by category */
  byCategory: Record<ShortcutCategory, number>;

  /** Most used shortcuts */
  mostUsed: Array<{ id: string; count: number }>;

  /** Detected conflicts */
  conflicts: ShortcutConflict[];
}

// ============================================================================
// Store Types
// ============================================================================

/**
 * Shortcut store state
 */
export interface ShortcutStore {
  /** Registered shortcuts */
  shortcuts: Map<string, RegisteredShortcut>;

  /** Active scopes */
  activeScopes: Set<ShortcutScope>;

  /** Current platform */
  platform: Platform;

  /** Whether shortcuts are globally enabled */
  enabled: boolean;

  /** User customizations */
  customizations: Map<string, string>; // id -> custom sequence

  /** Statistics */
  stats: ShortcutStats;

  // Actions
  register: (config: ShortcutConfig) => string;
  unregister: (id: string) => void;
  enable: (id: string) => void;
  disable: (id: string) => void;
  customize: (id: string, sequence: string) => void;
  resetCustomization: (id: string) => void;
  setScope: (scope: ShortcutScope, active: boolean) => void;
  getByScope: (scope: ShortcutScope) => RegisteredShortcut[];
  getByCategory: (category: ShortcutCategory) => RegisteredShortcut[];
  findConflicts: () => ShortcutConflict[];
  getStats: () => ShortcutStats;
  reset: () => void;
}

// ============================================================================
// Display Types
// ============================================================================

/**
 * Formatted shortcut display
 */
export interface FormattedShortcut {
  /** Original sequence */
  sequence: string;

  /** Platform-specific display string */
  display: string;

  /** Individual keys */
  keys: string[];

  /** Symbol representation (e.g., "⌘K") */
  symbols: string;

  /** Verbose representation (e.g., "Command+K") */
  verbose: string;
}

/**
 * Shortcut group for display
 */
export interface ShortcutGroup {
  /** Group name */
  name: string;

  /** Description */
  description?: string;

  /** Shortcuts in group */
  shortcuts: ShortcutConfig[];

  /** Category */
  category?: ShortcutCategory;

  /** Icon */
  icon?: string;
}

// ============================================================================
// Command Types (for integration with command palette)
// ============================================================================

/**
 * Command definition
 */
export interface Command {
  /** Unique identifier */
  id: string;

  /** Command name */
  name: string;

  /** Description */
  description?: string;

  /** Default shortcut */
  shortcut?: string;

  /** Handler */
  handler: CommandHandler;

  /** Category */
  category?: ShortcutCategory;

  /** Icon */
  icon?: string;

  /** Whether command is available */
  available?: () => boolean;

  /** Tags for searching */
  tags?: string[];
}

/**
 * Command handler function
 */
export type CommandHandler = (args?: any) => void | Promise<void>;

/**
 * Command palette item
 */
export interface CommandPaletteItem {
  /** Command */
  command: Command;

  /** Associated shortcut */
  shortcut?: FormattedShortcut;

  /** Relevance score (for search) */
  score?: number;
}

