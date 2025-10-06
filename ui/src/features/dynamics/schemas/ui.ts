/**
 * UI Component Schemas
 * Zod validation schemas for UI components
 */

import { z } from "zod";

// ============================================================================
// Badge Schema
// ============================================================================

export const badgeSchema = z.object({
  text: z.string().optional(),
  content: z.string().optional(),
  variant: z.enum(["default", "primary", "secondary", "success", "error", "warning"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Card Schema
// ============================================================================

export const cardSchema = z.object({
  title: z.string().optional(),
  footer: z.string().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Divider Schema
// ============================================================================

export const dividerSchema = z.object({
  orientation: z.enum(["horizontal", "vertical"]).optional(),
  variant: z.enum(["solid", "dashed", "dotted"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Modal Schema
// ============================================================================

export const modalSchema = z.object({
  title: z.string().optional(),
  open: z.boolean().optional(),
  size: z.enum(["small", "medium", "large", "fullscreen"]).optional(),
  centered: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Tabs Schema
// ============================================================================

export const tabsSchema = z.object({
  defaultTab: z.string().optional(),
  variant: z.enum(["default", "bordered", "pills"]).optional(),
  size: z.enum(["small", "medium", "large"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});
