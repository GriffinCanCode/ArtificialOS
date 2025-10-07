/**
 * Dock Store
 * Zustand state management for dock items
 */

import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import type { DockItem } from "../core/types";

// ============================================================================
// Store Interface
// ============================================================================

interface Store {
  items: DockItem[];

  // Actions
  add: (item: Omit<DockItem, "order">) => void;
  remove: (id: string) => void;
  reorder: (fromIndex: number, toIndex: number) => void;
  updateOrder: (items: DockItem[]) => void;
  toggle: (id: string) => void;
  get: (id: string) => DockItem | undefined;
  reset: () => void;
}

// ============================================================================
// Default Items
// ============================================================================

const DEFAULT_ITEMS: DockItem[] = [
  { id: "hub", label: "Hub", icon: "🚀", action: "hub", order: 0, pinned: true },
  { id: "files", label: "Files", icon: "📁", action: "file-explorer", order: 1, pinned: true },
  { id: "browser", label: "Browser", icon: "🌐", action: "browser", order: 2 },
  { id: "calculator", label: "Calculator", icon: "🧮", action: "calculator", order: 3 },
  { id: "notes", label: "Notes", icon: "📝", action: "notes", order: 4 },
  { id: "analysis", label: "Analysis", icon: "📊", action: "system-analysis", order: 5 },
];

// ============================================================================
// Store Implementation
// ============================================================================

export const useStore = create<Store>()(
  devtools(
    persist(
      (set, get) => ({
        items: DEFAULT_ITEMS,

        add: (item) => {
          const items = get().items;
          const maxOrder = Math.max(...items.map((i) => i.order), -1);
          const newItem: DockItem = {
            ...item,
            order: maxOrder + 1,
          };

          set(
            (state) => ({
              items: [...state.items, newItem],
            }),
            false,
            "add"
          );
        },

        remove: (id) => {
          set(
            (state) => {
              const filtered = state.items.filter((item) => item.id !== id && !item.pinned);
              // Reorder remaining items
              return {
                items: filtered.map((item, index) => ({ ...item, order: index })),
              };
            },
            false,
            "remove"
          );
        },

        reorder: (fromIndex, toIndex) => {
          set(
            (state) => {
              const items = [...state.items];
              const [movedItem] = items.splice(fromIndex, 1);
              items.splice(toIndex, 0, movedItem);
              return {
                items: items.map((item, index) => ({ ...item, order: index })),
              };
            },
            false,
            "reorder"
          );
        },

        updateOrder: (items) => {
          set(
            {
              items: items.map((item, index) => ({ ...item, order: index })),
            },
            false,
            "updateOrder"
          );
        },

        toggle: (id) => {
          set(
            (state) => ({
              items: state.items.map((item) =>
                item.id === id ? { ...item, pinned: !item.pinned } : item
              ),
            }),
            false,
            "toggle"
          );
        },

        get: (id) => {
          return get().items.find((item) => item.id === id);
        },

        reset: () => {
          set({ items: DEFAULT_ITEMS }, false, "reset");
        },
      }),
      {
        name: "dock-storage",
        partialize: (state) => ({ items: state.items }),
      }
    ),
    { name: "DockStore" }
  )
);

// ============================================================================
// Convenience Hooks
// ============================================================================

const actionsSelector = (state: Store) => ({
  add: state.add,
  remove: state.remove,
  reorder: state.reorder,
  updateOrder: state.updateOrder,
  toggle: state.toggle,
  get: state.get,
  reset: state.reset,
});

export function useActions() {
  return useStore(useShallow(actionsSelector));
}

export function useDockItems() {
  return useStore(useShallow((state) => state.items));
}
