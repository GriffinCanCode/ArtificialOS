/**
 * Search Engine
 * Unified search implementation using Fuse.js for fuzzy matching,
 * MiniSearch for autocomplete, and optimized algorithms
 */

import Fuse, { type IFuseOptions, type FuseResult } from "fuse.js";
import MiniSearch from "minisearch";
import type {
  SearchEngine,
  SearchConfig,
  SearchResult,
  SearchMatch,
  SearchIndex,
  Searchable,
  SearchSuggestion,
} from "../core/types";
import { createMemoryIndex } from "../utils/index";

// ============================================================================
// Main Search Engine
// ============================================================================

export class Engine<T extends Searchable> implements SearchEngine<T> {
  readonly provider = "fuse" as const;
  readonly config: SearchConfig<T>;
  readonly index: SearchIndex<T>;

  private fuse: Fuse<T>;
  private miniSearch: MiniSearch<T> | null = null;
  private fields: string[];

  constructor(items: T[], config: SearchConfig<T> = {}) {
    this.config = {
      threshold: 0.3,
      distance: 100,
      minMatchLength: 1,
      includeScore: true,
      includeMatches: false,
      ignoreLocation: true,
      caseSensitive: false,
      ...config,
    };

    this.index = createMemoryIndex<T>();
    this.index.addAll(items);

    this.fields = this.config.keys?.map((k) => k.name) || ["id"];
    this.fuse = this.createFuseInstance(items);

    // Initialize MiniSearch for autocomplete if needed
    if (items.length > 0) {
      this.initMiniSearch(items);
    }
  }

  /**
   * Create Fuse.js instance (primary search engine)
   */
  private createFuseInstance(items: T[]): Fuse<T> {
    const fuseOptions: IFuseOptions<T> = {
      keys: this.config.keys?.map((k) => ({
        name: k.name,
        weight: k.weight ?? 1,
      })) || [{ name: "id", weight: 1 }],
      threshold: this.config.threshold,
      distance: this.config.distance,
      minMatchCharLength: this.config.minMatchLength,
      includeScore: this.config.includeScore,
      includeMatches: this.config.includeMatches,
      ignoreLocation: this.config.ignoreLocation,
      useExtendedSearch: this.config.extendedSearch,
      isCaseSensitive: this.config.caseSensitive,
    };

    return new Fuse(items, fuseOptions);
  }

  /**
   * Initialize MiniSearch for autocomplete suggestions
   */
  private initMiniSearch(items: T[]): void {
    try {
      this.miniSearch = new MiniSearch<T>({
        fields: this.fields,
        storeFields: ["id"],
        searchOptions: {
          boost: this.buildBoostMap(),
          fuzzy: 0.2,
          prefix: true,
        },
      });
      this.miniSearch.addAll(items);
    } catch (error) {
      console.warn("MiniSearch initialization failed, autocomplete disabled:", error);
      this.miniSearch = null;
    }
  }

  /**
   * Build boost map from key weights
   */
  private buildBoostMap(): Record<string, number> {
    const boost: Record<string, number> = {};
    if (this.config.keys) {
      for (const key of this.config.keys) {
        boost[key.name] = key.weight || 1;
      }
    }
    return boost;
  }

  /**
   * Search items using Fuse.js
   */
  search(query: string, options?: Partial<SearchConfig<T>>): SearchResult<T>[] {
    if (!query.trim()) {
      return this.index.getAll().map((item) => ({
        item,
        score: 0,
      }));
    }

    const mergedConfig = { ...this.config, ...options };

    // Use Fuse for primary search
    const results = this.fuse.search(query);

    // Apply limit if specified
    const limited = mergedConfig.limit ? results.slice(0, mergedConfig.limit) : results;

    return limited.map((result) => this.convertResult(result));
  }

  /**
   * Convert Fuse result to SearchResult
   */
  private convertResult(result: FuseResult<T>): SearchResult<T> {
    const matches: SearchMatch[] | undefined = result.matches?.map((match) => ({
      key: match.key || "",
      value: match.value || "",
      indices: (match.indices || []).map((idx) => [idx[0], idx[1]] as [number, number]),
      score: result.score,
    }));

    return {
      item: result.item,
      score: result.score ?? 0,
      matches,
    };
  }

  /**
   * Get autocomplete suggestions using MiniSearch
   * Falls back to Fuse if MiniSearch unavailable
   */
  suggest(query: string, limit: number = 5): SearchSuggestion[] {
    if (!query.trim()) {
      return [];
    }

    // Try MiniSearch first (optimized for autocomplete)
    if (this.miniSearch) {
      try {
        const suggestions = this.miniSearch.autoSuggest(query, {
          fuzzy: 0.2,
          prefix: true,
        });

        return suggestions.slice(0, limit).map((s) => ({
          text: s.suggestion,
          score: s.score,
          type: "completion" as const,
        }));
      } catch (error) {
        console.warn("MiniSearch suggest failed, falling back to Fuse:", error);
      }
    }

    // Fallback to Fuse-based suggestions
    return this.suggestWithFuse(query, limit);
  }

  /**
   * Fallback suggestion using Fuse
   */
  private suggestWithFuse(query: string, limit: number): SearchSuggestion[] {
    const results = this.search(query, { threshold: 0.4, limit: limit * 2 });
    const suggestions = new Set<string>();
    const output: SearchSuggestion[] = [];

    for (const result of results) {
      if (output.length >= limit) break;

      const firstKey = this.config.keys?.[0]?.name || "id";
      const text = String(result.item[firstKey as keyof T] || "");

      if (text && !suggestions.has(text)) {
        suggestions.add(text);
        output.push({
          text,
          score: result.score,
          type: "completion",
        });
      }
    }

    return output;
  }

  /**
   * Add item to index
   */
  add(item: T): void {
    this.index.add(item);
    this.rebuild(this.index.getAll());
  }

  /**
   * Add multiple items
   */
  addAll(items: T[]): void {
    this.index.addAll(items);
    this.rebuild(this.index.getAll());
  }

  /**
   * Remove item from index
   */
  remove(predicate: (item: T) => boolean): void {
    this.index.remove(predicate);
    this.rebuild(this.index.getAll());
  }

  /**
   * Update item in index
   */
  update(item: T): void {
    this.index.update(item);
    this.rebuild(this.index.getAll());
  }

  /**
   * Clear index
   */
  clear(): void {
    this.index.clear();
    this.fuse = this.createFuseInstance([]);
    if (this.miniSearch) {
      this.miniSearch.removeAll();
    }
  }

  /**
   * Rebuild index
   */
  rebuild(items: T[]): void {
    this.fuse = this.createFuseInstance(items);

    if (this.miniSearch && items.length > 0) {
      this.miniSearch.removeAll();
      this.miniSearch.addAll(items);
    } else if (!this.miniSearch && items.length > 0) {
      this.initMiniSearch(items);
    }
  }

  /**
   * Dispose resources
   */
  dispose(): void {
    this.index.clear();
    if (this.miniSearch) {
      this.miniSearch.removeAll();
    }
  }
}

/**
 * Create search engine instance (convenience function)
 */
export function createEngine<T extends Searchable>(
  items: T[],
  config?: SearchConfig<T>
): Engine<T> {
  return new Engine(items, config);
}

