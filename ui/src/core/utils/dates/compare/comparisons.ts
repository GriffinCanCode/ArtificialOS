/**
 * Date Comparison Utilities
 */

import {
  isToday as dateFnsIsToday,
  isYesterday as dateFnsIsYesterday,
  isFuture as dateFnsIsFuture,
  isPast as dateFnsIsPast,
} from "date-fns";

/**
 * Check if date is today
 */
export function isToday(date: Date): boolean {
  return dateFnsIsToday(date);
}

/**
 * Check if date is yesterday
 */
export function isYesterday(date: Date): boolean {
  return dateFnsIsYesterday(date);
}

/**
 * Check if date is in the future
 */
export function isFuture(date: Date): boolean {
  return dateFnsIsFuture(date);
}

/**
 * Check if date is in the past
 */
export function isPast(date: Date): boolean {
  return dateFnsIsPast(date);
}

// ============================================================================
// Sorting Helpers
// ============================================================================

/**
 * Compare two dates for sorting (descending - newest first)
 * Use in array.sort() to sort dates from newest to oldest
 *
 * @example
 * sessions.sort((a, b) => compareDatesDesc(new Date(a.updated_at), new Date(b.updated_at)))
 */
export function compareDatesDesc(a: Date, b: Date): number {
  return b.getTime() - a.getTime();
}

/**
 * Compare two dates for sorting (ascending - oldest first)
 * Use in array.sort() to sort dates from oldest to newest
 *
 * @example
 * events.sort((a, b) => compareDatesAsc(new Date(a.created_at), new Date(b.created_at)))
 */
export function compareDatesAsc(a: Date, b: Date): number {
  return a.getTime() - b.getTime();
}

/**
 * Compare two Unix timestamps for sorting (descending - newest first)
 * Use in array.sort() to sort timestamps from newest to oldest
 *
 * @example
 * notes.sort((a, b) => compareTimestampsDesc(a.updatedAt, b.updatedAt))
 */
export function compareTimestampsDesc(a: number, b: number): number {
  return b - a;
}

/**
 * Compare two Unix timestamps for sorting (ascending - oldest first)
 * Use in array.sort() to sort timestamps from oldest to newest
 *
 * @example
 * messages.sort((a, b) => compareTimestampsAsc(a.timestamp, b.timestamp))
 */
export function compareTimestampsAsc(a: number, b: number): number {
  return a - b;
}
