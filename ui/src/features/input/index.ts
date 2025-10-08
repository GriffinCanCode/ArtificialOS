/**
 * Input Handling Module
 * Centralized input handling for keyboard, mouse, touch, and gestures
 */

// Core
export * from "./core/types";
export * from "./core/keyboard";
export * from "./core/mouse";
export * from "./core/gesture";

// Hooks
export * from "./hooks/useKeyboard";
export * from "./hooks/useMouse";
export * from "./hooks/useGesture";
export * from "./hooks/useValidation";

// Shortcut hooks
export {
  useShortcut,
  useSimpleShortcut,
  useGlobalShortcut,
  type UseShortcutOptions,
} from "./hooks/useShortcut";

export {
  useShortcuts,
  useShortcutMap,
  useScopedShortcuts
} from "./hooks/useShortcuts";

export {
  useScope,
  useScopes,
  useScopeActive,
  useActiveScopesArray,
  useToggleScope,
} from "./hooks/useScope";

// Validation
export * from "./validation/schemas";
export * from "./validation/validators";

// Formatting
export * from "./formatting/text";
export * from "./formatting/number";
export * from "./formatting/dates";

// Shortcuts (Centralized Shortcut Management)
// Re-export with explicit naming to avoid conflicts
export {
  // Core types and functions
  type ShortcutConfig as ShortcutDefinition,
  type ShortcutScope,
  type ShortcutPriority,
  type ShortcutCategory,
  type Platform,
  type RegisteredShortcut,
  type ShortcutConflict,
  type ShortcutStats,
  type FormattedShortcut,
  type Command,

  // Platform functions
  detectPlatform,
  isMac,
  isWindows,
  isLinux,

  // Formatting
  formatShortcut,
  formatShortcuts,
  formatShortcutHTML,
  formatShortcutAria,

  // Parser
  parseSequence,
  validateSequence,
  normalizeSequence,
  matchesSequence,

  // Registry
  registry,
  ShortcutRegistry,

  // Store hooks
  useStore as useShortcutStore,
  useActions as useShortcutActions,
  useShortcutsByScope,
  useShortcutsByCategory,
  useActiveScopes,
  useConflicts,
  useStats as useShortcutStats,
  useEnabled,


  // Commands
  allCommands,
  systemCommands,
  windowCommands,
  selectionCommands,
  clipboardCommands,
  getCommandsByCategory,
  getCommandById,
  createCommand,
} from "./shortcuts";
