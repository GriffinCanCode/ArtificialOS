/**
 * Shortcut Conflict Detection
 * Detect and resolve keyboard shortcut conflicts
 */

import { normalizeSequence, parseSequence } from "./parser";
import type {
  ShortcutConfig,
  ShortcutConflict,
  ShortcutScope,
  RegisteredShortcut,
  Platform
} from "./types";

// ============================================================================
// Conflict Detection
// ============================================================================

/**
 * Find conflicts between shortcuts
 */
export function findConflicts(
  shortcuts: RegisteredShortcut[],
  platform?: Platform
): ShortcutConflict[] {
  const conflicts: ShortcutConflict[] = [];
  const sequenceMap = new Map<string, ShortcutConfig[]>();

  // Group shortcuts by normalized sequence and overlapping scopes
  for (const registered of shortcuts) {
    if (!registered.config.enabled) continue;

    const normalized = normalizeSequence(registered.config.sequence, platform);
    const scope = registered.config.scope || "global";

    // Check if this sequence already exists in same or overlapping scope
    const existing = sequenceMap.get(normalized) || [];

    for (const existingConfig of existing) {
      const existingScope = existingConfig.scope || "global";

      // Check if scopes conflict
      if (scopesOverlap(scope, existingScope)) {
        // Found a conflict - check if already recorded
        let conflict = conflicts.find((c) => c.sequence === normalized);

        if (!conflict) {
          conflict = {
            sequence: normalized,
            shortcuts: [existingConfig],
            severity: "warning",
          };
          conflicts.push(conflict);
        }

        if (!conflict.shortcuts.includes(registered.config)) {
          conflict.shortcuts.push(registered.config);
        }
      }
    }

    existing.push(registered.config);
    sequenceMap.set(normalized, existing);
  }

  // Determine severity and resolution strategy
  for (const conflict of conflicts) {
    conflict.severity = determineSeverity(conflict);
    conflict.resolution = determineResolution(conflict);
  }

  return conflicts;
}

/**
 * Check if two scopes overlap
 */
export function scopesOverlap(scope1: ShortcutScope, scope2: ShortcutScope): boolean {
  // Global scope conflicts with everything
  if (scope1 === "global" || scope2 === "global") {
    return true;
  }

  // Same scope always conflicts
  if (scope1 === scope2) {
    return true;
  }

  // App and window scopes can coexist
  if ((scope1 === "app" && scope2 === "window") ||
      (scope1 === "window" && scope2 === "app")) {
    return false;
  }

  return false;
}

/**
 * Determine conflict severity
 */
function determineSeverity(
  conflict: ShortcutConflict
): "warning" | "error" {
  // Error if shortcuts have same priority
  const priorities = conflict.shortcuts.map((s) => s.priority || "normal");
  const uniquePriorities = new Set(priorities);

  if (uniquePriorities.size === 1) {
    return "error";
  }

  // Warning if different priorities (can be resolved)
  return "warning";
}

/**
 * Determine resolution strategy
 */
function determineResolution(
  conflict: ShortcutConflict
): "priority" | "scope" | "manual" {
  const shortcuts = conflict.shortcuts;

  // Check if can be resolved by priority
  const priorities = shortcuts.map((s) => s.priority || "normal");
  const uniquePriorities = new Set(priorities);

  if (uniquePriorities.size > 1) {
    return "priority";
  }

  // Check if can be resolved by scope
  const scopes = shortcuts.map((s) => s.scope || "global");
  const hasGlobal = scopes.includes("global");

  if (!hasGlobal) {
    return "scope";
  }

  return "manual";
}

// ============================================================================
// Resolution
// ============================================================================

/**
 * Resolve conflict based on priority
 */
export function resolveByPriority(
  shortcuts: ShortcutConfig[]
): ShortcutConfig | null {
  const priorityOrder = { critical: 4, high: 3, normal: 2, low: 1 };

  let winner: ShortcutConfig | null = null;
  let maxPriority = 0;

  for (const shortcut of shortcuts) {
    const priority = priorityOrder[shortcut.priority || "normal"];
    if (priority > maxPriority) {
      maxPriority = priority;
      winner = shortcut;
    }
  }

  return winner;
}

/**
 * Resolve conflict based on scope
 */
export function resolveByScope(
  shortcuts: ShortcutConfig[],
  activeScopes: Set<ShortcutScope>
): ShortcutConfig | null {
  // Priority order: most specific to least specific
  const scopePriority: ShortcutScope[] = [
    "app",
    "terminal",
    "hub",
    "creator",
    "window",
    "desktop",
    "global",
  ];

  for (const scope of scopePriority) {
    if (!activeScopes.has(scope)) continue;

    const match = shortcuts.find((s) => s.scope === scope);
    if (match) return match;
  }

  return null;
}

/**
 * Get winning shortcut from conflict
 */
export function resolveConflict(
  conflict: ShortcutConflict,
  activeScopes: Set<ShortcutScope>
): ShortcutConfig | null {
  switch (conflict.resolution) {
    case "priority":
      return resolveByPriority(conflict.shortcuts);

    case "scope":
      return resolveByScope(conflict.shortcuts, activeScopes);

    case "manual":
    default:
      // Cannot auto-resolve, return highest priority
      return resolveByPriority(conflict.shortcuts);
  }
}

// ============================================================================
// Validation
// ============================================================================

/**
 * Check if new shortcut would create conflicts
 */
export function wouldConflict(
  config: ShortcutConfig,
  existing: RegisteredShortcut[],
  platform?: Platform
): ShortcutConflict | null {
  const normalized = normalizeSequence(config.sequence, platform);
  const scope = config.scope || "global";

  const conflicts: ShortcutConfig[] = [];

  for (const registered of existing) {
    if (!registered.config.enabled) continue;

    const existingNormalized = normalizeSequence(
      registered.config.sequence,
      platform
    );
    const existingScope = registered.config.scope || "global";

    if (normalized === existingNormalized && scopesOverlap(scope, existingScope)) {
      conflicts.push(registered.config);
    }
  }

  if (conflicts.length === 0) {
    return null;
  }

  conflicts.push(config);

  return {
    sequence: normalized,
    shortcuts: conflicts,
    severity: determineSeverity({ sequence: normalized, shortcuts: conflicts, severity: "warning" }),
    resolution: determineResolution({ sequence: normalized, shortcuts: conflicts, severity: "warning" }),
  };
}

/**
 * Suggest alternative shortcuts
 */
export function suggestAlternatives(
  sequence: string,
  platform?: Platform
): string[] {
  const parsed = parseSequence(sequence, platform);
  const alternatives: string[] = [];

  // Try adding Shift
  if (!parsed.modifiers.includes("Shift")) {
    alternatives.push([...parsed.modifiers, "Shift"].sort().join("+") + "+" + parsed.key);
  }

  // Try adding Alt
  if (!parsed.modifiers.includes("Alt")) {
    alternatives.push([...parsed.modifiers, "Alt"].sort().join("+") + "+" + parsed.key);
  }

  // Try different key
  const similarKeys: Record<string, string[]> = {
    k: ["j", "l", "i"],
    j: ["k", "h", "u"],
    w: ["q", "e", "r"],
    p: ["o", "[", "l"],
  };

  const alternativeKeys = similarKeys[parsed.key.toLowerCase()] || [];
  for (const altKey of alternativeKeys) {
    alternatives.push(
      parsed.modifiers.length > 0
        ? parsed.modifiers.join("+") + "+" + altKey.toUpperCase()
        : altKey.toUpperCase()
    );
  }

  return alternatives.slice(0, 5); // Return top 5
}

