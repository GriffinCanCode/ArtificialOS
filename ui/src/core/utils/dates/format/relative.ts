/**
 * Relative Time Formatting Utilities
 */

import { formatDistanceToNow } from "date-fns";

/**
 * Format relative time with natural language
 * Examples: "just now", "2 minutes ago", "about 3 hours ago"
 *
 * Uses date-fns formatDistanceToNow for better edge case handling
 */
export function formatRelativeTime(date: Date): string {
  const diffMs = Date.now() - date.getTime();

  // For very recent times, show "just now"
  if (diffMs < 60000) {
    return "just now";
  }

  // For times within the last day, use short format
  if (diffMs < 86400000) {
    const hours = Math.floor(diffMs / 3600000);
    const minutes = Math.floor(diffMs / 60000);

    if (hours > 0) return `${hours}h ago`;
    return `${minutes}m ago`;
  }

  // For older times, use date-fns with natural language
  return formatDistanceToNow(date, { addSuffix: true });
}

/**
 * Format timestamp as relative time
 */
export function formatTimeAgo(timestamp: number): string {
  return formatRelativeTime(new Date(timestamp));
}
