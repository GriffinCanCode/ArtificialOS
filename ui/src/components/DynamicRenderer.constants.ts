/**
 * DynamicRenderer Constants and Validation
 * Constants and validation functions for UI spec processing
 */

// ============================================================================
// Constants
// ============================================================================

// JSON size limits (in bytes) - must match backend
export const MAX_UI_SPEC_SIZE = 512 * 1024; // 512KB
export const MAX_JSON_DEPTH = 20; // Maximum nesting depth

// Virtual scrolling threshold - use virtual scrolling for lists with 50+ items
export const VIRTUAL_SCROLL_THRESHOLD = 50;
export const DEFAULT_ITEM_HEIGHT = 60; // Default height for list items in pixels

// Debounce settings
export const PARSE_DEBOUNCE_MS = 150; // Parse at most every 150ms

// ============================================================================
// Validation Functions
// ============================================================================

/**
 * Validate JSON size to prevent DoS
 */
export function validateJSONSize(jsonStr: string, maxSize: number = MAX_UI_SPEC_SIZE): void {
  const size = new Blob([jsonStr]).size;
  if (size > maxSize) {
    throw new Error(`JSON size ${size} bytes exceeds maximum ${maxSize} bytes`);
  }
}

/**
 * Validate JSON nesting depth
 */
export function validateJSONDepth(obj: any, maxDepth: number = MAX_JSON_DEPTH, currentDepth: number = 0): void {
  if (currentDepth > maxDepth) {
    throw new Error(`JSON nesting depth ${currentDepth} exceeds maximum ${maxDepth}`);
  }
  
  if (typeof obj === 'object' && obj !== null) {
    if (Array.isArray(obj)) {
      obj.forEach(item => validateJSONDepth(item, maxDepth, currentDepth + 1));
    } else {
      Object.values(obj).forEach(value => validateJSONDepth(value, maxDepth, currentDepth + 1));
    }
  }
}

