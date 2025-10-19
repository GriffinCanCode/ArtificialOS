/**
 * Component Validation Utilities
 * Zod schema validation helpers for component props
 */

import { z } from "zod";
import { logger } from "../../../core/monitoring/core/logger";

// ============================================================================
// Validation Types
// ============================================================================

export interface ValidationResult<T = any> {
  success: boolean;
  data?: T;
  errors?: z.ZodError;
}

// ============================================================================
// Validation Functions
// ============================================================================

/**
 * Validate component props against a Zod schema
 *
 * @param props - Component props to validate
 * @param schema - Zod schema to validate against
 * @param componentType - Component type for logging
 * @param strict - If true, throw on validation error (default: false)
 * @returns Validation result with parsed data or errors
 */
export function validateComponentProps<T>(
  props: any,
  schema: z.ZodSchema<T>,
  componentType: string,
  strict: boolean = false
): ValidationResult<T> {
  try {
    const parsed = schema.parse(props);
    return {
      success: true,
      data: parsed,
    };
  } catch (error) {
    if (error instanceof z.ZodError) {
      if (process.env.NODE_ENV === "development") {
        logger.warn("Component props validation failed", {
          component: "ComponentValidator",
          componentType,
          errors: error.issues,
          props,
        });
      }

      if (strict) {
        throw new Error(
          `Invalid props for ${componentType}: ${error.issues
            .map((e) => `${e.path.join(".")}: ${e.message}`)
            .join(", ")}`
        );
      }

      return {
        success: false,
        errors: error,
      };
    }

    // Unexpected error
    logger.error("Unexpected validation error", error as Error, {
      component: "ComponentValidator",
      componentType,
    });

    return {
      success: false,
    };
  }
}

/**
 * Safe parse that returns partial data even on validation error
 * Useful for graceful degradation
 *
 * @param props - Component props to validate
 * @param schema - Zod schema to validate against
 * @returns Parsed data (original props if validation fails)
 */
export function safeParseProps<T>(props: any, schema: z.ZodSchema<T>): T {
  const result = schema.safeParse(props);
  return result.success ? result.data : props;
}

/**
 * Get human-readable validation errors
 *
 * @param error - Zod error object
 * @returns Array of formatted error messages
 */
export function formatValidationErrors(error: z.ZodError): string[] {
  return error.issues.map((e) => {
    const path = e.path.length > 0 ? `${e.path.join(".")}: ` : "";
    return `${path}${e.message}`;
  });
}
