/**
 * Local Storage Management
 * Handles favorites and recents persistence
 */

import type { RecentApp } from '../types';

const FAVORITES_KEY = 'hub_favorites';
const RECENTS_KEY = 'hub_recents';
const MAX_RECENTS = 20;

/**
 * Load favorites from localStorage
 */
export function loadFavorites(): Set<string> {
  try {
    const stored = localStorage.getItem(FAVORITES_KEY);
    if (stored) {
      return new Set(JSON.parse(stored));
    }
  } catch (err) {
    console.error('[Hub] Failed to load favorites:', err);
  }
  return new Set();
}

/**
 * Save favorites to localStorage
 */
export function saveFavorites(favorites: Set<string>): void {
  try {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify([...favorites]));
  } catch (err) {
    console.error('[Hub] Failed to save favorites:', err);
  }
}

/**
 * Load recents from localStorage
 */
export function loadRecents(): Map<string, RecentApp> {
  try {
    const stored = localStorage.getItem(RECENTS_KEY);
    if (stored) {
      const array = JSON.parse(stored) as Array<[string, RecentApp]>;
      return new Map(array);
    }
  } catch (err) {
    console.error('[Hub] Failed to load recents:', err);
  }
  return new Map();
}

/**
 * Save recents to localStorage
 */
export function saveRecents(recents: Map<string, RecentApp>): void {
  try {
    // Keep only top N most recent
    const sorted = [...recents.entries()]
      .sort((a, b) => b[1].lastLaunched - a[1].lastLaunched)
      .slice(0, MAX_RECENTS);

    localStorage.setItem(RECENTS_KEY, JSON.stringify(sorted));
  } catch (err) {
    console.error('[Hub] Failed to save recents:', err);
  }
}

/**
 * Record app launch
 */
export function recordLaunch(appId: string, recents: Map<string, RecentApp>): Map<string, RecentApp> {
  const now = Date.now();
  const existing = recents.get(appId);

  const updated = new Map(recents);
  updated.set(appId, {
    id: appId,
    lastLaunched: now,
    launchCount: existing ? existing.launchCount + 1 : 1,
  });

  saveRecents(updated);
  return updated;
}

