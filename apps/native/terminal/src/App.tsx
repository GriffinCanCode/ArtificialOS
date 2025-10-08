/**
 * Terminal App
 * Multi-tab terminal emulator with xterm.js
 */

import { useEffect, useState, useCallback } from 'react';
import type { NativeAppProps } from './sdk';
import type { TerminalTab, TerminalSettings } from './types';
import { TerminalView } from './components/TerminalView';
import './styles/App.css';

export default function TerminalApp({ context }: NativeAppProps) {
  const { window: win } = context;

  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [settings] = useState<TerminalSettings>({
    shell: '/bin/zsh',
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: 'dark',
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 10000,
  });

  /**
   * Create new terminal session
   */
  const createSession = useCallback(async () => {
    try {
      const result = await context.executor.execute('terminal.create_session', {
        shell: settings.shell,
        working_dir: '/tmp',
        cols: 80,
        rows: 24,
      });

      if (result?.id) {
        const newTab: TerminalTab = {
          id: `tab-${Date.now()}`,
          sessionId: result.id,
          title: `Terminal ${tabs.length + 1}`,
          active: true,
        };

        setTabs((prev) => [...prev.map((t) => ({ ...t, active: false })), newTab]);
        setActiveTabId(newTab.id);
      }
    } catch (err) {
      console.error('[Terminal] Failed to create session:', err);
      alert(`Failed to create terminal session: ${(err as Error).message}`);
    }
  }, [context.executor, settings.shell, tabs.length]);

  /**
   * Close terminal session
   */
  const closeSession = useCallback(
    async (tabId: string) => {
      const tab = tabs.find((t) => t.id === tabId);
      if (!tab) return;

      try {
        await context.executor.execute('terminal.kill', {
          session_id: tab.sessionId,
        });
      } catch (err) {
        console.error('[Terminal] Failed to kill session:', err);
      }

      setTabs((prev) => {
        const newTabs = prev.filter((t) => t.id !== tabId);

        // If closing active tab, activate another
        if (tabId === activeTabId && newTabs.length > 0) {
          const nextTab = newTabs[newTabs.length - 1];
          nextTab.active = true;
          setActiveTabId(nextTab.id);
        }

        return newTabs;
      });
    },
    [tabs, activeTabId, context.executor]
  );

  /**
   * Switch to tab
   */
  const switchTab = useCallback((tabId: string) => {
    setTabs((prev) =>
      prev.map((t) => ({
        ...t,
        active: t.id === tabId,
      }))
    );
    setActiveTabId(tabId);
  }, []);

  /**
   * Update tab title
   */
  const updateTabTitle = useCallback((tabId: string, title: string) => {
    setTabs((prev) =>
      prev.map((t) => (t.id === tabId ? { ...t, title } : t))
    );
  }, []);

  /**
   * Initialize with first terminal
   */
  useEffect(() => {
    win.setTitle('💻 Terminal');
    createSession();
  }, []);

  const activeTab = tabs.find((t) => t.id === activeTabId);

  return (
    <div className="terminal-app">
      {/* Tab bar */}
      <div className="terminal-tabs">
        <div className="tab-list">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              className={`terminal-tab ${tab.active ? 'active' : ''}`}
              onClick={() => switchTab(tab.id)}
            >
              <span className="tab-icon">💻</span>
              <span className="tab-title">{tab.title}</span>
              <button
                className="tab-close"
                onClick={(e) => {
                  e.stopPropagation();
                  closeSession(tab.id);
                }}
                title="Close terminal"
              >
                ×
              </button>
            </div>
          ))}
        </div>

        <button
          className="tab-new"
          onClick={createSession}
          title="New terminal (Ctrl+Shift+T)"
        >
          +
        </button>
      </div>

      {/* Terminal view */}
      <div className="terminal-content">
        {activeTab && (
          <TerminalView
            key={activeTab.id}
            sessionId={activeTab.sessionId}
            context={context}
            settings={settings}
            onTitleChange={(title) => updateTabTitle(activeTab.id, title)}
          />
        )}

        {tabs.length === 0 && (
          <div className="terminal-empty">
            <div className="empty-icon">💻</div>
            <div className="empty-text">No terminal sessions</div>
            <button className="empty-button" onClick={createSession}>
              Create Terminal
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
