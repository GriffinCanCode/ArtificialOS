/**
 * Shortcut Registry
 * Central registry for all keyboard shortcuts with lifecycle management
 */

// @ts-expect-error - tinykeys has types but package.json exports resolution issue
import { tinykeys } from "tinykeys";
import { toTinykeysFormat, detectPlatform } from "./platform";
import { normalizeSequence, validateSequence } from "./parser";
import { findConflicts, resolveConflict, wouldConflict } from "./conflict";
import type {
  ShortcutConfig,
  RegisteredShortcut,
  ShortcutContext,
  ShortcutScope,
  ShortcutConflict,
  Platform,
  ShortcutStats,
} from "./types";

// ============================================================================
// Registry Class
// ============================================================================

/**
 * Central shortcut registry with lifecycle management
 */
export class ShortcutRegistry {
  private shortcuts = new Map<string, RegisteredShortcut>();
  private activeScopes = new Set<ShortcutScope>(["global"]);
  private platform: Platform = detectPlatform();
  private cleanupFunctions = new Map<string, () => void>();
  private enabled = true;

  // ============================================================================
  // Registration
  // ============================================================================

  /**
   * Register a new shortcut
   */
  register(config: ShortcutConfig): string {
    // Validate sequence
    const validation = validateSequence(config.sequence);
    if (!validation.valid) {
      throw new Error(`Invalid shortcut sequence: ${validation.error}`);
    }

    // Check for conflicts
    const conflict = wouldConflict(config, Array.from(this.shortcuts.values()), this.platform);
    if (conflict && conflict.severity === "error") {
      console.warn(
        `Shortcut conflict detected for ${config.sequence}:`,
        conflict.shortcuts.map((s) => s.id)
      );
    }

    // Create registered shortcut
    const registered: RegisteredShortcut = {
      config: {
        ...config,
        enabled: config.enabled ?? true,
        scope: config.scope || "global",
        priority: config.priority || "normal",
        customizable: config.customizable ?? true,
      },
      registeredAt: Date.now(),
      triggerCount: 0,
      isActive: true,
    };

    // Store shortcut
    this.shortcuts.set(config.id, registered);

    // Bind shortcut if in active scope
    if (this.isScopeActive(registered.config.scope!)) {
      this.bindShortcut(config.id);
    }

    return config.id;
  }

  /**
   * Unregister a shortcut
   */
  unregister(id: string): boolean {
    const registered = this.shortcuts.get(id);
    if (!registered) return false;

    // Cleanup bindings
    this.unbindShortcut(id);

    // Remove from registry
    this.shortcuts.delete(id);

    return true;
  }

  /**
   * Update shortcut configuration
   */
  update(id: string, updates: Partial<ShortcutConfig>): boolean {
    const registered = this.shortcuts.get(id);
    if (!registered) return false;

    // If sequence changed, rebind
    const sequenceChanged = updates.sequence && updates.sequence !== registered.config.sequence;

    if (sequenceChanged) {
      this.unbindShortcut(id);
    }

    // Update config
    registered.config = {
      ...registered.config,
      ...updates,
    };

    if (sequenceChanged) {
      this.bindShortcut(id);
    }

    return true;
  }

  // ============================================================================
  // Binding
  // ============================================================================

  /**
   * Bind shortcut to keyboard events
   */
  private bindShortcut(id: string): void {
    const registered = this.shortcuts.get(id);
    if (!registered || !registered.config.enabled || !this.enabled) {
      return;
    }

    const { config } = registered;
    const sequence = toTinykeysFormat(config.sequence, this.platform);

    // Create handler wrapper
    const handler = (event: KeyboardEvent) => {
      // Check if should ignore (e.g., typing in input)
      if (!config.allowInInput && this.shouldIgnoreEvent(event)) {
        return;
      }

      // Check scope
      if (!this.isScopeActive(config.scope!)) {
        return;
      }

      // Create context
      const context: ShortcutContext = {
        config,
        scope: config.scope!,
        activeScopes: new Set(this.activeScopes),
        platform: this.platform,
      };

      // Execute handler
      try {
        const result = config.handler(event, context);

        // If handler returns false, don't prevent default
        if (result !== false) {
          event.preventDefault();
          event.stopPropagation();
        }

        // Update statistics
        registered.triggerCount++;
        registered.lastTriggered = Date.now();
      } catch (error) {
        console.error(`Error executing shortcut ${config.id}:`, error);
      }
    };

    // Bind with tinykeys
    const cleanup = tinykeys(window, {
      [sequence]: handler,
    });

    // Store cleanup function
    this.cleanupFunctions.set(id, cleanup);
  }

  /**
   * Unbind shortcut from keyboard events
   */
  private unbindShortcut(id: string): void {
    const cleanup = this.cleanupFunctions.get(id);
    if (cleanup) {
      cleanup();
      this.cleanupFunctions.delete(id);
    }
  }

  /**
   * Rebind all shortcuts (useful after scope change)
   */
  private rebindAll(): void {
    // Unbind all
    for (const id of this.shortcuts.keys()) {
      this.unbindShortcut(id);
    }

    // Bind all active
    for (const id of this.shortcuts.keys()) {
      const registered = this.shortcuts.get(id)!;
      if (this.isScopeActive(registered.config.scope!)) {
        this.bindShortcut(id);
      }
    }
  }

