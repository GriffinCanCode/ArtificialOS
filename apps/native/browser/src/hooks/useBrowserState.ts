/**
 * Browser state management hook
 */

import { useState, useCallback } from 'react';
import type { BrowserTab, Bookmark, HistoryEntry, BrowserSettings } from '../types';
import type { NativeAppContext } from '../sdk.d';

export function useBrowserState(context: NativeAppContext) {
  const [tabs, setTabs] = useState<BrowserTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [settings, setSettings] = useState<BrowserSettings>({
    searchEngine: 'duckduckgo',
    homepage: 'about:blank',
    newTabPage: 'blank',
    downloadPath: '/tmp/downloads',
    enableJavaScript: false,
    enableImages: true,
    enableReaderMode: true,
    showConsole: false,
  });

  // Load persisted data
  const loadData = useCallback(async () => {
    try {
      const [bookmarksData, historyData, settingsData] = await Promise.all([
        context.executor.execute('storage.get', { key: 'bookmarks' }),
        context.executor.execute('storage.get', { key: 'history' }),
        context.executor.execute('storage.get', { key: 'settings' }),
      ]);

      if (bookmarksData?.value) setBookmarks(bookmarksData.value);
      if (historyData?.value) setHistory(historyData.value);
      if (settingsData?.value) setSettings({ ...settings, ...settingsData.value });
    } catch (err) {
      console.error('[Browser] Failed to load data:', err);
    }
  }, [context.executor]);

  // Save data
  const saveBookmarks = useCallback(
    async (newBookmarks: Bookmark[]) => {
      setBookmarks(newBookmarks);
      try {
        await context.executor.execute('storage.set', {
          key: 'bookmarks',
          value: newBookmarks,
        });
      } catch (err) {
        console.error('[Browser] Failed to save bookmarks:', err);
      }
    },
    [context.executor]
  );

  const saveHistory = useCallback(
    async (newHistory: HistoryEntry[]) => {
      // Keep only last 1000 entries
      const trimmed = newHistory.slice(-1000);
      setHistory(trimmed);
      try {
        await context.executor.execute('storage.set', {
          key: 'history',
          value: trimmed,
        });
      } catch (err) {
        console.error('[Browser] Failed to save history:', err);
      }
    },
    [context.executor]
  );

  const saveSettings = useCallback(
    async (newSettings: BrowserSettings) => {
      setSettings(newSettings);
      try {
        await context.executor.execute('storage.set', {
          key: 'settings',
          value: newSettings,
        });
      } catch (err) {
        console.error('[Browser] Failed to save settings:', err);
      }
    },
    [context.executor]
  );

  return {
    tabs,
    setTabs,
    activeTabId,
    setActiveTabId,
    bookmarks,
    saveBookmarks,
    history,
    saveHistory,
    settings,
    saveSettings,
    loadData,
  };
}

