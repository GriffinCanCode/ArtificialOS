/**
 * Spotlight Search Component
 * Modern search interface with GSAP animations
 * Drops down from title bar for spatial continuity
 */

import React, { useEffect, useCallback, useRef, useState } from "react";
import { useSearchActions, useSearchResults, useSearchQuery } from "../store/store";
import { useDebouncedValue } from "../../../hooks/useDebouncedValue";
import { Search, FileText, AppWindow, Zap, Settings, X } from "lucide-react";
import gsap from "gsap";
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
  const backdropRef = useRef<HTMLDivElement>(null);
  const windowRef = useRef<HTMLDivElement>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const query = useSearchQuery();
  const { setQuery, clear, deactivate } = useSearchActions();
  const results = useSearchResults();

  // Debounce search to avoid too many queries
  const debouncedQuery = useDebouncedValue(query, 150);

  // GSAP Animation on mount/unmount
  useEffect(() => {
    if (!backdropRef.current || !windowRef.current) return;

    if (isOpen) {
      // Opening animation
      const tl = gsap.timeline({
        defaults: { ease: "power3.out" }
      });

      // Start window from top-right (where search button is)
      gsap.set(windowRef.current, {
        y: -50,
        x: 100,
        scale: 0.85,
        opacity: 0,
        transformOrigin: "top right"
      });

      tl.to(backdropRef.current, {
        opacity: 1,
        duration: 0.3,
        ease: "power2.out"
      })
      .to(windowRef.current, {
        y: 0,
        x: 0,
        scale: 1,
        opacity: 1,
        duration: 0.45,
        ease: "back.out(1.4)"
      }, "-=0.1");

      // Focus input after animation starts
      setTimeout(() => {
        inputRef.current?.focus();
      }, 100);

      setSelectedIndex(0);
    } else {
      // Closing animation
      const tl = gsap.timeline({
        onComplete: () => {
          // Reset after close animation
          gsap.set([backdropRef.current, windowRef.current], { clearProps: "all" });
        }
      });

      tl.to(windowRef.current, {
        y: -30,
        scale: 0.9,
        opacity: 0,
        duration: 0.25,
        ease: "power2.in"
      })
      .to(backdropRef.current, {
        opacity: 0,
        duration: 0.2,
        ease: "power2.in"
      }, "-=0.1");
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
      <div
        ref={backdropRef}
        className="spotlight-backdrop"
        onClick={handleClose}
        style={{ opacity: 0 }}
      />

      {/* Spotlight Window */}
      <div
        ref={windowRef}
        className="spotlight-window"
        style={{ opacity: 0 }}
      >
        {/* Search Input */}
        <div className="spotlight-header">
          <Search className="spotlight-search-icon" size={20} strokeWidth={2.5} />
          <input
            ref={inputRef}
            type="text"
            className="spotlight-input"
            placeholder="Search files, apps, and more..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            spellCheck={false}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
          />
          {query && (
            <button className="spotlight-clear" onClick={() => clear()} aria-label="Clear">
              <X size={16} />
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
                        {isSelected && (
                          <div className="spotlight-result-action">
                            <kbd className="spotlight-kbd">↵</kbd>
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
function getResultIcon(contextId: string): React.ReactNode {
  const iconProps = { size: 18, strokeWidth: 2 };

  switch (contextId) {
    case "files":
      return <FileText {...iconProps} />;
    case "apps":
      return <AppWindow {...iconProps} />;
    case "actions":
      return <Zap {...iconProps} />;
    case "services":
      return <Settings {...iconProps} />;
    default:
      return <FileText {...iconProps} />;
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

