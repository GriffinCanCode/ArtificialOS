/**
 * Shortcut Store
 * Zustand state management for keyboard shortcuts with persistence
 */

import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import { registry } from "../core/registry";
import type {
  ShortcutConfig,
  RegisteredShortcut,
  ShortcutScope,
  ShortcutCategory,
  ShortcutConflict,
  ShortcutStats,
  Platform,
} from "../core/types";

// ============================================================================
// Store Interface
// ============================================================================

interface Store {
  // State
  shortcuts: Map<string, RegisteredShortcut>;
  activeScopes: Set<ShortcutScope>;
  platform: Platform;
  enabled: boolean;
  customizations: Map<string, string>; // id -> custom sequence

  // Actions
  register: (config: ShortcutConfig) => string;
  registerMany: (configs: ShortcutConfig[]) => string[];
  unregister: (id: string) => void;
  enable: (id: string) => void;
  disable: (id: string) => void;
  toggle: (id: string) => void;
  customize: (id: string, sequence: string) => void;
  resetCustomization: (id: string) => void;
  setScope: (scope: ShortcutScope, active: boolean) => void;
  setEnabled: (enabled: boolean) => void;

  // Queries
  get: (id: string) => RegisteredShortcut | undefined;
  getAll: () => RegisteredShortcut[];
  getByScope: (scope: ShortcutScope) => RegisteredShortcut[];
  getByCategory: (category: ShortcutCategory) => RegisteredShortcut[];
  findConflicts: () => ShortcutConflict[];
  getStats: () => ShortcutStats;

  // Utilities
  refresh: () => void;
  reset: () => void;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useStore = create<Store>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        shortcuts: new Map(),
        activeScopes: new Set<ShortcutScope>(["global"]),
        platform: registry["platform"],
        enabled: true,
        customizations: new Map(),

        // ====================================================================
        // Actions
        // ====================================================================

        register: (config) => {
          // Apply customization if exists
          const customization = get().customizations.get(config.id);
          const finalConfig = customization
            ? { ...config, sequence: customization }
            : config;

          const id = registry.register(finalConfig);
          get().refresh();
          return id;
        },

        registerMany: (configs) => {
          const ids: string[] = [];
          for (const config of configs) {
            ids.push(get().register(config));
          }
          return ids;
        },

        unregister: (id) => {
          registry.unregister(id);
          set((state) => {
            const newShortcuts = new Map(state.shortcuts);
            newShortcuts.delete(id);
            return { shortcuts: newShortcuts };
          });
        },

        enable: (id) => {
          registry.enable(id);
          get().refresh();
        },

        disable: (id) => {
          registry.disable(id);
          get().refresh();
        },

        toggle: (id) => {
          const shortcut = get().get(id);
          if (shortcut) {
            if (shortcut.config.enabled) {
              get().disable(id);
            } else {
              get().enable(id);
            }
          }
        },

        customize: (id, sequence) => {
          set((state) => {
            const newCustomizations = new Map(state.customizations);
            newCustomizations.set(id, sequence);
            return { customizations: newCustomizations };
          });

          // Update registry
          registry.update(id, { sequence });
          get().refresh();
        },

        resetCustomization: (id) => {
          const shortcut = get().get(id);
          if (!shortcut) return;

          set((state) => {
            const newCustomizations = new Map(state.customizations);
            newCustomizations.delete(id);
            return { customizations: newCustomizations };
          });

          // Get original sequence (would need to store this)
          // For now, just refresh
          get().refresh();
        },

        setScope: (scope, active) => {
          registry.setScope(scope, active);
          set((state) => {
            const newScopes = new Set(state.activeScopes);
            if (active) {
              newScopes.add(scope);
            } else {
              newScopes.delete(scope);
            }
            return { activeScopes: newScopes };
          });
          get().refresh();
        },

        setEnabled: (enabled) => {
          registry.setEnabled(enabled);
          set({ enabled });
        },

        // ====================================================================
        // Queries
        // ====================================================================

        get: (id) => {
          return registry.get(id);
        },

        getAll: () => {
          return registry.getAll();
        },

        getByScope: (scope) => {
          return registry.getByScope(scope);
        },

        getByCategory: (category) => {
          return registry.getByCategory(category);
        },

        findConflicts: () => {
          return registry.findConflicts();
        },

        getStats: () => {
          return registry.getStats();
        },

        // ====================================================================
        // Utilities
        // ====================================================================

        refresh: () => {
          set({
            shortcuts: new Map(
              registry.getAll().map((s) => [s.config.id, s])
            ),
            activeScopes: registry.getActiveScopes(),
          });
        },

        reset: () => {
          registry.clear();
          set({
            shortcuts: new Map(),
            activeScopes: new Set<ShortcutScope>(["global"]),
            customizations: new Map(),
          });
        },
      }),
      {
        name: "shortcuts-storage",
        version: 1,
        partialize: (state) => ({
          customizations: Array.from(state.customizations.entries()),
          enabled: state.enabled,
        }),
        merge: (persistedState: any, currentState: Store): Store => {
          // Restore customizations from array to Map
          const customizations = new Map<string, string>(
            persistedState.customizations || []
          );

          return {
            ...currentState,
            customizations,
            enabled: persistedState.enabled ?? true,
          };
        },
      }
    ),
    { name: "ShortcutStore" }
  )
);

// ============================================================================
// Hooks for Selective Subscriptions
// ============================================================================

/**
 * Subscribe to all shortcuts
 */
export function useShortcuts() {
  return useStore((state) => Array.from(state.shortcuts.values()));
}

/**
 * Subscribe to shortcut actions
 */
export function useActions() {
  return useStore(
    useShallow((state) => ({
      register: state.register,
      registerMany: state.registerMany,
      unregister: state.unregister,
      enable: state.enable,
      disable: state.disable,
      toggle: state.toggle,
      customize: state.customize,
      resetCustomization: state.resetCustomization,
      setScope: state.setScope,
      setEnabled: state.setEnabled,
      refresh: state.refresh,
      reset: state.reset,
    }))
  );
}

/**
 * Subscribe to specific shortcut
 */
export function useShortcut(id: string) {
  return useStore((state) => state.get(id));
}

/**
 * Subscribe to shortcuts by scope
 */
export function useShortcutsByScope(scope: ShortcutScope) {
  return useStore((state) => state.getByScope(scope));
}

/**
 * Subscribe to shortcuts by category
 */
export function useShortcutsByCategory(category: ShortcutCategory) {
  return useStore((state) => state.getByCategory(category));
}

/**
 * Subscribe to active scopes
 */
export function useActiveScopes() {
  return useStore((state) => state.activeScopes);
}

/**
 * Subscribe to conflicts
 */
export function useConflicts() {
  return useStore((state) => state.findConflicts());
}

/**
 * Subscribe to statistics
 */
export function useStats() {
  return useStore((state) => state.getStats());
}

/**
 * Subscribe to enabled state
 */
export function useEnabled() {
  return useStore((state) => state.enabled);
}

