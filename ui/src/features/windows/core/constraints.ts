/**
 * Constraints Core
 * Pure functions for window constraints and validation
 */

import type { Size, Bounds, Constraints } from "./types";
import { DEFAULT_CONSTRAINTS } from "./types";

/**
 * Apply constraints to size
 */
export function constrainSize(size: Size, constraints: Constraints = DEFAULT_CONSTRAINTS): Size {
  return {
    width: Math.max(
      constraints.minWidth,
      constraints.maxWidth ? Math.min(size.width, constraints.maxWidth) : size.width
    ),
    height: Math.max(
      constraints.minHeight,
      constraints.maxHeight ? Math.min(size.height, constraints.maxHeight) : size.height
    ),
  };
}

/**
 * Validate size against constraints
 */
export function isValidSize(size: Size, constraints: Constraints = DEFAULT_CONSTRAINTS): boolean {
  if (size.width < constraints.minWidth || size.height < constraints.minHeight) {
    return false;
  }

  if (constraints.maxWidth && size.width > constraints.maxWidth) {
    return false;
  }

  if (constraints.maxHeight && size.height > constraints.maxHeight) {
    return false;
  }

  return true;
}

/**
 * Apply constraints to bounds
 */
export function constrainBoundsWithLimits(
  bounds: Bounds,
  constraints: Constraints = DEFAULT_CONSTRAINTS
): Bounds {
  return {
    position: bounds.position,
    size: constrainSize(bounds.size, constraints),
  };
}
