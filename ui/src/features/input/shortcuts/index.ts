/**
 * Shortcuts Feature
 * Centralized keyboard shortcut management system
 */

// ============================================================================
// Core
// ============================================================================

export * from "./core/types";
export * from "./core/platform";
export * from "./core/parser";
export * from "./core/formatter";
export * from "./core/conflict";
export { registry, ShortcutRegistry } from "./core/registry";

// ============================================================================
// Store (internal - access via hooks through main input module)
// ============================================================================

export {
  useStore,
  useActions,
  useShortcutsByScope,
  useShortcutsByCategory,
  useActiveScopes,
  useConflicts,
  useStats,
  useEnabled,
} from "./store/store";

// ============================================================================
// Commands
// ============================================================================

export * from "./commands";

