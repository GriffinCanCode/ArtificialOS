/**
 * Form Component Schemas
 * Zod validation schemas for form components
 */

import { z } from "zod";

// ============================================================================
// Select Schema
// ============================================================================

export const selectSchema = z.object({
  options: z.array(
    z.union([
      z.string(),
      z.object({
        label: z.string(),
        value: z.string(),
      }),
    ])
  ),
  value: z.string().optional(),
  variant: z.enum(["default", "filled", "outline"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  error: z.boolean().optional(),
  disabled: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Textarea Schema
// ============================================================================

export const textareaSchema = z.object({
  placeholder: z.string().optional(),
  value: z.string().optional(),
  variant: z.enum(["default", "filled", "outline"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  error: z.boolean().optional(),
  disabled: z.boolean().optional(),
  rows: z.number().optional(),
  resize: z.enum(["none", "vertical", "horizontal", "both"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});
