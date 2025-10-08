/**
 * Search Core Types
 * Type definitions for centralized search system
 */

// ============================================================================
// Search Providers
// ============================================================================

/**
 * Available search engine providers
 */
export type SearchProvider = "fuse" | "flex" | "mini" | "native";

/**
 * Search algorithm types
 */
export type SearchAlgorithm = "fuzzy" | "exact" | "prefix" | "contains" | "semantic" | "weighted";

// ============================================================================
// Search Configuration
// ============================================================================

/**
 * Generic search engine configuration
 */
export interface SearchConfig<T = any> {
  /** Search provider to use */
  provider?: SearchProvider;

  /** Algorithm type */
  algorithm?: SearchAlgorithm;

  /** Fields to search in (with optional weights) */
  keys?: SearchKey[];

  /** Fuzzy matching threshold (0.0 = perfect match, 1.0 = match anything) */
  threshold?: number;

  /** Maximum distance for fuzzy matching */
  distance?: number;

  /** Minimum characters to match */
  minMatchLength?: number;

  /** Include score in results */
  includeScore?: boolean;

  /** Include matched indices */
  includeMatches?: boolean;

  /** Ignore field location when matching */
  ignoreLocation?: boolean;

  /** Case sensitive search */
  caseSensitive?: boolean;

  /** Token separator (for tokenization) */
  tokenSeparator?: RegExp;

  /** Use extended search syntax */
  extendedSearch?: boolean;

  /** Custom scoring function */
  scoreFn?: (item: T, query: string) => number;

  /** Limit results */
  limit?: number;
}

/**
 * Search field key with optional weight
 */
export interface SearchKey {
  name: string;
  weight?: number;
}

// ============================================================================
// Search Results
// ============================================================================

/**
 * Generic search result
 */
export interface SearchResult<T = any> {
  /** Matched item */
  item: T;

  /** Relevance score (0 = perfect match, 1 = worst match) */
  score: number;

  /** Matched field indices */
  matches?: SearchMatch[];

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Match information for highlighting
 */
export interface SearchMatch {
  /** Field name that matched */
  key: string;

  /** Matched text value */
  value: string;

  /** Character indices of matches */
  indices: Array<[number, number]>;

  /** Match score */
  score?: number;
}

/**
 * Search suggestions
 */
export interface SearchSuggestion {
  /** Suggested query text */
  text: string;

  /** Suggestion score */
  score: number;

  /** Suggestion type */
  type: "completion" | "correction" | "history" | "trending";

  /** Additional context */
  context?: string;
}

// ============================================================================
// Search Index
// ============================================================================

/**
 * Search index for efficient lookups
 */
export interface SearchIndex<T = any> {
  /** Add item to index */
  add(item: T): void;

  /** Add multiple items */
  addAll(items: T[]): void;

  /** Remove item from index */
  remove(predicate: (item: T) => boolean): void;

  /** Update item in index */
  update(item: T): void;

  /** Clear entire index */
  clear(): void;

  /** Get all indexed items */
  getAll(): T[];

  /** Index size */
  size(): number;

  /** Check if index is empty */
  isEmpty(): boolean;
}

/**
 * Searchable interface - items that can be searched
 */
export interface Searchable {
  /** Unique identifier */
  id: string;

  /** Searchable text fields */
  [key: string]: any;
}

// ============================================================================
// Search Engine Interface
// ============================================================================

/**
 * Generic search engine interface
 * All search providers must implement this
 */
export interface SearchEngine<T = any> {
  /** Provider name */
  readonly provider: SearchProvider;

  /** Current configuration */
  readonly config: SearchConfig<T>;

  /** Search index */
  readonly index: SearchIndex<T>;

  /** Search items */
  search(query: string, options?: Partial<SearchConfig<T>>): SearchResult<T>[];

  /** Add item to search index */
  add(item: T): void;

  /** Add multiple items */
  addAll(items: T[]): void;

  /** Remove item from index */
  remove(predicate: (item: T) => boolean): void;

  /** Update item in index */
  update(item: T): void;

  /** Clear search index */
  clear(): void;

  /** Rebuild index */
  rebuild(items: T[]): void;

  /** Get search suggestions */
  suggest?(query: string, limit?: number): SearchSuggestion[];

  /** Dispose resources */
  dispose?(): void;
}

// ============================================================================
// Search Query
// ============================================================================

/**
 * Search query with metadata
 */
export interface SearchQuery {
  /** Raw query string */
  text: string;

  /** Normalized query */
  normalized: string;

  /** Query tokens */
  tokens: string[];

  /** Query type */
  type: "simple" | "advanced" | "boolean" | "regex";

  /** Applied filters */
  filters?: Record<string, any>;

  /** Timestamp */
  timestamp: number;
}

// ============================================================================
// Search Context
// ============================================================================

/**
 * Search context for domain-specific search
 */
export interface SearchContext<T = any> {
  /** Context identifier */
  id: string;

  /** Context name */
  name: string;

  /** Search engine */
  engine: SearchEngine<T>;

  /** Context-specific config */
  config?: SearchConfig<T>;

  /** Context priority (for global search) */
  priority?: number;

  /** Whether context is active */
  active: boolean;
}

// ============================================================================
// Highlight
// ============================================================================

/**
 * Highlight style options
 */
export interface HighlightStyle {
  /** HTML tag to wrap matches */
  tag?: string;

  /** CSS class name */
  className?: string;

  /** Inline styles */
  style?: Record<string, string>;

  /** Prefix before match */
  prefix?: string;

  /** Suffix after match */
  suffix?: string;
}

/**
 * Highlighted text result
 */
export interface HighlightedText {
  /** Original text */
  original: string;

  /** Highlighted HTML */
  html: string;

  /** Text segments */
  segments: TextSegment[];

  /** Number of matches */
  matchCount: number;
}

/**
 * Text segment (matched or unmatched)
 */
export interface TextSegment {
  /** Segment text */
  text: string;

  /** Whether segment is a match */
  isMatch: boolean;

  /** Start index in original text */
  start: number;

  /** End index in original text */
  end: number;
}

// ============================================================================
// Distance Metrics
// ============================================================================

/**
 * Distance metric types for similarity calculation
 */
export type DistanceMetric = "levenshtein" | "damerau" | "hamming" | "jaro" | "cosine";

/**
 * Distance function signature
 */
export type DistanceFn = (a: string, b: string) => number;

// ============================================================================
// Filters & Transformers
// ============================================================================

/**
 * Search filter function
 */
export type SearchFilter<T> = (item: T, query: string) => boolean;

/**
 * Result transformer function
 */
export type ResultTransformer<T, R> = (result: SearchResult<T>) => R;

/**
 * Text normalizer function
 */
export type TextNormalizer = (text: string) => string;

/**
 * Tokenizer function
 */
export type Tokenizer = (text: string) => string[];

// ============================================================================
// Ranking & Scoring
// ============================================================================

/**
 * Ranking strategy
 */
export type RankingStrategy = "score" | "bm25" | "tfidf" | "custom";

/**
 * Scoring weights
 */
export interface ScoringWeights {
  /** Exact match bonus */
  exactMatch?: number;

  /** Prefix match bonus */
  prefixMatch?: number;

  /** Word boundary bonus */
  wordBoundary?: number;

  /** Consecutive match bonus */
  consecutiveMatch?: number;

  /** Field weight */
  fieldWeight?: number;

  /** Recency weight (for time-based ranking) */
  recency?: number;

  /** Popularity weight */
  popularity?: number;
}

