/**
 * Browser App
 * Full-featured web browser with multi-tab support
 */

import { useEffect, useCallback } from 'react';
import type { NativeAppProps } from './sdk.d';
import { useBrowserState } from './hooks/useBrowserState';
import { useTabManager } from './hooks/useTabManager';
import { TabBar } from './components/TabBar';
import { AddressBar } from './components/AddressBar';
import { BrowserView } from './components/BrowserView';
import { processInput } from './utils/url';
import './styles/App.css';

export default function BrowserApp({ context }: NativeAppProps) {
  const { window: win } = context;
  const {
    tabs,
    setTabs,
    activeTabId,
    setActiveTabId,
    settings,
    loadData,
  } = useBrowserState(context);

  const {
    createTab,
    closeTab,
    switchTab,
    updateTab,
    navigateTab,
    goBack,
    goForward,
    reloadTab,
  } = useTabManager(setTabs, activeTabId, setActiveTabId);

  // Initialize
  useEffect(() => {
    win.setTitle('🌐 Browser');
    loadData();
    // Create initial tab
    createTab(settings.homepage);
  }, []);

  const activeTab = tabs.find((t) => t.id === activeTabId);

  // Navigation handlers
  const handleNavigate = useCallback(
    (url: string) => {
      if (!activeTabId) return;
      const finalUrl = processInput(url, settings.searchEngine);
      navigateTab(activeTabId, finalUrl);
    },
    [activeTabId, navigateTab, settings.searchEngine]
  );

  const handleLoadComplete = useCallback(
    (title?: string, favicon?: string) => {
      if (!activeTabId) return;
      updateTab(activeTabId, {
        loading: false,
        title: title || activeTab?.title,
        favicon: favicon || activeTab?.favicon,
      });
    },
    [activeTabId, activeTab, updateTab]
  );

  const handleError = useCallback(
    (error: string) => {
      if (!activeTabId) return;
      updateTab(activeTabId, {
        loading: false,
      });
      console.error('[Browser] Load error:', error);
    },
    [activeTabId, updateTab]
  );

  const handleHome = useCallback(() => {
    handleNavigate(settings.homepage);
  }, [handleNavigate, settings.homepage]);

  return (
    <div className="browser-app">
      {/* Tab bar */}
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onTabClick={switchTab}
        onTabClose={closeTab}
        onNewTab={() => createTab(settings.homepage)}
      />

      {/* Address bar */}
      {activeTab && (
        <AddressBar
          tab={activeTab}
          searchEngine={settings.searchEngine}
          onNavigate={handleNavigate}
          onBack={() => goBack(activeTab.id)}
          onForward={() => goForward(activeTab.id)}
          onReload={() => reloadTab(activeTab.id)}
          onHome={handleHome}
        />
      )}

      {/* Content area */}
      <div className="browser-content">
        {activeTab ? (
          <BrowserView
            key={activeTab.id}
            tab={activeTab}
            context={context}
            onLoadComplete={handleLoadComplete}
            onError={handleError}
          />
        ) : (
          <div className="browser-empty">
            <div className="empty-icon">🌐</div>
            <div className="empty-text">No tabs open</div>
            <button
              className="empty-button"
              onClick={() => createTab(settings.homepage)}
            >
              New Tab
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

