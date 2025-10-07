/**
 * Validation Schemas
 * Zod schemas for common input validation patterns
 */

import { z } from "zod";
import { isFuture, isPast } from "../../../core/utils/dates";

// ============================================================================
// Text Validation
// ============================================================================

export const nonEmptyString = z.string().min(1, "Cannot be empty").trim();

export const emailSchema = z.string().email("Invalid email address");

export const urlSchema = z.string().url("Invalid URL");

export const phoneSchema = z.string().regex(/^\+?[\d\s\-()]+$/, "Invalid phone number");

export const alphanumericSchema = z
  .string()
  .regex(/^[a-zA-Z0-9]+$/, "Only letters and numbers allowed");

export const slugSchema = z.string().regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/, "Invalid slug format");

// ============================================================================
// Number Validation
// ============================================================================

export const positiveNumber = z.number().positive("Must be positive");

export const nonNegativeNumber = z.number().min(0, "Cannot be negative");

export const percentageSchema = z.number().min(0).max(100, "Must be 0-100");

export const integerSchema = z.number().int("Must be an integer");

// ============================================================================
// Date Validation
// ============================================================================

export const dateSchema = z.date();

export const futureDateSchema = z.date().refine((date) => isFuture(date), "Must be a future date");

export const pastDateSchema = z.date().refine((date) => isPast(date), "Must be a past date");

// ============================================================================
// Form Validation
// ============================================================================

export const passwordSchema = z
  .string()
  .min(8, "Password must be at least 8 characters")
  .regex(/[A-Z]/, "Password must contain uppercase letter")
  .regex(/[a-z]/, "Password must contain lowercase letter")
  .regex(/[0-9]/, "Password must contain number");

export const usernameSchema = z
  .string()
  .min(3, "Username must be at least 3 characters")
  .max(20, "Username must be less than 20 characters")
  .regex(/^[a-zA-Z0-9_-]+$/, "Only letters, numbers, _ and - allowed");

// ============================================================================
// Session & App Validation
// ============================================================================

export const sessionNameSchema = z
  .string()
  .min(2, "Name must be at least 2 characters")
  .max(50, "Name must be less than 50 characters")
  .trim();

export const sessionDescriptionSchema = z
  .string()
  .max(200, "Description must be less than 200 characters")
  .optional();

export const appDescriptionSchema = z
  .string()
  .min(10, "Description must be at least 10 characters")
  .max(200, "Description must be less than 200 characters");

export const tagsSchema = z.array(z.string()).max(5, "Maximum 5 tags allowed");

// ============================================================================
// File Validation
// ============================================================================

export const fileNameSchema = z
  .string()
  .min(1, "File name required")
  .max(255, "File name too long")
  .regex(/^[^<>:"/\\|?*]+$/, "Invalid characters in file name");

export const filePathSchema = z
  .string()
  .min(1, "Path required")
  .regex(/^[^\0]+$/, "Invalid path");

// ============================================================================
// Compound Validation
// ============================================================================

export function createLengthSchema(min: number, max: number) {
  return z.string().min(min).max(max);
}

export function createRangeSchema(min: number, max: number) {
  return z.number().min(min).max(max);
}

export function createEnumSchema<T extends readonly [string, ...string[]]>(values: T) {
  return z.enum(values);
}

export function createArraySchema<T>(
  itemSchema: z.ZodType<T>,
  minLength?: number,
  maxLength?: number
) {
  let schema = z.array(itemSchema);
  if (minLength !== undefined) schema = schema.min(minLength);
  if (maxLength !== undefined) schema = schema.max(maxLength);
  return schema;
}
