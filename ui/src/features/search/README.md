# Search Module

Centralized search system powering fuzzy search, autocomplete, filtering, and global Spotlight functionality across all features.

## Architecture

```
search/
├── core/              # Types and factory
│   ├── types.ts       # TypeScript definitions
│   └── engine.ts      # Engine factory
├── engine/            # Search implementation
│   └── engine.ts      # Unified search engine (Fuse + MiniSearch)
├── features/          # Search features
│   ├── highlight.ts   # Text highlighting
│   └── filter.ts      # Result filtering & refinement
├── utils/             # Utilities
│   ├── index.ts       # In-memory index
│   └── normalize.ts   # Text normalization & distance metrics
├── hooks/             # React hooks
│   └── useSearch.ts   # Search hook
├── store/             # Global search store
│   └── store.ts       # Zustand store for Spotlight
└── index.ts           # Public exports
```

## Design Philosophy

**Unified Implementation**: Single search engine leveraging best-in-class libraries:
- **Fuse.js** - Primary fuzzy matching (best accuracy)
- **MiniSearch** - Autocomplete suggestions (optimized for prefix search)

**Extensible**: Easy to add new search contexts for different features
**Performant**: O(1) indexing, efficient fuzzy matching algorithms
**Type-Safe**: Full TypeScript support with generic interfaces

## Features

### 🔍 Core Search
- **Fuzzy matching** - Find items by approximate matches
- **Weighted fields** - Prioritize certain fields in ranking
- **Score-based ranking** - Sort by relevance
- **Configurable thresholds** - Control match strictness

### 🎯 Autocomplete
- **Prefix matching** - Real-time suggestions
- **Smart suggestions** - Context-aware completions
- **Fallback handling** - Graceful degradation

### 🎨 Text Highlighting
- **Match highlighting** - Visual emphasis on matches
- **Custom styles** - Configurable appearance
- **Excerpt generation** - Context around matches
- **Overlapping merge** - Clean highlight rendering

### 🔧 Advanced Features
- **Multi-field search** - Search across multiple properties
- **Result filtering** - Score, date, field-based filters
- **Deduplication** - Remove duplicate results
- **Grouping** - Organize results by field
- **Pagination** - Efficient result navigation

### 🌟 Global Search (Spotlight)
- **Multi-context search** - Search across all features
- **Priority-based** - Important contexts first
- **Real-time updates** - Dynamic result refresh
- **Context management** - Enable/disable search domains

## Usage

### Basic Search

```typescript
import { createEngine } from "@/features/search";

interface Product {
  id: string;
  name: string;
  description: string;
  category: string;
}

const products: Product[] = [
  { id: "1", name: "MacBook Pro", description: "Powerful laptop", category: "computers" },
  { id: "2", name: "iPhone 15", description: "Latest smartphone", category: "phones" },
];

// Create search engine
const engine = createEngine(products, {
  keys: [
    { name: "name", weight: 0.7 },
    { name: "description", weight: 0.2 },
    { name: "category", weight: 0.1 },
  ],
  threshold: 0.3,
});

// Search
const results = engine.search("macbook");
// => [{ item: { id: "1", name: "MacBook Pro", ... }, score: 0.02 }]

// Autocomplete
const suggestions = engine.suggest("iph", 5);
// => [{ text: "iPhone 15", score: 0.05, type: "completion" }]
```

### React Hook

```typescript
import { useSearch } from "@/features/search";

function ProductSearch() {
  const { query, setQuery, results, isSearching, count, clear } = useSearch({
    items: products,
    keys: [{ name: "name", weight: 1 }],
    debounce: 300, // Debounce search by 300ms
  });

  return (
    <div>
      <input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search products..."
      />
      {isSearching && <span>{count} results</span>}
      <button onClick={clear}>Clear</button>
      
      {results.map((result) => (
        <div key={result.item.id}>
          {result.item.name} - Score: {result.score.toFixed(2)}
        </div>
      ))}
    </div>
  );
}
```

### Text Highlighting

```typescript
import { highlight, highlightQuery } from "@/features/search";

// Highlight specific indices
const result = highlight("Hello World", [[0, 4]], {
  tag: "mark",
  className: "highlight",
});
// => { html: "<mark>Hello</mark> World", segments: [...], matchCount: 1 }

// Highlight query matches
const result2 = highlightQuery("world", "Hello World");
// => { html: "Hello <mark>World</mark>", segments: [...], matchCount: 1 }

// Custom styling
const result3 = highlightQuery("search", "Search this text", {
  tag: "span",
  className: "search-match",
  style: { backgroundColor: "#ffeb3b", fontWeight: "bold" },
});
```

### Filtering Results

```typescript
import { byScore, byField, and, deduplicate, groupBy, paginate } from "@/features/search";

// Filter by score
const highQuality = results.filter((r) => byScore(0.3)(r.item, query, r));

// Filter by field
const inStock = results.filter((r) => byField("inStock", (val) => val === true)(r.item, query, r));

// Combine filters
const filtered = results.filter((r) => 
  and(byScore(0.3), byField("category", (v) => v === "computers"))(r.item, query, r)
);

// Deduplicate by field
const unique = deduplicate(results, "name");

// Group by category
const grouped = groupBy(results, "category");

// Paginate
const page1 = paginate(results, 0, 10); // First 10 results
const page2 = paginate(results, 1, 10); // Next 10 results
```

### Global Search (Spotlight)

