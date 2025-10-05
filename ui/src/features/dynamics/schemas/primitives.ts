/**
 * Primitive Component Schemas
 * Zod validation schemas for primitive components
 */

import { z } from "zod";

// ============================================================================
// Button Schema
// ============================================================================

export const buttonSchema = z.object({
  text: z.string().optional(),
  variant: z.enum(["default", "primary", "secondary", "danger", "ghost", "outline"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  fullWidth: z.boolean().optional(),
  disabled: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Input Schema
// ============================================================================

export const inputSchema = z.object({
  type: z.enum(["text", "password", "email", "number", "tel", "url", "search"]).optional(),
  placeholder: z.string().optional(),
  value: z.union([z.string(), z.number()]).optional(),
  variant: z.enum(["default", "filled", "outline", "underline"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  error: z.boolean().optional(),
  disabled: z.boolean().optional(),
  readonly: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Text Schema
// ============================================================================

export const textSchema = z.object({
  content: z.string(),
  variant: z.enum(["h1", "h2", "h3", "body", "caption", "label"]).optional(),
  weight: z.enum(["normal", "medium", "semibold", "bold"]).optional(),
  color: z.enum(["primary", "secondary", "accent", "muted", "error", "success"]).optional(),
  align: z.enum(["left", "center", "right"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Checkbox Schema
// ============================================================================

export const checkboxSchema = z.object({
  label: z.string().optional(),
  checked: z.boolean().optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  variant: z.enum(["default", "primary"]).optional(),
  disabled: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Radio Schema
// ============================================================================

export const radioSchema = z.object({
  label: z.string().optional(),
  name: z.string(),
  value: z.string(),
  size: z.enum(["small", "medium", "large"]).optional(),
  disabled: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Slider Schema
// ============================================================================

export const sliderSchema = z.object({
  min: z.number().optional(),
  max: z.number().optional(),
  step: z.number().optional(),
  value: z.number().optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  variant: z.enum(["default", "primary"]).optional(),
  disabled: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});
