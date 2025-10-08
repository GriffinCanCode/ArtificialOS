/**
 * Icon Search Utility
 * Fuzzy search using Fuse.js for icon filtering
 */

import Fuse, { type IFuseOptions } from "fuse.js";
import type { Icon, SearchOptions } from "../core/types";

// ============================================================================
// Default Search Configuration
// ============================================================================

const DEFAULT_SEARCH_OPTIONS: IFuseOptions<Icon> = {
  keys: [
    { name: "label", weight: 0.7 },
    { name: "metadata.appId", weight: 0.2 },
    { name: "metadata.packageId", weight: 0.2 },
    { name: "metadata.path", weight: 0.1 },
  ],
  threshold: 0.3, // Lower = more strict, higher = more fuzzy
  distance: 100,
  minMatchCharLength: 1,
  includeScore: true,
  useExtendedSearch: false,
  ignoreLocation: true,
};

// ============================================================================
// Search Engine
// ============================================================================

/**
 * Create Fuse search instance for icons
 */
export function createSearchEngine(icons: Icon[], options?: SearchOptions): Fuse<Icon> {
  const fuseOptions: IFuseOptions<Icon> = {
    ...DEFAULT_SEARCH_OPTIONS,
    threshold: options?.threshold ?? DEFAULT_SEARCH_OPTIONS.threshold,
    includeScore: options?.includeScore ?? true,
  };

  // Override keys if provided
  if (options?.keys) {
    fuseOptions.keys = options.keys;
  }

  return new Fuse(icons, fuseOptions);
}

/**
 * Search icons using fuzzy matching
 * Returns sorted results with scores
 */
export function searchIcons(
  query: string,
  icons: Icon[],
  options?: SearchOptions
): Array<{ icon: Icon; score: number }> {
  if (!query.trim()) {
    return icons.map((icon) => ({ icon, score: 0 }));
  }

  const fuse = createSearchEngine(icons, options);
  const results = fuse.search(query);

  return results.map((result) => ({
    icon: result.item,
    score: result.score ?? 0,
  }));
}

/**
 * Filter icons by search query
 * Returns only matching icon IDs
 */
export function filterIcons(query: string, icons: Icon[], options?: SearchOptions): string[] {
  if (!query.trim()) {
    return icons.map((icon) => icon.id);
  }

  const results = searchIcons(query, icons, options);
  return results.map((result) => result.icon.id);
}

/**
 * Get search highlights for matched text
 * Returns character indices to highlight
 */
export function getSearchHighlights(
  query: string,
  text: string,
  options?: SearchOptions
): Array<[number, number]> {
  if (!query.trim() || !text) {
    return [];
  }

  const fuse = new Fuse([text], {
    threshold: options?.threshold ?? 0.3,
    includeMatches: true,
    ignoreLocation: true,
  });

  const result = fuse.search(query)[0];
  if (!result?.matches) {
    return [];
  }

  const highlights: Array<[number, number]> = [];
  for (const match of result.matches) {
    if (match.indices) {
      highlights.push(...match.indices);
    }
  }

  return highlights;
}

/**
 * Highlight search query in text
 * Returns text with <mark> tags around matches
 */
export function highlightMatches(query: string, text: string, options?: SearchOptions): string {
  const highlights = getSearchHighlights(query, text, options);
  if (highlights.length === 0) {
    return text;
  }

  // Sort highlights by start index
  const sorted = [...highlights].sort((a, b) => a[0] - b[0]);

  // Build highlighted text
  let result = "";
  let lastIndex = 0;

  for (const [start, end] of sorted) {
    // Add text before highlight
    result += text.slice(lastIndex, start);
    // Add highlighted text
    result += `<mark>${text.slice(start, end + 1)}</mark>`;
    lastIndex = end + 1;
  }

  // Add remaining text
  result += text.slice(lastIndex);

  return result;
}

// ============================================================================
// Search Scoring
// ============================================================================

/**
 * Calculate relevance score for icon
 * Lower score = better match
 */
export function calculateRelevance(icon: Icon, query: string, options?: SearchOptions): number {
  if (!query.trim()) {
    return 0;
  }

  const fuse = createSearchEngine([icon], options);
  const results = fuse.search(query);

  return results[0]?.score ?? 1;
}

/**
 * Sort icons by search relevance
 */
export function sortByRelevance(icons: Icon[], query: string, options?: SearchOptions): Icon[] {
  if (!query.trim()) {
    return icons;
  }

  const results = searchIcons(query, icons, options);
  return results.map((r) => ({
    ...r.icon,
    searchScore: r.score,
  }));
}

// ============================================================================
// Search Suggestions
// ============================================================================

/**
 * Get search suggestions based on partial query
 */
export function getSearchSuggestions(query: string, icons: Icon[], limit: number = 5): string[] {
  if (!query.trim()) {
    return [];
  }

  const results = searchIcons(query, icons, { threshold: 0.4 });
  const suggestions = new Set<string>();

  for (const result of results) {
    if (suggestions.size >= limit) break;
    suggestions.add(result.icon.label);
  }

  return Array.from(suggestions);
}

/**
 * Check if query matches icon
 */
export function matchesQuery(icon: Icon, query: string, options?: SearchOptions): boolean {
  if (!query.trim()) {
    return true;
  }

  const score = calculateRelevance(icon, query, options);
  const threshold = options?.threshold ?? 0.3;

  return score <= threshold;
}

