/**
 * Basic Date Formatting Utilities
 */

import { format as dateFnsFormat } from "date-fns";

/**
 * Format a date using date-fns format tokens
 *
 * Common formats:
 * - "yyyy-MM-dd" => "2024-03-15"
 * - "MMM d, yyyy" => "Mar 15, 2024"
 * - "EEEE, MMMM do" => "Friday, March 15th"
 * - "h:mm a" => "3:30 PM"
 *
 * @see https://date-fns.org/docs/format
 */
export function formatDate(date: Date, formatString: string = "yyyy-MM-dd"): string {
  // Map legacy format strings for backward compatibility
  const legacyFormatMap: Record<string, string> = {
    "YYYY-MM-DD": "yyyy-MM-dd",
    "YYYY": "yyyy",
    "MM": "MM",
    "DD": "dd",
    "HH": "HH",
    "mm": "mm",
    "ss": "ss",
  };

  const mappedFormat = legacyFormatMap[formatString] || formatString;
  return dateFnsFormat(date, mappedFormat);
}

/**
 * Format time in 12-hour or 24-hour format
 */
export function formatTime(date: Date, use24Hour: boolean = false): string {
  return dateFnsFormat(date, use24Hour ? "HH:mm" : "h:mm a");
}

/**
 * Format date and time together
 */
export function formatDateTime(date: Date): string {
  return dateFnsFormat(date, "yyyy-MM-dd HH:mm");
}

/**
 * Format date as ISO 8601 string
 * Useful for logging, API calls, and standardized timestamps
 */
export function formatISO(date: Date = new Date()): string {
  return date.toISOString();
}
