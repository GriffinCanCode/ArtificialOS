/**
 * useRecents Hook
 * Manages recently launched apps
 */

import { useState, useCallback } from 'react';
import type { RecentApp } from '../types';
import { loadRecents, recordLaunch } from '../lib/storage';

export function useRecents() {
  const [recents, setRecents] = useState<Map<string, RecentApp>>(() => loadRecents());

  /**
   * Record an app launch
   */
  const addRecent = useCallback((appId: string) => {
    setRecents((prev) => recordLaunch(appId, prev));
  }, []);

  /**
   * Get recently launched apps sorted by time
   */
  const getRecents = useCallback(() => {
    return [...recents.entries()]
      .sort((a, b) => b[1].lastLaunched - a[1].lastLaunched)
      .map(([, data]) => data);
  }, [recents]);

  return {
    recents,
    addRecent,
    getRecents,
  };
}

