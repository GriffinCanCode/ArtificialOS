/**
 * Icon Search Component
 * Search bar with fuzzy matching for icon filtering
 */

import React, { useCallback, useRef, useEffect } from "react";
import { useActions, useSearchState } from "../store/store";
import "./Search.css";

// ============================================================================
// Search Component Props
// ============================================================================

export interface SearchProps {
  onFocus?: () => void;
  onBlur?: () => void;
  onEscape?: () => void;
  placeholder?: string;
  autoFocus?: boolean;
}

// ============================================================================
// Search Component
// ============================================================================

export const Search: React.FC<SearchProps> = React.memo(
  ({ onFocus, onBlur, onEscape, placeholder = "Search icons...", autoFocus = false }) => {
    const searchState = useSearchState();
    const { setSearchQuery } = useActions();
    const inputRef = useRef<HTMLInputElement>(null);

    // Handle input change
    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        setSearchQuery(e.target.value);
      },
      [setSearchQuery]
    );

    // Handle clear
    const handleClear = useCallback(() => {
      setSearchQuery("");
      inputRef.current?.focus();
    }, [setSearchQuery]);

    // Handle escape
    const handleKeyDown = useCallback(
      (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Escape") {
          e.preventDefault();
          if (searchState.query) {
            handleClear();
          } else if (onEscape) {
            onEscape();
          }
        }
      },
      [searchState.query, handleClear, onEscape]
    );

    // Auto-focus if requested
    useEffect(() => {
      if (autoFocus && inputRef.current) {
        inputRef.current.focus();
      }
    }, [autoFocus]);

    return (
      <div className={`icon-search ${searchState.isActive ? "active" : ""}`}>
        <div className="icon-search-icon">🔍</div>
        <input
          ref={inputRef}
          type="text"
          className="icon-search-input"
          placeholder={placeholder}
          value={searchState.query}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onFocus={onFocus}
          onBlur={onBlur}
        />
        {searchState.isActive && (
          <button className="icon-search-clear" onClick={handleClear} aria-label="Clear search">
            ✕
          </button>
        )}
        {searchState.isActive && (
          <div className="icon-search-results">
            {searchState.results.length} result{searchState.results.length !== 1 ? "s" : ""}
          </div>
        )}
      </div>
    );
  }
);

Search.displayName = "IconSearch";

