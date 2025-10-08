/**
 * Search Filtering
 * Advanced filtering and result refinement
 */

import type { SearchResult, SearchFilter, Searchable } from "../core/types";

// ============================================================================
// Filters
// ============================================================================

/**
 * Filter by score threshold
 */
export function byScore<T extends Searchable>(threshold: number): SearchFilter<T> {
  return (_item: T, _query: string, result?: SearchResult<T>) => {
    return (result?.score ?? 0) <= threshold;
  };
}

/**
 * Filter by minimum match length
 */
export function byMinLength<T extends Searchable>(minLength: number): SearchFilter<T> {
  return (_item: T, query: string) => {
    return query.length >= minLength;
  };
}

/**
 * Filter by field value
 */
export function byField<T extends Searchable>(
  field: keyof T,
  predicate: (value: any) => boolean
): SearchFilter<T> {
  return (item: T) => {
    return predicate(item[field]);
  };
}

/**
 * Filter by multiple fields (AND logic)
 */
export function byFields<T extends Searchable>(
  filters: Array<[keyof T, (value: any) => boolean]>
): SearchFilter<T> {
  return (item: T) => {
    return filters.every(([field, predicate]) => predicate(item[field]));
  };
}

/**
 * Filter by date range
 */
export function byDateRange<T extends Searchable>(
  field: keyof T,
  start?: Date,
  end?: Date
): SearchFilter<T> {
  return (item: T) => {
    const value = item[field];

    // Check if value is a Date or number (timestamp)
    let date: Date;

    if (typeof value === "number") {
      date = new Date(value);
    } else if (value && typeof value === "object" && "getTime" in value) {
      // Duck-typing check for Date-like objects
      date = value as Date;
    } else {
      return false;
    }

    if (start && date < start) return false;
    if (end && date > end) return false;

    return true;
  };
}

/**
 * Combine filters with AND logic
 */
export function and<T extends Searchable>(...filters: SearchFilter<T>[]): SearchFilter<T> {
  return (item: T, query: string) => {
    return filters.every((filter) => filter(item, query));
  };
}

/**
 * Combine filters with OR logic
 */
export function or<T extends Searchable>(...filters: SearchFilter<T>[]): SearchFilter<T> {
  return (item: T, query: string) => {
    return filters.some((filter) => filter(item, query));
  };
}

/**
 * Negate filter
 */
export function not<T extends Searchable>(filter: SearchFilter<T>): SearchFilter<T> {
  return (item: T, query: string) => {
    return !filter(item, query);
  };
}

// ============================================================================
// Result Refinement
// ============================================================================

/**
 * Deduplicate results by field
 */
export function deduplicate<T extends Searchable>(
  results: SearchResult<T>[],
  field: keyof T = "id" as keyof T
): SearchResult<T>[] {
  const seen = new Set();
  return results.filter((result) => {
    const value = result.item[field];
    if (seen.has(value)) {
      return false;
    }
    seen.add(value);
    return true;
  });
}

/**
 * Group results by field
 */
export function groupBy<T extends Searchable>(
  results: SearchResult<T>[],
  field: keyof T
): Map<any, SearchResult<T>[]> {
  const groups = new Map<any, SearchResult<T>[]>();

  for (const result of results) {
    const key = result.item[field];
    const group = groups.get(key) || [];
    group.push(result);
    groups.set(key, group);
  }

  return groups;
}

/**
 * Take top N results
 */
export function take<T extends Searchable>(results: SearchResult<T>[], limit: number): SearchResult<T>[] {
  return results.slice(0, limit);
}

/**
 * Skip first N results
 */
export function skip<T extends Searchable>(results: SearchResult<T>[], offset: number): SearchResult<T>[] {
  return results.slice(offset);
}

/**
 * Paginate results
 */
export function paginate<T extends Searchable>(
  results: SearchResult<T>[],
  page: number,
  pageSize: number
): SearchResult<T>[] {
  const start = page * pageSize;
  return results.slice(start, start + pageSize);
}

