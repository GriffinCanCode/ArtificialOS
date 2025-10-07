/**
 * Duration Formatting Utilities
 */

import { intervalToDuration } from "date-fns";

/**
 * Format duration in human-readable format
 * Uses date-fns formatDuration for proper pluralization
 */
export function formatDuration(milliseconds: number): string {
  const duration = intervalToDuration({ start: 0, end: milliseconds });

  // Custom short format for consistency with existing UI
  const { hours = 0, minutes = 0, seconds = 0 } = duration;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }
  return `${seconds}s`;
}

/**
 * Format duration in shortest possible format
 */
export function formatShortDuration(milliseconds: number): string {
  const duration = intervalToDuration({ start: 0, end: milliseconds });
  const { hours = 0, minutes = 0, seconds = 0 } = duration;

  if (hours > 0) return `${hours}h`;
  if (minutes > 0) return `${minutes}m`;
  return `${seconds}s`;
}
