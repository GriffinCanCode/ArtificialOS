/**
 * Preferences Hook
 * Manages user preferences with persistent storage
 */

import { useState, useEffect, useCallback } from 'react';
import type { NativeAppContext } from '../sdk';
import type { Preferences, UsePreferencesReturn } from '../types';

const DEFAULT_PREFERENCES: Preferences = {
  showHidden: false,
  lastPath: '/storage',
  favorites: [],
  recent: [],
  tags: {},
  colors: {},
};

export function usePreferences(context: NativeAppContext): UsePreferencesReturn {
  const { executor } = context;
  const [data, setData] = useState<Preferences>(DEFAULT_PREFERENCES);

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, []);

  // Save preferences when changed
  useEffect(() => {
    savePreferences();
  }, [data]);

  const loadPreferences = async () => {
    try {
      const result = await executor.execute('storage.get', { key: 'preferences' });
      if (result?.value) {
        setData({ ...DEFAULT_PREFERENCES, ...result.value });
      }
    } catch (err) {
      console.error('Failed to load preferences:', err);
    }
  };

  const savePreferences = async () => {
    try {
      await executor.execute('storage.set', { key: 'preferences', value: data });
    } catch (err) {
      console.error('Failed to save preferences:', err);
    }
  };

  const toggleShowHidden = useCallback(() => {
    setData(prev => ({ ...prev, showHidden: !prev.showHidden }));
  }, []);

  const addFavorite = useCallback((path: string) => {
    setData(prev => {
      if (prev.favorites.includes(path)) return prev;
      return { ...prev, favorites: [...prev.favorites, path] };
    });
  }, []);

  const removeFavorite = useCallback((path: string) => {
    setData(prev => ({
      ...prev,
      favorites: prev.favorites.filter(p => p !== path),
    }));
  }, []);

  const addRecent = useCallback((path: string) => {
    setData(prev => {
      const filtered = prev.recent.filter(p => p !== path);
      return {
        ...prev,
        lastPath: path,
        recent: [path, ...filtered].slice(0, 20), // Keep last 20
      };
    });
  }, []);

  const setTag = useCallback((path: string, tags: string[]) => {
    setData(prev => ({
      ...prev,
      tags: { ...prev.tags, [path]: tags },
    }));
  }, []);

  const setColor = useCallback((path: string, color: string) => {
    setData(prev => ({
      ...prev,
      colors: { ...prev.colors, [path]: color },
    }));
  }, []);

  return {
    data,
    toggleShowHidden,
    addFavorite,
    removeFavorite,
    addRecent,
    setTag,
    setColor,
  };
}

