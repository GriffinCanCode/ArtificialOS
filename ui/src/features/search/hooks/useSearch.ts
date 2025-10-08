/**
 * useSearch Hook
 * React hook for search functionality
 */

import { useState, useMemo, useCallback, useEffect } from "react";
import type { SearchResult, SearchConfig, Searchable } from "../core/types";
import { createSearchEngine } from "../core/engine";

// ============================================================================
// useSearch Hook
// ============================================================================

export interface UseSearchOptions<T extends Searchable> extends SearchConfig<T> {
  /** Initial items to search */
  items: T[];

  /** Initial query */
  initialQuery?: string;

  /** Debounce delay in ms */
  debounce?: number;
}

export interface UseSearchReturn<T extends Searchable> {
  /** Current search query */
  query: string;

  /** Set search query */
  setQuery: (query: string) => void;

  /** Search results */
  results: SearchResult<T>[];

  /** Whether search is active */
  isSearching: boolean;

  /** Number of results */
  count: number;

  /** Clear search */
  clear: () => void;

  /** Refresh search with current query */
  refresh: () => void;
}

/**
 * Search hook with automatic result updates
 */
export function useSearch<T extends Searchable>(
  options: UseSearchOptions<T>
): UseSearchReturn<T> {
  const { items, initialQuery = "", debounce = 0, ...config } = options;

  const [query, setQuery] = useState(initialQuery);
  const [debouncedQuery, setDebouncedQuery] = useState(initialQuery);

  // Create search engine (memoized)
  const engine = useMemo(() => {
    return createSearchEngine(items, config);
  }, [items, config]);

  // Debounce query
  useEffect(() => {
    if (debounce === 0) {
      setDebouncedQuery(query);
      return;
    }

    const timer = setTimeout(() => {
      setDebouncedQuery(query);
    }, debounce);

    return () => clearTimeout(timer);
  }, [query, debounce]);

  // Perform search
  const results = useMemo(() => {
    return engine.search(debouncedQuery);
  }, [engine, debouncedQuery]);

  const clear = useCallback(() => {
    setQuery("");
  }, []);

  const refresh = useCallback(() => {
    setDebouncedQuery(query);
  }, [query]);

  return {
    query,
    setQuery,
    results,
    isSearching: query.length > 0,
    count: results.length,
    clear,
    refresh,
  };
}

