/**
 * SearchBar Component
 * Search input with fuzzy matching
 */

import React, { useRef, useEffect } from 'react';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onClear: () => void;
  autoFocus?: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  value,
  onChange,
  onClear,
  autoFocus = false,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  return (
    <div className="search-bar">
      <div className="search-icon">🔍</div>
      <input
        ref={inputRef}
        type="text"
        className="search-input"
        placeholder="Search apps... (press / to focus)"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        autoComplete="off"
        spellCheck={false}
      />
      {value && (
        <button className="search-clear" onClick={onClear} title="Clear search">
          ×
        </button>
      )}
    </div>
  );
};

