/**
 * Internationalization (i18n) Utilities
 */

import { format as dateFnsFormat, formatDistanceToNow } from "date-fns";
import { enUS } from "date-fns/locale";

// Locale type for better TypeScript support
type DateFnsLocale = typeof enUS;

/**
 * Format date with locale support
 *
 * @param date - Date to format
 * @param formatString - date-fns format string
 * @param locale - date-fns locale object
 *
 * @example
 * import { es, fr, de } from 'date-fns/locale';
 * formatLocalized(new Date(), "PPP", es); // "15 de marzo de 2024"
 * formatLocalized(new Date(), "PPP", fr); // "15 mars 2024"
 * formatLocalized(new Date(), "PPP", de); // "15. März 2024"
 */
export function formatLocalized(
  date: Date,
  formatString: string = "PPP",
  locale: DateFnsLocale = enUS
): string {
  return dateFnsFormat(date, formatString, { locale });
}

/**
 * Format relative time with locale support
 *
 * @example
 * import { es } from 'date-fns/locale';
 * formatRelativeLocalized(date, es); // "hace 2 horas"
 */
export function formatRelativeLocalized(date: Date, locale: DateFnsLocale = enUS): string {
  const diffMs = Date.now() - date.getTime();

  // For very recent times, use locale-aware "just now"
  if (diffMs < 60000) {
    return locale.code === "en-US" ? "just now" : formatDistanceToNow(date, { locale });
  }

  // For times within the last day, use short format
  if (diffMs < 86400000) {
    const hours = Math.floor(diffMs / 3600000);
    const minutes = Math.floor(diffMs / 60000);

    if (hours > 0) return `${hours}h ago`;
    return `${minutes}m ago`;
  }

  return formatDistanceToNow(date, { addSuffix: true, locale });
}

/**
 * Get localized month names
 *
 * @example
 * getMonthNames(); // ["January", "February", ...]
 * getMonthNames(es); // ["enero", "febrero", ...]
 */
export function getMonthNames(locale: DateFnsLocale = enUS): string[] {
  return Array.from({ length: 12 }, (_, i) => {
    const date = new Date(2024, i, 1);
    return dateFnsFormat(date, "MMMM", { locale });
  });
}

/**
 * Get localized short month names
 *
 * @example
 * getShortMonthNames(); // ["Jan", "Feb", ...]
 */
export function getShortMonthNames(locale: DateFnsLocale = enUS): string[] {
  return Array.from({ length: 12 }, (_, i) => {
    const date = new Date(2024, i, 1);
    return dateFnsFormat(date, "MMM", { locale });
  });
}

/**
 * Get localized day names
 *
 * @example
 * getDayNames(); // ["Sunday", "Monday", ...]
 */
export function getDayNames(locale: DateFnsLocale = enUS): string[] {
  return Array.from({ length: 7 }, (_, i) => {
    const date = new Date(2024, 0, i); // Start from a Sunday
    return dateFnsFormat(date, "EEEE", { locale });
  });
}

/**
 * Get localized short day names
 *
 * @example
 * getShortDayNames(); // ["Sun", "Mon", ...]
 */
export function getShortDayNames(locale: DateFnsLocale = enUS): string[] {
  return Array.from({ length: 7 }, (_, i) => {
    const date = new Date(2024, 0, i);
    return dateFnsFormat(date, "EEE", { locale });
  });
}

// Re-export DateFnsLocale type for consumers
export type { DateFnsLocale };
