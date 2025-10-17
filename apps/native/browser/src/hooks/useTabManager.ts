/**
 * Tab management hook
 */

import { useCallback } from 'react';
import type { BrowserTab } from '../types';
import { getTitleFromUrl } from '../utils/url';
import { generateTabId } from '../utils/id';

export function useTabManager(
  setTabs: React.Dispatch<React.SetStateAction<BrowserTab[]>>,
  activeTabId: string | null,
  setActiveTabId: React.Dispatch<React.SetStateAction<string | null>>
) {
  /**
   * Create new tab
   */
  const createTab = useCallback(
    (url: string = 'about:blank') => {
      const newTab: BrowserTab = {
        id: generateTabId(),
        title: getTitleFromUrl(url),
        url,
        loading: false,
        canGoBack: false,
        canGoForward: false,
        history: [url],
        historyIndex: 0,
      };

      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(newTab.id);
      return newTab.id;
    },
    [setTabs, setActiveTabId]
  );

  /**
   * Close tab
   */
  const closeTab = useCallback(
    (tabId: string) => {
      setTabs((prev) => {
        const newTabs = prev.filter((t) => t.id !== tabId);

        // If closing active tab, switch to another
        if (tabId === activeTabId && newTabs.length > 0) {
          const index = prev.findIndex((t) => t.id === tabId);
          const nextTab = newTabs[Math.min(index, newTabs.length - 1)];
          setActiveTabId(nextTab.id);
        } else if (newTabs.length === 0) {
          setActiveTabId(null);
        }

        return newTabs;
      });
    },
    [setTabs, setActiveTabId, activeTabId]
  );

  /**
   * Switch to tab
   */
  const switchTab = useCallback(
    (tabId: string) => {
      setActiveTabId(tabId);
    },
    [setActiveTabId]
  );

  /**
   * Update tab properties
   */
  const updateTab = useCallback(
    (tabId: string, updates: Partial<BrowserTab>) => {
      setTabs((prev) =>
        prev.map((tab) => (tab.id === tabId ? { ...tab, ...updates } : tab))
      );
    },
    [setTabs]
  );

  /**
   * Navigate to URL in tab
   */
  const navigateTab = useCallback(
    (tabId: string, url: string) => {
      setTabs((prev) =>
        prev.map((tab) => {
          if (tab.id !== tabId) return tab;

          // Add to history
          const newHistory = [...tab.history.slice(0, tab.historyIndex + 1), url];
          const newIndex = newHistory.length - 1;

          return {
            ...tab,
            url,
            title: getTitleFromUrl(url),
            loading: true,
            history: newHistory,
            historyIndex: newIndex,
            canGoBack: newIndex > 0,
            canGoForward: false,
          };
        })
      );
    },
    [setTabs]
  );

  /**
   * Go back in history
   */
  const goBack = useCallback(
    (tabId: string) => {
      setTabs((prev) =>
        prev.map((tab) => {
          if (tab.id !== tabId || !tab.canGoBack) return tab;

          const newIndex = tab.historyIndex - 1;
          const url = tab.history[newIndex];

          return {
            ...tab,
            url,
            title: getTitleFromUrl(url),
            loading: true,
            historyIndex: newIndex,
            canGoBack: newIndex > 0,
            canGoForward: true,
          };
        })
      );
    },
    [setTabs]
  );

  /**
   * Go forward in history
   */
  const goForward = useCallback(
    (tabId: string) => {
      setTabs((prev) =>
        prev.map((tab) => {
          if (tab.id !== tabId || !tab.canGoForward) return tab;

          const newIndex = tab.historyIndex + 1;
          const url = tab.history[newIndex];

          return {
            ...tab,
            url,
            title: getTitleFromUrl(url),
            loading: true,
            historyIndex: newIndex,
            canGoBack: true,
            canGoForward: newIndex < tab.history.length - 1,
          };
        })
      );
    },
    [setTabs]
  );

  /**
   * Reload tab
   */
  const reloadTab = useCallback(
    (tabId: string) => {
      setTabs((prev) =>
        prev.map((tab) =>
          tab.id === tabId ? { ...tab, loading: true } : tab
        )
      );
    },
    [setTabs]
  );

  return {
    createTab,
    closeTab,
    switchTab,
    updateTab,
    navigateTab,
    goBack,
    goForward,
    reloadTab,
  };
}

