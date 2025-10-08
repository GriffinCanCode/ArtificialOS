/**
 * Spotlight Search Component
 * macOS-style global search interface
 */

import React, { useEffect, useCallback, useRef, useState } from "react";
import { useSearchActions, useSearchResults, useSearchQuery, useSearchActive } from "../store/store";
import { useScope } from "../../input";
import { useDebouncedValue } from "../../../hooks/useDebouncedValue";
import "./Spotlight.css";

export interface SpotlightProps {
  /** Whether spotlight is visible */
  isOpen: boolean;
  /** Callback when spotlight closes */
  onClose: () => void;
  /** Callback when an item is selected */
  onSelect?: (item: any) => void;
}

export const Spotlight: React.FC<SpotlightProps> = ({ isOpen, onClose, onSelect }) => {
  const inputRef = useRef<HTMLInputElement>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const query = useSearchQuery();
  const { setQuery, clear, deactivate } = useSearchActions();
  const results = useSearchResults();

  // Debounce search to avoid too many queries
  const debouncedQuery = useDebouncedValue(query, 150);

  // Activate spotlight scope when open
  useScope("spotlight", { enabled: isOpen });

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setSelectedIndex(0);
    }
  }, [isOpen]);

  // Update search when query changes
  useEffect(() => {
    if (debouncedQuery && isOpen) {
      // Search will be triggered by the store automatically
    }
  }, [debouncedQuery, isOpen]);

  // Handle close
  const handleClose = useCallback(() => {
    clear();
    deactivate();
    onClose();
    setSelectedIndex(0);
  }, [clear, deactivate, onClose]);

  // Handle keyboard navigation
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const flatResults = results.flatMap((ctx) => ctx.results);
      const totalResults = flatResults.length;

      switch (e.key) {
        case "Escape":
          e.preventDefault();
          handleClose();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => Math.min(prev + 1, totalResults - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => Math.max(prev - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (flatResults[selectedIndex]) {
            onSelect?.(flatResults[selectedIndex]);
            handleClose();
          }
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, results, selectedIndex, handleClose, onSelect]);

  if (!isOpen) return null;

  // Flatten results for rendering
  let flatIndex = 0;

  return (
    <>
      {/* Backdrop */}
      <div className="spotlight-backdrop" onClick={handleClose} />

      {/* Spotlight Window */}
      <div className="spotlight-window">
        {/* Search Input */}
        <div className="spotlight-header">
          <div className="spotlight-icon">🔍</div>
          <input
            ref={inputRef}
            type="text"
            className="spotlight-input"
            placeholder="Search anything..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            spellCheck={false}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
          />
          {query && (
            <button className="spotlight-clear" onClick={() => clear()} aria-label="Clear">
              ✕
            </button>
          )}
        </div>

        {/* Results */}
        <div className="spotlight-results">
          {!query && (
            <div className="spotlight-empty">
              <p>Type to search files, apps, and more</p>
              <div className="spotlight-hints">
                <kbd>↑</kbd><kbd>↓</kbd> navigate  •  <kbd>Enter</kbd> open  •  <kbd>Esc</kbd> close
              </div>
            </div>
          )}

          {query && results.length === 0 && (
            <div className="spotlight-empty">
              <p>No results found for "{query}"</p>
            </div>
          )}

          {results.map((context) => {
            if (context.results.length === 0) return null;

            return (
              <div key={context.contextId} className="spotlight-category">
                <div className="spotlight-category-header">{context.contextName}</div>
                <div className="spotlight-category-results">
                  {context.results.slice(0, 5).map((result) => {
                    const isSelected = flatIndex === selectedIndex;
                    const currentIndex = flatIndex++;

                    return (
                      <div
                        key={currentIndex}
                        className={`spotlight-result ${isSelected ? "selected" : ""}`}
                        onClick={() => {
                          onSelect?.(result);
                          handleClose();
                        }}
                        onMouseEnter={() => setSelectedIndex(currentIndex)}
                      >
                        <div className="spotlight-result-icon">
                          {getResultIcon(context.contextId)}
                        </div>
                        <div className="spotlight-result-content">
                          <div className="spotlight-result-title">
                            {getResultTitle(result.item)}
                          </div>
                          <div className="spotlight-result-subtitle">
                            {getResultSubtitle(result.item, context.contextId)}
                          </div>
                        </div>
                        {result.score !== undefined && (
                          <div className="spotlight-result-score">
                            {Math.round((1 - result.score) * 100)}% match
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </>
  );
};

// Helper functions for rendering results
function getResultIcon(contextId: string): string {
  switch (contextId) {
    case "files":
      return "📄";
    case "apps":
      return "📱";
    case "actions":
      return "⚡";
    case "services":
      return "🔧";
    default:
      return "📦";
  }
}

function getResultTitle(item: any): string {
  return item.name || item.label || item.title || item.path || "Unknown";
}

function getResultSubtitle(item: any, contextId: string): string {
  if (contextId === "files") {
    return item.path || "";
  }
  return item.description || item.category || "";
}