```typescript
import {
  useSearchStore,
  useSearchActions,
  useSearchResults,
  useSearchContexts,
} from "@/features/search";

function Spotlight() {
  const { setQuery, toggle, clear } = useSearchActions();
  const results = useSearchResults();
  const { registerContext } = useSearchContexts();

  // Register search contexts
  useEffect(() => {
    registerContext("files", "Files", fileItems, {
      keys: [{ name: "name", weight: 0.8 }, { name: "path", weight: 0.2 }],
    }, 10); // Priority 10

    registerContext("apps", "Applications", appItems, {
      keys: [{ name: "name", weight: 1 }],
    }, 20); // Priority 20 (higher = shown first)

    return () => {
      unregisterContext("files");
      unregisterContext("apps");
    };
  }, []);

  return (
    <div className="spotlight">
      <input
        placeholder="Search everything..."
        onChange={(e) => setQuery(e.target.value)}
        onFocus={() => toggle()}
      />
      
      {results.map(({ contextId, contextName, results }) => (
        <div key={contextId}>
          <h3>{contextName}</h3>
          {results.slice(0, 5).map((result) => (
            <div key={result.item.id}>
              {result.item.name || result.item.label}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
```

### Configuration Presets

```typescript
import { fuzzyConfig, exactConfig, autocompleteConfig } from "@/features/search";

// Fuzzy search (default)
const fuzzyEngine = createEngine(items, fuzzyConfig());

// Exact matching
const exactEngine = createEngine(items, exactConfig());

// Autocomplete optimized
const autocompleteEngine = createEngine(items, autocompleteConfig());
```

## API Reference

### SearchEngine

```typescript
interface SearchEngine<T> {
  search(query: string, options?: SearchConfig<T>): SearchResult<T>[];
  suggest(query: string, limit?: number): SearchSuggestion[];
  add(item: T): void;
  addAll(items: T[]): void;
  remove(predicate: (item: T) => boolean): void;
  update(item: T): void;
  clear(): void;
  rebuild(items: T[]): void;
  dispose(): void;
}
```

### SearchConfig

```typescript
interface SearchConfig<T> {
  keys?: SearchKey[];           // Fields to search with weights
  threshold?: number;           // 0-1, lower = stricter (default: 0.3)
  distance?: number;            // Max distance for matches (default: 100)
  minMatchLength?: number;      // Min characters to match (default: 1)
  includeScore?: boolean;       // Include relevance score (default: true)
  includeMatches?: boolean;     // Include match indices (default: false)
  ignoreLocation?: boolean;     // Ignore match position (default: true)
  caseSensitive?: boolean;      // Case sensitive search (default: false)
  limit?: number;               // Max results
}
```

### SearchResult

```typescript
interface SearchResult<T> {
  item: T;                      // Matched item
  score: number;                // Relevance (0 = perfect, 1 = worst)
  matches?: SearchMatch[];      // Match details
}
```

## Performance

- **Index creation**: O(n) where n = number of items
- **Search**: O(n) with early termination
- **Autocomplete**: O(log n) with prefix tree
- **Memory**: ~1-2KB per 100 items (depends on item size)

### Benchmarks (1000 items)

| Operation | Time |
|-----------|------|
| Index creation | ~5ms |
| Fuzzy search | ~2-3ms |
| Autocomplete | ~1ms |
| Highlight | <0.5ms |

## Integration with Icons

The icons feature is automatically integrated:

```typescript
// Icons use centralized search under the hood
import { searchIcons, filterIcons, sortByRelevance } from "@/features/icons";

const results = searchIcons("terminal", icons);
// => Uses centralized search system

// Register icons in global search
const { registerContext } = useSearchContexts();
registerContext("icons", "Desktop Icons", icons, {
  keys: [{ name: "label", weight: 1 }],
}, 15);
```

## Future Enhancements

- [ ] Semantic search using embeddings
- [ ] Search history with frequency ranking
- [ ] Boolean query operators (AND, OR, NOT)
- [ ] Regular expression support
- [ ] Phonetic matching (soundex)
- [ ] Custom tokenizers for CJK languages
- [ ] Search result caching
- [ ] Web Worker support for large datasets

## Technical Details

### Libraries

- **Fuse.js** (v7.1.0) - Fuzzy search with proven track record
- **MiniSearch** (latest) - Lightweight autocomplete engine

### Why This Approach?

1. **Single Source of Truth**: One implementation reduces duplication
2. **Best of Both Worlds**: Fuse for accuracy, MiniSearch for speed
3. **Graceful Degradation**: Autocomplete falls back to Fuse if MiniSearch fails
4. **Type Safety**: Full TypeScript generics for compile-time safety
5. **Extensible**: Easy to add new features without breaking existing code

### Distance Metrics

The module includes Levenshtein distance for advanced use cases:

```typescript
import { levenshtein, similarity, isSimilar } from "@/features/search";

const distance = levenshtein("kitten", "sitting"); // => 3
const sim = similarity("kitten", "sitting"); // => 0.57
const similar = isSimilar("hello", "helo", 0.8); // => true
```

## References

- [Fuse.js Documentation](https://fusejs.io/)
- [MiniSearch Documentation](https://lucaong.github.io/minisearch/)
- [Information Retrieval](https://nlp.stanford.edu/IR-book/)
- [Fuzzy String Matching](https://en.wikipedia.org/wiki/Approximate_string_matching)

