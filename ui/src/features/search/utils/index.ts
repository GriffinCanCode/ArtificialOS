/**
 * In-Memory Search Index
 * Fast O(1) lookups with automatic updates
 */

import type { SearchIndex, Searchable } from "../core/types";

/**
 * Memory-based search index implementation
 */
export class MemoryIndex<T extends Searchable> implements SearchIndex<T> {
  private items: Map<string, T>;
  private version: number;

  constructor() {
    this.items = new Map();
    this.version = 0;
  }

  add(item: T): void {
    this.items.set(item.id, item);
    this.version++;
  }

  addAll(items: T[]): void {
    for (const item of items) {
      this.items.set(item.id, item);
    }
    this.version++;
  }

  remove(predicate: (item: T) => boolean): void {
    const toRemove: string[] = [];
    for (const [id, item] of this.items) {
      if (predicate(item)) {
        toRemove.push(id);
      }
    }
    for (const id of toRemove) {
      this.items.delete(id);
    }
    if (toRemove.length > 0) {
      this.version++;
    }
  }

  update(item: T): void {
    if (this.items.has(item.id)) {
      this.items.set(item.id, item);
      this.version++;
    }
  }

  clear(): void {
    this.items.clear();
    this.version++;
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

  getVersion(): number {
    return this.version;
  }
}

/**
 * Create a new memory index
 */
export function createMemoryIndex<T extends Searchable>(): SearchIndex<T> {
  return new MemoryIndex<T>();
}
