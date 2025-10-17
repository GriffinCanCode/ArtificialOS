/**
 * ID Generation Utilities - Sequential Numbering Only
 *
 * For ID generation, use ../id/index.ts which provides ULID-based IDs:
 * - newAppID(), newWindowID(), newSessionID(), etc.
 * - generateRaw(), generatePrefixed(), etc.
 *
 * This module only provides sequential numbering utilities.
 */

/**
 * Generates a sequential ID with counter
 * Useful for ordered lists where relative order matters
 *
 * @param prefix - Identifier prefix
 * @param counter - Sequential counter value
 *
 * Example: generateSequentialId("item", 5) => "item-00005"
 */
export function generateSequentialId(prefix: string, counter: number): string {
  const paddedCounter = counter.toString().padStart(5, "0");
  return `${prefix}-${paddedCounter}`;
}

/**
 * ID Generator class for managing sequential IDs
 * Useful when you need consistent sequential numbering
 */
export class IDGenerator {
  private counters: Map<string, number> = new Map();

  /**
   * Get next sequential ID for a prefix
   */
  next(prefix: string): string {
    const current = this.counters.get(prefix) ?? 0;
    const next = current + 1;
    this.counters.set(prefix, next);
    return generateSequentialId(prefix, next);
  }

  /**
   * Reset counter for a prefix
   */
  reset(prefix: string): void {
    this.counters.set(prefix, 0);
  }

  /**
   * Get current counter value without incrementing
   */
  current(prefix: string): number {
    return this.counters.get(prefix) ?? 0;
  }

  /**
   * Set counter to specific value
   */
  set(prefix: string, value: number): void {
    this.counters.set(prefix, value);
  }
}

/**
 * Default ID generator instance
 * Use this for consistent sequential numbering across your app
 */
export const defaultIDGenerator = new IDGenerator();
