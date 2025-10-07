/**
 * Date Parsing Utilities
 */

import { parseISO, isValid } from "date-fns";

/**
 * Parse date string safely with validation
 * Supports ISO 8601 and other common formats
 */
export function parseDate(dateString: string): Date | null {
  try {
    // Try ISO format first (most common in APIs)
    const parsed = parseISO(dateString);
    if (isValid(parsed)) {
      return parsed;
    }

    // Fallback to native Date parsing
    const fallback = new Date(dateString);
    return isValid(fallback) ? fallback : null;
  } catch {
    return null;
  }
}

/**
 * Convert Unix timestamp to Date
 */
export function parseTimestamp(timestamp: number): Date {
  return new Date(timestamp);
}
