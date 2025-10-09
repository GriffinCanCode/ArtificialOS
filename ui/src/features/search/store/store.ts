/**
 * Global Search Store
 * Zustand store for Spotlight and global search functionality
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import type { SearchContext, SearchResult, Searchable, SearchConfig } from "../core/types";
import { createSearchEngine } from "../core/engine";

// ============================================================================
// Store Interface
// ============================================================================

interface SearchStore {
  // State
  query: string;
  isActive: boolean;
  contexts: Map<string, SearchContext>;
  activeContextIds: Set<string>;
  results: Map<string, SearchResult<any>[]>;

  // Actions
  setQuery: (query: string) => void;
  activate: () => void;
  deactivate: () => void;
  toggle: () => void;
  clear: () => void;

  // Context management
  registerContext: <T extends Searchable>(
    id: string,
    name: string,
    items: T[],
    config?: SearchConfig<T>,
    priority?: number
  ) => void;
  unregisterContext: (id: string) => void;
  enableContext: (id: string) => void;
  disableContext: (id: string) => void;
  getContextResults: <T extends Searchable>(id: string) => SearchResult<T>[];
  getAllResults: () => Array<{ contextId: string; contextName: string; results: SearchResult<any>[] }>;

  // Search execution
  search: (query?: string) => void;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useSearchStore = create<SearchStore>()(
  devtools(
    (set, get) => ({
      // Initial state
      query: "",
      isActive: false,
      contexts: new Map(),
      activeContextIds: new Set(),
      results: new Map(),

      // ====================================================================
      // Query Management
      // ====================================================================

      setQuery: (query) => {
        set({ query }, false, "setQuery");
        get().search(query);
      },

      activate: () => {
        set({ isActive: true }, false, "activate");
      },

      deactivate: () => {
        set({ isActive: false }, false, "deactivate");
      },

      toggle: () => {
        set((state) => ({ isActive: !state.isActive }), false, "toggle");
      },

      clear: () => {
        set(
          {
            query: "",
            results: new Map(),
          },
          false,
          "clear"
        );
      },

      // ====================================================================
      // Context Management
      // ====================================================================

      registerContext: (id, name, items, config = {}, priority = 0) => {
        const engine = createSearchEngine(items, config);

        const context: SearchContext = {
          id,
          name,
          engine,
          config,
          priority,
          active: true,
        };

        set(
          (state) => {
            const newContexts = new Map(state.contexts);
            newContexts.set(id, context);

            const newActiveContextIds = new Set(state.activeContextIds);
            newActiveContextIds.add(id);

            return {
              contexts: newContexts,
              activeContextIds: newActiveContextIds,
            };
          },
          false,
          "registerContext"
        );

        // Perform initial search if query exists
        if (get().query) {
          get().search();
        }
      },

      unregisterContext: (id) => {
        set(
          (state) => {
            const newContexts = new Map(state.contexts);
            const context = newContexts.get(id);

            if (context) {
              context.engine.dispose?.();
              newContexts.delete(id);
            }

            const newActiveContextIds = new Set(state.activeContextIds);
            newActiveContextIds.delete(id);

            const newResults = new Map(state.results);
            newResults.delete(id);

            return {
              contexts: newContexts,
              activeContextIds: newActiveContextIds,
              results: newResults,
            };
          },
          false,
          "unregisterContext"
        );
      },

      enableContext: (id) => {
        set(
          (state) => {
            const newActiveContextIds = new Set(state.activeContextIds);
            newActiveContextIds.add(id);

            const newContexts = new Map(state.contexts);
            const context = newContexts.get(id);
            if (context) {
              context.active = true;
            }

            return {
              activeContextIds: newActiveContextIds,
              contexts: newContexts,
            };
          },
          false,
          "enableContext"
        );

        // Re-search if query exists
        if (get().query) {
          get().search();
        }
      },

      disableContext: (id) => {
        set(
          (state) => {
            const newActiveContextIds = new Set(state.activeContextIds);
            newActiveContextIds.delete(id);

            const newContexts = new Map(state.contexts);
            const context = newContexts.get(id);
            if (context) {
              context.active = false;
            }

            const newResults = new Map(state.results);
            newResults.delete(id);

            return {
              activeContextIds: newActiveContextIds,
              contexts: newContexts,
              results: newResults,
            };
          },
          false,
          "disableContext"
        );
      },

      getContextResults: (id) => {
        return get().results.get(id) || [];
      },

      getAllResults: () => {
        const state = get();
        const allResults: Array<{
          contextId: string;
          contextName: string;
          results: SearchResult<any>[];
        }> = [];

        // Get contexts sorted by priority
        const sortedContexts = Array.from(state.contexts.values())
          .filter((ctx) => state.activeContextIds.has(ctx.id))
          .sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0));

        for (const context of sortedContexts) {
          const results = state.results.get(context.id) || [];
          if (results.length > 0) {
            allResults.push({
              contextId: context.id,
              contextName: context.name,
              results,
            });
          }
        }

        return allResults;
      },

      // ====================================================================
      // Search Execution
      // ====================================================================

      search: (queryOverride?: string) => {
        const state = get();
        const query = queryOverride ?? state.query;

        if (!query.trim()) {
          set({ results: new Map() }, false, "search:empty");
          return;
        }

        const newResults = new Map<string, SearchResult<any>[]>();

        for (const contextId of state.activeContextIds) {
          const context = state.contexts.get(contextId);
          if (!context) continue;

          try {
            const results = context.engine.search(query);
            newResults.set(contextId, results);
          } catch (error) {
            console.error(`Search failed for context ${contextId}:`, error);
            newResults.set(contextId, []);
          }
        }

        set({ results: newResults }, false, "search");
      },
    }),
    { name: "SearchStore" }
  )
);

// ============================================================================
// Selective Hooks
// ============================================================================

/**
 * Subscribe to search query
 */
export function useSearchQuery() {
  return useSearchStore((state) => state.query);
}

/**
 * Subscribe to search active state
 */
export function useSearchActive() {
  return useSearchStore((state) => state.isActive);
}

/**
 * Subscribe to search actions
 */
export function useSearchActions() {
  return useSearchStore(
    useShallow((state) => ({
      setQuery: state.setQuery,
      activate: state.activate,
      deactivate: state.deactivate,
      toggle: state.toggle,
      clear: state.clear,
      search: state.search,
    }))
  );
}

/**
 * Subscribe to all search results
 * Uses shallow comparison to prevent infinite re-renders
 */
export function useSearchResults() {
  return useSearchStore(useShallow((state) => state.getAllResults()));
}

/**
 * Subscribe to specific context results
 * Uses shallow comparison to prevent infinite re-renders
 */
export function useContextResults(contextId: string) {
  return useSearchStore(useShallow((state) => state.getContextResults(contextId)));
}

/**
 * Subscribe to context management
 */
export function useSearchContexts() {
  return useSearchStore(
    useShallow((state) => ({
      registerContext: state.registerContext,
      unregisterContext: state.unregisterContext,
      enableContext: state.enableContext,
      disableContext: state.disableContext,
    }))
  );
}

