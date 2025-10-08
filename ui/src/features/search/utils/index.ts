/**
 * Search Index Implementation
 * In-memory index for searchable items
 */

import type { SearchIndex, Searchable } from "../core/types";

// ============================================================================
// Memory Index
// ============================================================================

class MemoryIndex<T extends Searchable> implements SearchIndex<T> {
  private items: Map<string, T>;

  constructor() {
    this.items = new Map();
  }

  add(item: T): void {
    this.items.set(item.id, item);
  }

  addAll(items: T[]): void {
    for (const item of items) {
      this.items.set(item.id, item);
    }
  }

  remove(predicate: (item: T) => boolean): void {
    for (const [id, item] of this.items) {
      if (predicate(item)) {
        this.items.delete(id);
      }
    }
  }

  update(item: T): void {
    this.items.set(item.id, item);
  }

  clear(): void {
    this.items.clear();
  }

  getAll(): T[] {
    return Array.from(this.items.values());
  }

  size(): number {
    return this.items.size;
  }

  isEmpty(): boolean {
    return this.items.size === 0;
  }
}

/**
 * Create in-memory index
 */
export function createMemoryIndex<T extends Searchable>(): SearchIndex<T> {
  return new MemoryIndex<T>();
}

