/**
 * Special Component Schemas
 * Zod validation schemas for special-purpose components
 */

import { z } from "zod";

// ============================================================================
// AppShortcut Schema
// ============================================================================

export const appShortcutSchema = z.object({
  app_id: z.string(),
  name: z.string(),
  icon: z.string(),
  description: z.string().optional(),
  category: z.string().optional(),
  variant: z.enum(["icon", "card", "list"]).optional(),
});

export type AppShortcutProps = z.infer<typeof appShortcutSchema>;

// ============================================================================
// Iframe Schema
// ============================================================================

export const iframeSchema = z.object({
  src: z.string().url("Invalid iframe URL"),
  title: z.string().optional(),
  width: z.union([z.number(), z.string()]).optional(),
  height: z.union([z.number(), z.string()]).optional(),
  bordered: z.boolean().optional(),
  rounded: z.boolean().optional(),
  sandbox: z.string().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Progress Schema
// ============================================================================

export const progressSchema = z.object({
  value: z.number().min(0, "Progress value must be >= 0"),
  max: z.number().min(1, "Progress max must be >= 1").optional(),
  variant: z.enum(["default", "primary", "success", "error"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});
