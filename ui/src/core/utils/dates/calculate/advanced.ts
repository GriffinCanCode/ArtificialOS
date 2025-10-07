/**
 * Advanced Date Utilities
 */

import {
  startOfWeek,
  endOfWeek,
  startOfMonth,
  endOfMonth,
  startOfYear,
  endOfYear,
  isWeekend as dateFnsIsWeekend,
  isLeapYear,
  getDaysInMonth,
  getWeek,
  getQuarter,
  addWeeks,
  addMonths,
  addYears,
  subDays,
  subWeeks,
  subMonths,
  subYears,
} from "date-fns";

/**
 * Check if a date is a weekend
 */
export function isWeekend(date: Date): boolean {
  return dateFnsIsWeekend(date);
}

/**
 * Check if a year is a leap year
 * Handles edge cases correctly (divisible by 100 but not 400, etc.)
 */
export function isLeapYearCheck(date: Date | number): boolean {
  return isLeapYear(date);
}

/**
 * Get number of days in a month (handles leap years correctly)
 */
export function getDaysInMonthCount(date: Date): number {
  return getDaysInMonth(date);
}

/**
 * Get ISO week number (1-53)
 */
export function getWeekNumber(date: Date): number {
  return getWeek(date);
}

/**
 * Get quarter (1-4)
 */
export function getQuarterNumber(date: Date): number {
  return getQuarter(date);
}

/**
 * Get start of week (Sunday by default)
 */
export function getStartOfWeek(date: Date, weekStartsOn: 0 | 1 | 2 | 3 | 4 | 5 | 6 = 0): Date {
  return startOfWeek(date, { weekStartsOn });
}

/**
 * Get end of week
 */
export function getEndOfWeek(date: Date, weekStartsOn: 0 | 1 | 2 | 3 | 4 | 5 | 6 = 0): Date {
  return endOfWeek(date, { weekStartsOn });
}

/**
 * Get start of month
 */
export function getStartOfMonth(date: Date): Date {
  return startOfMonth(date);
}

/**
 * Get end of month
 */
export function getEndOfMonth(date: Date): Date {
  return endOfMonth(date);
}

/**
 * Get start of year
 */
export function getStartOfYear(date: Date): Date {
  return startOfYear(date);
}

/**
 * Get end of year
 */
export function getEndOfYear(date: Date): Date {
  return endOfYear(date);
}

/**
 * Add weeks to a date
 */
export function addWeeksToDate(date: Date, weeks: number): Date {
  return addWeeks(date, weeks);
}

/**
 * Add months to a date (handles variable month lengths correctly)
 */
export function addMonthsToDate(date: Date, months: number): Date {
  return addMonths(date, months);
}

/**
 * Add years to a date (handles leap years correctly)
 */
export function addYearsToDate(date: Date, years: number): Date {
  return addYears(date, years);
}

/**
 * Subtract days from a date
 */
export function subtractDays(date: Date, days: number): Date {
  return subDays(date, days);
}

/**
 * Subtract weeks from a date
 */
export function subtractWeeks(date: Date, weeks: number): Date {
  return subWeeks(date, weeks);
}

/**
 * Subtract months from a date
 */
export function subtractMonths(date: Date, months: number): Date {
  return subMonths(date, months);
}

/**
 * Subtract years from a date
 */
export function subtractYears(date: Date, years: number): Date {
  return subYears(date, years);
}
