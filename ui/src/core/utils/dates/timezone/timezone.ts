/**
 * Timezone Utilities
 * Powered by date-fns-tz for robust timezone handling
 */

import {
  formatInTimeZone,
  toZonedTime,
  fromZonedTime,
  getTimezoneOffset,
} from "date-fns-tz";

/**
 * Format a date in a specific timezone
 *
 * @param date - Date to format
 * @param timeZone - IANA timezone (e.g., "America/New_York", "Europe/London")
 * @param formatString - date-fns format string
 *
 * @example
 * formatInTimezone(new Date(), "America/New_York", "yyyy-MM-dd HH:mm zzz")
 * // => "2024-03-15 09:30 EDT"
 */
export function formatInTimezone(
  date: Date,
  timeZone: string,
  formatString: string = "yyyy-MM-dd HH:mm zzz"
): string {
  return formatInTimeZone(date, timeZone, formatString);
}

/**
 * Convert a date to a specific timezone
 *
 * @param date - UTC date
 * @param timeZone - Target timezone
 * @returns Date object representing the same moment in the target timezone
 *
 * @example
 * const utcDate = new Date("2024-03-15T12:00:00Z");
 * const nyTime = convertToTimezone(utcDate, "America/New_York");
 */
export function convertToTimezone(date: Date, timeZone: string): Date {
  return toZonedTime(date, timeZone);
}

/**
 * Convert a zoned time to UTC
 *
 * @param date - Date in specific timezone
 * @param timeZone - Source timezone
 * @returns UTC Date object
 *
 * @example
 * const nyDate = new Date("2024-03-15T12:00:00"); // Treated as NY time
 * const utcDate = convertFromTimezone(nyDate, "America/New_York");
 */
export function convertFromTimezone(date: Date, timeZone: string): Date {
  return fromZonedTime(date, timeZone);
}

/**
 * Get timezone offset in milliseconds
 *
 * @param timeZone - IANA timezone
 * @param date - Date to check offset for (defaults to now)
 * @returns Offset in milliseconds
 */
export function getTimezoneOffsetMs(timeZone: string, date: Date = new Date()): number {
  return getTimezoneOffset(timeZone, date);
}

/**
 * Get timezone offset as string (e.g., "-05:00", "+01:00")
 */
export function getTimezoneOffsetString(timeZone: string, date: Date = new Date()): string {
  const offsetMs = getTimezoneOffset(timeZone, date);
  const offsetMinutes = Math.abs(offsetMs / 60000);
  const hours = Math.floor(offsetMinutes / 60);
  const minutes = offsetMinutes % 60;
  const sign = offsetMs < 0 ? "-" : "+";
  return `${sign}${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`;
}

/**
 * Format date with user's local timezone
 */
export function formatLocal(date: Date, formatString: string = "yyyy-MM-dd HH:mm"): string {
  const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  return formatInTimeZone(date, userTimeZone, formatString);
}

/**
 * Get current date in specific timezone
 *
 * @example
 * const nyTime = getCurrentInTimezone(TIMEZONES.EST);
 * const londonTime = getCurrentInTimezone(TIMEZONES.GMT);
 */
export function getCurrentInTimezone(timeZone: string): Date {
  return convertToTimezone(new Date(), timeZone);
}

/**
 * Format current time in multiple timezones
 *
 * @example
 * formatInMultipleTimezones(new Date(), ["America/New_York", "Europe/London", "Asia/Tokyo"])
 * // => {
 * //   "America/New_York": "2024-03-15 09:30 EDT",
 * //   "Europe/London": "2024-03-15 13:30 GMT",
 * //   "Asia/Tokyo": "2024-03-15 22:30 JST"
 * // }
 */
export function formatInMultipleTimezones(
  date: Date,
  timeZones: string[],
  formatString: string = "yyyy-MM-dd HH:mm zzz"
): Record<string, string> {
  return timeZones.reduce(
    (acc, tz) => {
      acc[tz] = formatInTimeZone(date, tz, formatString);
      return acc;
    },
    {} as Record<string, string>
  );
}
