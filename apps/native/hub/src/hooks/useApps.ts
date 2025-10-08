/**
 * useApps Hook
 * Manages app fetching, caching, and filtering
 */

import { useState, useEffect, useCallback } from 'react';
import type { AppMetadata, CategoryFilter } from '../types';
import { fetchApps } from '../lib/api';
import { fuzzySearch } from '../lib/fuzzy';

export function useApps() {
  const [apps, setApps] = useState<AppMetadata[]>([]);
  const [filteredApps, setFilteredApps] = useState<AppMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Load apps from backend
   */
  const loadApps = useCallback(async (category?: string) => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetchApps(category === 'all' ? undefined : category);
      setApps(response.apps);
      setFilteredApps(response.apps);
    } catch (err) {
      console.error('[Hub] Failed to load apps:', err);
      setError(err instanceof Error ? err.message : 'Failed to load apps');
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Filter apps by search query
   */
  const searchApps = useCallback(
    (query: string) => {
      if (!query.trim()) {
        setFilteredApps(apps);
        return;
      }

      const results = fuzzySearch(apps, query, (app) => [
        app.name,
        app.description,
        app.category,
        ...app.tags,
      ]);

      setFilteredApps(results);
    },
    [apps]
  );

  /**
   * Filter apps by category
   */
  const filterByCategory = useCallback(
    (category: CategoryFilter, favorites: Set<string>, recents: Map<string, any>) => {
      if (category === 'all') {
        setFilteredApps(apps);
      } else if (category === 'favorites') {
        setFilteredApps(apps.filter((app) => favorites.has(app.id)));
      } else if (category === 'recent') {
        const recentIds = [...recents.keys()];
        const recentApps = apps
          .filter((app) => recentIds.includes(app.id))
          .sort((a, b) => {
            const aTime = recents.get(a.id)?.lastLaunched || 0;
            const bTime = recents.get(b.id)?.lastLaunched || 0;
            return bTime - aTime;
          });
        setFilteredApps(recentApps);
      } else {
        setFilteredApps(apps.filter((app) => app.category === category));
      }
    },
    [apps]
  );

  /**
   * Reload apps
   */
  const reload = useCallback(() => {
    loadApps();
  }, [loadApps]);

  // Initial load
  useEffect(() => {
    loadApps();
  }, [loadApps]);

  return {
    apps,
    filteredApps,
    loading,
    error,
    searchApps,
    filterByCategory,
    reload,
  };
}

