/**
 * Date Calculation Utilities
 */

import {
  addDays as dateFnsAddDays,
  addHours as dateFnsAddHours,
  addMinutes as dateFnsAddMinutes,
  differenceInDays as dateFnsDiffInDays,
  differenceInHours as dateFnsDiffInHours,
  differenceInMinutes as dateFnsDiffInMinutes,
} from "date-fns";

/**
 * Add days to a date (immutable)
 */
export function addDays(date: Date, days: number): Date {
  return dateFnsAddDays(date, days);
}

/**
 * Add hours to a date (immutable)
 */
export function addHours(date: Date, hours: number): Date {
  return dateFnsAddHours(date, hours);
}

/**
 * Add minutes to a date (immutable)
 */
export function addMinutes(date: Date, minutes: number): Date {
  return dateFnsAddMinutes(date, minutes);
}

/**
 * Calculate difference in days between two dates
 */
export function diffInDays(date1: Date, date2: Date): number {
  return dateFnsDiffInDays(date2, date1);
}

/**
 * Calculate difference in hours between two dates
 */
export function diffInHours(date1: Date, date2: Date): number {
  return dateFnsDiffInHours(date2, date1);
}

/**
 * Calculate difference in minutes between two dates
 */
export function diffInMinutes(date1: Date, date2: Date): number {
  return dateFnsDiffInMinutes(date2, date1);
}