  // ============================================================================
  // Scope Management
  // ============================================================================

  /**
   * Set scope active state
   */
  setScope(scope: ShortcutScope, active: boolean): void {
    if (active) {
      this.activeScopes.add(scope);
    } else {
      this.activeScopes.delete(scope);
    }

    // Rebind shortcuts
    this.rebindAll();
  }

  /**
   * Check if scope is active
   */
  isScopeActive(scope: ShortcutScope): boolean {
    return this.activeScopes.has(scope) || this.activeScopes.has("global");
  }

  /**
   * Get active scopes
   */
  getActiveScopes(): Set<ShortcutScope> {
    return new Set(this.activeScopes);
  }

  // ============================================================================
  // Queries
  // ============================================================================

  /**
   * Get shortcut by ID
   */
  get(id: string): RegisteredShortcut | undefined {
    return this.shortcuts.get(id);
  }

  /**
   * Get all shortcuts
   */
  getAll(): RegisteredShortcut[] {
    return Array.from(this.shortcuts.values());
  }

  /**
   * Get shortcuts by scope
   */
  getByScope(scope: ShortcutScope): RegisteredShortcut[] {
    return this.getAll().filter((s) => s.config.scope === scope);
  }

  /**
   * Get shortcuts by category
   */
  getByCategory(category: string): RegisteredShortcut[] {
    return this.getAll().filter((s) => s.config.category === category);
  }

  /**
   * Find shortcuts by sequence
   */
  findBySequence(sequence: string): RegisteredShortcut[] {
    const normalized = normalizeSequence(sequence, this.platform);
    return this.getAll().filter(
      (s) => normalizeSequence(s.config.sequence, this.platform) === normalized
    );
  }

  // ============================================================================
  // Conflict Management
  // ============================================================================

  /**
   * Find all conflicts
   */
  findConflicts(): ShortcutConflict[] {
    return findConflicts(this.getAll(), this.platform);
  }

  /**
   * Resolve conflicts automatically
   */
  resolveConflicts(): void {
    const conflicts = this.findConflicts();

    for (const conflict of conflicts) {
      const winner = resolveConflict(conflict, this.activeScopes);

      if (winner) {
        // Disable conflicting shortcuts
        for (const shortcut of conflict.shortcuts) {
          if (shortcut.id !== winner.id) {
            this.update(shortcut.id, { enabled: false });
          }
        }
      }
    }
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  /**
   * Get registry statistics
   */
  getStats(): ShortcutStats {
    const all = this.getAll();
    const active = all.filter((s) => s.config.enabled);

    // Count by scope
    const byScope: Record<ShortcutScope, number> = {
      global: 0,
      window: 0,
      desktop: 0,
      creator: 0,
      hub: 0,
      terminal: 0,
      app: 0,
    };

    for (const shortcut of all) {
      const scope = shortcut.config.scope || "global";
      byScope[scope]++;
    }

    // Count by category
    const byCategory: Record<string, number> = {};
    for (const shortcut of all) {
      const category = shortcut.config.category || "custom";
      byCategory[category] = (byCategory[category] || 0) + 1;
    }

    // Most used
    const mostUsed = all
      .sort((a, b) => b.triggerCount - a.triggerCount)
      .slice(0, 10)
      .map((s) => ({ id: s.config.id, count: s.triggerCount }));

    return {
      total: all.length,
      active: active.length,
      byScope,
      byCategory: byCategory as any,
      mostUsed,
      conflicts: this.findConflicts(),
    };
  }

  // ============================================================================
  // Control
  // ============================================================================

  /**
   * Enable/disable entire registry
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;

    if (enabled) {
      this.rebindAll();
    } else {
      // Unbind all
      for (const id of this.shortcuts.keys()) {
        this.unbindShortcut(id);
      }
    }
  }

  /**
   * Enable shortcut
   */
  enable(id: string): boolean {
    return this.update(id, { enabled: true });
  }

  /**
   * Disable shortcut
   */
  disable(id: string): boolean {
    const success = this.update(id, { enabled: false });
    if (success) {
      this.unbindShortcut(id);
    }
    return success;
  }

  /**
   * Clear all shortcuts
   */
  clear(): void {
    // Cleanup all bindings
    for (const id of this.shortcuts.keys()) {
      this.unbindShortcut(id);
    }

    this.shortcuts.clear();
    this.cleanupFunctions.clear();
  }

  // ============================================================================
  // Utilities
  // ============================================================================

  /**
   * Check if event should be ignored
   */
  private shouldIgnoreEvent(event: KeyboardEvent): boolean {
    const target = event.target as HTMLElement;

    if (!target) return false;

    // Ignore if typing in input/textarea
    const tagName = target.tagName;
    if (tagName === "INPUT" || tagName === "TEXTAREA") {
      return true;
    }

    // Ignore if contenteditable
    if (target.isContentEditable) {
      return true;
    }

    return false;
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

export const registry = new ShortcutRegistry();

