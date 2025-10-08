/**
 * Search Module Tests
 * Tests for search engine, indexing, and spotlight functionality
 */

import { describe, it, expect, beforeEach } from "vitest";
import { createEngine } from "../../src/features/search/engine/engine";
import { createMemoryIndex } from "../../src/features/search/utils/index";
import { highlight, highlightQuery } from "../../src/features/search/features/highlight";

interface TestItem {
  id: string;
  name: string;
  description: string;
  category: string;
}

describe("Search Engine", () => {
  const testItems: TestItem[] = [
    { id: "1", name: "MacBook Pro", description: "Powerful laptop", category: "computers" },
    { id: "2", name: "iPhone 15", description: "Latest smartphone", category: "phones" },
    { id: "3", name: "iPad Air", description: "Tablet device", category: "tablets" },
    { id: "4", name: "AirPods Pro", description: "Wireless earbuds", category: "audio" },
  ];

  it("should search items with fuzzy matching", () => {
    const engine = createEngine(testItems, {
      keys: [{ name: "name", weight: 1 }],
      threshold: 0.3,
    });

    const results = engine.search("macbook");
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].item.name).toBe("MacBook Pro");
  });

  it("should search across multiple fields", () => {
    const engine = createEngine(testItems, {
      keys: [
        { name: "name", weight: 0.7 },
        { name: "description", weight: 0.3 },
      ],
      threshold: 0.4,
    });

    const results = engine.search("laptop");
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].item.id).toBe("1");
  });

  it("should return scored results", () => {
    const engine = createEngine(testItems, {
      keys: [{ name: "name", weight: 1 }],
      includeScore: true,
    });

    const results = engine.search("iphone");
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].score).toBeDefined();
    expect(results[0].score).toBeGreaterThanOrEqual(0);
    expect(results[0].score).toBeLessThanOrEqual(1);
  });

  it("should limit results", () => {
    const engine = createEngine(testItems, {
      keys: [{ name: "category", weight: 1 }],
    });

    const results = engine.search("", { limit: 2 });
    expect(results.length).toBeLessThanOrEqual(2);
  });

  it("should provide autocomplete suggestions", () => {
    const engine = createEngine(testItems, {
      keys: [{ name: "name", weight: 1 }],
    });

    const suggestions = engine.suggest("ip", 3);
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions[0].type).toBe("completion");
  });

  it("should handle empty query", () => {
    const engine = createEngine(testItems, {
      keys: [{ name: "name", weight: 1 }],
    });

    const results = engine.search("");
    expect(results.length).toBe(testItems.length);
  });
});

describe("Memory Index", () => {
  it("should add items", () => {
    const index = createMemoryIndex<TestItem>();
    const item = { id: "1", name: "Test", description: "Test item", category: "test" };

    index.add(item);
    expect(index.size()).toBe(1);
    expect(index.getAll()).toContainEqual(item);
  });

  it("should add multiple items", () => {
    const index = createMemoryIndex<TestItem>();
    const items = [
      { id: "1", name: "Test 1", description: "", category: "test" },
      { id: "2", name: "Test 2", description: "", category: "test" },
    ];

    index.addAll(items);
    expect(index.size()).toBe(2);
  });

  it("should remove items by predicate", () => {
    const index = createMemoryIndex<TestItem>();
    index.addAll([
      { id: "1", name: "Test 1", description: "", category: "keep" },
      { id: "2", name: "Test 2", description: "", category: "remove" },
    ]);

    index.remove((item) => item.category === "remove");
    expect(index.size()).toBe(1);
    expect(index.getAll()[0].category).toBe("keep");
  });

  it("should update items", () => {
    const index = createMemoryIndex<TestItem>();
    const item = { id: "1", name: "Test", description: "Original", category: "test" };
    index.add(item);

    const updated = { ...item, description: "Updated" };
    index.update(updated);

    expect(index.getAll()[0].description).toBe("Updated");
  });

  it("should clear all items", () => {
    const index = createMemoryIndex<TestItem>();
    index.addAll([
      { id: "1", name: "Test 1", description: "", category: "test" },
      { id: "2", name: "Test 2", description: "", category: "test" },
    ]);

    index.clear();
    expect(index.size()).toBe(0);
    expect(index.isEmpty()).toBe(true);
  });
});

describe("Text Highlighting", () => {
  it("should highlight matches", () => {
    const result = highlight("Hello World", [[0, 4]]);
    expect(result.matchCount).toBe(1);
    expect(result.html).toContain("<mark");
    expect(result.html).toContain("Hello");
  });

  it("should highlight multiple matches", () => {
    const result = highlight("test test test", [
      [0, 3],
      [5, 8],
      [10, 13],
    ]);
    expect(result.matchCount).toBe(3);
  });

  it("should merge overlapping indices", () => {
    const result = highlight("overlapping", [
      [0, 4],
      [3, 7],
    ]);
    expect(result.matchCount).toBe(1); // Merged into one
  });

  it("should highlight query matches", () => {
    const result = highlightQuery("world", "Hello World");
    expect(result.matchCount).toBe(1);
    expect(result.html.toLowerCase()).toContain("world");
  });

  it("should handle no matches", () => {
    const result = highlightQuery("xyz", "Hello World");
    expect(result.matchCount).toBe(0);
    expect(result.html).toBe("Hello World");
  });

  it("should return text segments", () => {
    const result = highlight("Hello World", [[6, 10]]);
    expect(result.segments.length).toBe(2);
    expect(result.segments[0].isMatch).toBe(false);
    expect(result.segments[0].text).toBe("Hello ");
    expect(result.segments[1].isMatch).toBe(true);
    expect(result.segments[1].text).toBe("World");
  });
});

describe("Search Integration", () => {
  it("should rebuild index dynamically", () => {
    const items = [{ id: "1", name: "Test", description: "", category: "test" }];
    const engine = createEngine(items, {
      keys: [{ name: "name", weight: 1 }],
    });

    let results = engine.search("test");
    expect(results.length).toBe(1);

    // Add more items
    engine.add({ id: "2", name: "Another Test", description: "", category: "test" });

    results = engine.search("test");
    expect(results.length).toBe(2);
  });

  it("should handle case insensitive search by default", () => {
    const engine = createEngine(
      [{ id: "1", name: "TEST", description: "", category: "test" }],
      {
        keys: [{ name: "name", weight: 1 }],
        caseSensitive: false,
      }
    );

    const results = engine.search("test");
    expect(results.length).toBe(1);
  });

  it("should respect threshold settings", () => {
    const engine = createEngine(
      [
        { id: "1", name: "exact", description: "", category: "test" },
        { id: "2", name: "approximately", description: "", category: "test" },
      ],
      {
        keys: [{ name: "name", weight: 1 }],
        threshold: 0.1, // Strict matching
      }
    );

    const results = engine.search("exact");
    expect(results.length).toBe(1);
    expect(results[0].item.name).toBe("exact");
  });
});

