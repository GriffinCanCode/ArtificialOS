/**
 * useFavorites Hook
 * Manages favorite apps
 */

import { useState, useCallback } from 'react';
import { loadFavorites, saveFavorites } from '../lib/storage';

export function useFavorites() {
  const [favorites, setFavorites] = useState<Set<string>>(() => loadFavorites());

  /**
   * Toggle favorite status
   */
  const toggleFavorite = useCallback((appId: string) => {
    setFavorites((prev) => {
      const next = new Set(prev);
      if (next.has(appId)) {
        next.delete(appId);
      } else {
        next.add(appId);
      }
      saveFavorites(next);
      return next;
    });
  }, []);

  /**
   * Check if app is favorited
   */
  const isFavorite = useCallback(
    (appId: string) => {
      return favorites.has(appId);
    },
    [favorites]
  );

  return {
    favorites,
    toggleFavorite,
    isFavorite,
  };
}

