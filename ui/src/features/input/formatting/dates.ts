/**
 * Date Formatting Utilities
 * Lightweight wrapper that re-exports from core/utils/dates
 *
 * For comprehensive date utilities, import from ../../../core/utils/dates directly
 */

// Basic formatting
export { formatDate, formatTime, formatDateTime } from "../../../core/utils/dates";

// Relative time
export { formatRelativeTime, formatTimeAgo } from "../../../core/utils/dates";

// Duration
export { formatDuration, formatShortDuration } from "../../../core/utils/dates";

// Parsing
export { parseDate, parseTimestamp } from "../../../core/utils/dates";

// Calculations
export { addDays, addHours, addMinutes } from "../../../core/utils/dates";
export { diffInDays, diffInHours, diffInMinutes } from "../../../core/utils/dates";

// Comparisons
export { isToday, isYesterday, isFuture, isPast } from "../../../core/utils/dates";
