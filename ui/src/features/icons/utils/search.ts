/**
 * Icon Search Utility
 * Wrapper around centralized search system for icons
 */

import type { Icon, SearchOptions } from "../core/types";
import { createEngine } from "../../search";
import type { SearchConfig } from "../../search/core/types";

// ============================================================================
// Icon Search Configuration
// ============================================================================

const DEFAULT_CONFIG: SearchConfig<Icon> = {
  keys: [
    { name: "label", weight: 0.7 },
    { name: "metadata.appId", weight: 0.2 },
    { name: "metadata.packageId", weight: 0.2 },
    { name: "metadata.path", weight: 0.1 },
  ],
  threshold: 0.3,
  distance: 100,
  minMatchLength: 1,
  includeScore: true,
  ignoreLocation: true,
};

// ============================================================================
// Search Functions
// ============================================================================

/**
 * Search icons using fuzzy matching
 */
export function searchIcons(
  query: string,
  icons: Icon[],
  options?: SearchOptions
): Array<{ icon: Icon; score: number }> {
  if (!query.trim()) {
    return icons.map((icon) => ({ icon, score: 0 }));
  }

  const config: SearchConfig<Icon> = {
    ...DEFAULT_CONFIG,
    threshold: options?.threshold ?? DEFAULT_CONFIG.threshold,
    keys: options?.keys
      ? options.keys.map((k) => (typeof k === "string" ? { name: k, weight: 1 } : k))
      : DEFAULT_CONFIG.keys,
  };

  const engine = createEngine(icons, config);
  const results = engine.search(query);

  return results.map((result) => ({
    icon: result.item,
    score: result.score,
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
 * Sort icons by search relevance
 */
export function sortByRelevance(icons: Icon[], query: string, options?: SearchOptions): Icon[] {
  if (!query.trim()) {
    return icons;
  }

  const results = searchIcons(query, icons, options);
  return results.map((r) => r.icon);
}

/**
 * Get search suggestions based on partial query
 */
export function getSearchSuggestions(query: string, icons: Icon[], limit: number = 5): string[] {
  if (!query.trim()) {
    return [];
  }

  const config: SearchConfig<Icon> = { ...DEFAULT_CONFIG, threshold: 0.4, limit: limit * 2 };
  const engine = createEngine(icons, config);
  const suggestions = engine.suggest(query, limit);

  return suggestions.map((s) => s.text);
}

/**
 * Check if query matches icon
 */
export function matchesQuery(icon: Icon, query: string, options?: SearchOptions): boolean {
  if (!query.trim()) {
    return true;
  }

  const results = searchIcons(query, [icon], options);
  const threshold = options?.threshold ?? 0.3;

  return results.length > 0 && results[0].score <= threshold;
}

// ============================================================================
// Deprecated Functions (for backward compatibility)
// ============================================================================

/**
 * @deprecated Use searchIcons instead
 * Legacy function for creating search engine
 */
export function createSearchEngine(icons: Icon[], options?: SearchOptions) {
  const config: SearchConfig<Icon> = {
    ...DEFAULT_CONFIG,
    threshold: options?.threshold ?? DEFAULT_CONFIG.threshold,
  };
  return createEngine(icons, config);
}

/**
 * @deprecated Import from @/features/search instead
 */
export { highlight as highlightMatches } from "../../search/features/highlight";

/**
 * @deprecated Import from @/features/search instead
 */
export { highlight as getSearchHighlights } from "../../search/features/highlight";

/**
 * @deprecated Use searchIcons instead
 */
export function calculateRelevance(icon: Icon, query: string, options?: SearchOptions): number {
  const results = searchIcons(query, [icon], options);
  return results[0]?.score ?? 1;
}
