/**
 * Address Bar Component
 * URL input with search and navigation controls
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import type { BrowserTab } from '../types';
import { processInput } from '../utils/url';
import './AddressBar.css';

interface AddressBarProps {
  tab: BrowserTab;
  searchEngine: 'duckduckgo' | 'google' | 'bing';
  onNavigate: (url: string) => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onHome: () => void;
}

export function AddressBar({
  tab,
  searchEngine,
  onNavigate,
  onBack,
  onForward,
  onReload,
  onHome,
}: AddressBarProps) {
  const [input, setInput] = useState(tab.url);
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Update input when tab URL changes
  useEffect(() => {
    if (!focused) {
      setInput(tab.url);
    }
  }, [tab.url, focused]);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const finalUrl = processInput(input, searchEngine);
      onNavigate(finalUrl);
      inputRef.current?.blur();
    },
    [input, searchEngine, onNavigate]
  );

  const handleFocus = useCallback(() => {
    setFocused(true);
    inputRef.current?.select();
  }, []);

  const handleBlur = useCallback(() => {
    setFocused(false);
    setInput(tab.url);
  }, [tab.url]);

  return (
    <div className="address-bar">
      {/* Navigation controls */}
      <div className="nav-controls">
        <button
          className="nav-btn"
          onClick={onBack}
          disabled={!tab.canGoBack}
          title="Go back (Alt+Left)"
        >
          ←
        </button>
        <button
          className="nav-btn"
          onClick={onForward}
          disabled={!tab.canGoForward}
          title="Go forward (Alt+Right)"
        >
          →
        </button>
        <button
          className="nav-btn"
          onClick={onReload}
          disabled={tab.loading}
          title="Reload (Ctrl+R)"
        >
          {tab.loading ? '⟳' : '↻'}
        </button>
        <button className="nav-btn" onClick={onHome} title="Home">
          🏠
        </button>
      </div>

      {/* URL input */}
      <form className="url-form" onSubmit={handleSubmit}>
        <div className="url-input-container">
          {tab.loading && <div className="loading-indicator" />}
          <input
            ref={inputRef}
            type="text"
            className="url-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onFocus={handleFocus}
            onBlur={handleBlur}
            placeholder="Search or enter URL..."
            spellCheck={false}
          />
          <button type="submit" className="go-btn" title="Go (Enter)">
            →
          </button>
        </div>
      </form>
    </div>
  );
}

