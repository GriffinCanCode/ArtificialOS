/**
 * Search Engine Factory
 * Creates and manages search engine instances
 */

import type { SearchEngine, SearchConfig, Searchable } from "./types";
import { createEngine as createEngineInstance } from "../engine/engine";

// ============================================================================
// Factory Function
// ============================================================================

/**
 * Create search engine instance
 */
export function createSearchEngine<T extends Searchable>(
  items: T[],
  config: SearchConfig<T> = {}
): SearchEngine<T> {
  return createEngineInstance(items, config);
}


// ============================================================================
// Default Configurations
// ============================================================================

/**
 * Default fuzzy search config (Fuse.js)
 */
export const FUZZY_CONFIG: SearchConfig = {
  provider: "fuse",
  algorithm: "fuzzy",
  threshold: 0.3,
  distance: 100,
  minMatchLength: 1,
  includeScore: true,
  includeMatches: true,
  ignoreLocation: true,
  caseSensitive: false,
};

/**
 * Default exact search config
 */
export const EXACT_CONFIG: SearchConfig = {
  provider: "flex",
  algorithm: "exact",
  threshold: 0,
  caseSensitive: false,
  includeScore: true,
};

/**
 * Default prefix search config (autocomplete)
 */
export const PREFIX_CONFIG: SearchConfig = {
  provider: "mini",
  algorithm: "prefix",
  threshold: 0.1,
  minMatchLength: 1,
  includeScore: true,
};

/**
 * Default weighted search config
 */
export const WEIGHTED_CONFIG: SearchConfig = {
  provider: "fuse",
  algorithm: "weighted",
  threshold: 0.4,
  includeScore: true,
  ignoreLocation: true,
};

// ============================================================================
// Configuration Presets
// ============================================================================

/**
 * Fuzzy search optimized config
 */
export function fuzzyConfig<T>(): SearchConfig<T> {
  return {
    threshold: 0.3,
    distance: 100,
    includeScore: true,
    ignoreLocation: true,
  };
}

/**
 * Exact match config
 */
export function exactConfig<T>(): SearchConfig<T> {
  return {
    threshold: 0,
    caseSensitive: false,
    includeScore: true,
  };
}

/**
 * Autocomplete optimized config
 */
export function autocompleteConfig<T>(): SearchConfig<T> {
  return {
    threshold: 0.1,
    minMatchLength: 1,
    includeScore: true,
  };
}

