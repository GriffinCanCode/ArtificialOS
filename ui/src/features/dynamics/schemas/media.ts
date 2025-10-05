/**
 * Media Component Schemas
 * Zod validation schemas for media components
 */

import { z } from "zod";

// ============================================================================
// Image Schema
// ============================================================================

export const imageSchema = z.object({
  src: z.string().url("Invalid image URL"),
  alt: z.string().optional(),
  width: z.union([z.number(), z.string()]).optional(),
  height: z.union([z.number(), z.string()]).optional(),
  fit: z.enum(["cover", "contain", "fill", "none"]).optional(),
  rounded: z.enum(["none", "small", "medium", "large", "full"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Video Schema
// ============================================================================

export const videoSchema = z.object({
  src: z.string().url("Invalid video URL"),
  controls: z.boolean().optional(),
  autoPlay: z.boolean().optional(),
  loop: z.boolean().optional(),
  muted: z.boolean().optional(),
  width: z.union([z.number(), z.string()]).optional(),
  height: z.union([z.number(), z.string()]).optional(),
  fit: z.enum(["cover", "contain", "fill"]).optional(),
  rounded: z.enum(["none", "small", "medium", "large"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Audio Schema
// ============================================================================

export const audioSchema = z.object({
  src: z.string().url("Invalid audio URL"),
  controls: z.boolean().optional(),
  autoPlay: z.boolean().optional(),
  loop: z.boolean().optional(),
  variant: z.enum(["default", "minimal"]).optional(),
  style: z.record(z.string(), z.any()).optional(),
});

// ============================================================================
// Canvas Schema
// ============================================================================

export const canvasSchema = z.object({
  width: z.number().optional(),
  height: z.number().optional(),
  bordered: z.boolean().optional(),
  onMount: z.boolean().optional(),
  style: z.record(z.string(), z.any()).optional(),
});
