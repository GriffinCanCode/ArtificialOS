/**
 * Layout Component Schemas
 * Zod validation schemas for layout components
 */

import { z } from "zod";

// ============================================================================
// Container Schema
// ============================================================================

export const containerSchema = z.object({
  layout: z.enum(["vertical", "horizontal"]).optional(),
  spacing: z.enum(["none", "small", "medium", "large"]).optional(),
  padding: z.enum(["none", "small", "medium", "large"]).optional(),
  align: z.enum(["start", "center", "end", "stretch"]).optional(),
  justify: z.enum(["start", "center", "end", "between", "around"]).optional(),
  gap: z.number().optional(),
  role: z.string().optional(),
  itemHeight: z.number().optional(),
  maxHeight: z.number().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Grid Schema
// ============================================================================

export const gridSchema = z.object({
  columns: z.enum(["1", "2", "3", "4", "5", "6"]).optional(),
  spacing: z.enum(["none", "small", "medium", "large"]).optional(),
  responsive: z.boolean().optional(),
  gap: z.number().optional(),
  itemHeight: z.number().optional(),
  maxHeight: z.number().optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// List Schema
// ============================================================================

export const listSchema = z.object({
  variant: z.enum(["default", "bordered", "striped"]).optional(),
  spacing: z.enum(["none", "small", "medium", "large"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});
