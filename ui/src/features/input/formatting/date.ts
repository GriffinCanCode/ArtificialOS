/**
 * Date Formatting Utilities
 * Pure functions for date transformation and formatting
 */

// ============================================================================
// Basic Formatting
// ============================================================================

export function formatDate(date: Date, format: string = "YYYY-MM-DD"): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");

  return format
    .replace("YYYY", String(year))
    .replace("MM", month)
    .replace("DD", day)
    .replace("HH", hours)
    .replace("mm", minutes)
    .replace("ss", seconds);
}

export function formatTime(date: Date, use24Hour: boolean = false): string {
  const hours = date.getHours();
  const minutes = String(date.getMinutes()).padStart(2, "0");

  if (use24Hour) {
    return `${String(hours).padStart(2, "0")}:${minutes}`;
  }

  const period = hours >= 12 ? "PM" : "AM";
  const displayHours = hours % 12 || 12;
  return `${displayHours}:${minutes} ${period}`;
}

export function formatDateTime(date: Date): string {
  return `${formatDate(date)} ${formatTime(date)}`;
}

// ============================================================================
// Relative Time
// ============================================================================

export function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSeconds < 60) return "just now";
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
  return `${Math.floor(diffDays / 365)}y ago`;
}

export function formatTimeAgo(timestamp: number): string {
  return formatRelativeTime(new Date(timestamp));
}

// ============================================================================
// Duration
// ============================================================================

export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  }
  return `${seconds}s`;
}

export function formatShortDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) return `${hours}h`;
  if (minutes > 0) return `${minutes}m`;
  return `${seconds}s`;
}

// ============================================================================
// Parsing
// ============================================================================

export function parseDate(dateString: string): Date | null {
  try {
    const date = new Date(dateString);
    return isNaN(date.getTime()) ? null : date;
  } catch {
    return null;
  }
}

export function parseTimestamp(timestamp: number): Date {
  return new Date(timestamp);
}

// ============================================================================
// Calculations
// ============================================================================

export function addDays(date: Date, days: number): Date {
  const result = new Date(date);
  result.setDate(result.getDate() + days);
  return result;
}

export function addHours(date: Date, hours: number): Date {
  const result = new Date(date);
  result.setHours(result.getHours() + hours);
  return result;
}

export function addMinutes(date: Date, minutes: number): Date {
  const result = new Date(date);
  result.setMinutes(result.getMinutes() + minutes);
  return result;
}

export function diffInDays(date1: Date, date2: Date): number {
  const diffMs = date2.getTime() - date1.getTime();
  return Math.floor(diffMs / (1000 * 60 * 60 * 24));
}

export function diffInHours(date1: Date, date2: Date): number {
  const diffMs = date2.getTime() - date1.getTime();
  return Math.floor(diffMs / (1000 * 60 * 60));
}

export function diffInMinutes(date1: Date, date2: Date): number {
  const diffMs = date2.getTime() - date1.getTime();
  return Math.floor(diffMs / (1000 * 60));
}

// ============================================================================
// Comparisons
// ============================================================================

export function isToday(date: Date): boolean {
  const today = new Date();
  return (
    date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
  );
}

export function isYesterday(date: Date): boolean {
  const yesterday = addDays(new Date(), -1);
  return (
    date.getDate() === yesterday.getDate() &&
    date.getMonth() === yesterday.getMonth() &&
    date.getFullYear() === yesterday.getFullYear()
  );
}

export function isFuture(date: Date): boolean {
  return date.getTime() > Date.now();
}

export function isPast(date: Date): boolean {
  return date.getTime() < Date.now();
}
