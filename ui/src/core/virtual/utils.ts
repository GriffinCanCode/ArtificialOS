/**
 * Virtual Scrolling Utilities
 * Helper functions for size estimation and measurements
 */

import type { SizeEstimator, AutoSizeConfig, MeasuredSize } from "./types";

// ============================================================================
// Size Estimators
// ============================================================================

/**
 * Creates a fixed size estimator
 */
export const fixedSize = (size: number): SizeEstimator => {
  return () => size;
};

/**
 * Creates a variable size estimator with fallback
 */
export const variableSize = (
  sizes: (number | undefined)[],
  defaultSize: number
): SizeEstimator => {
  return (index: number) => sizes[index] ?? defaultSize;
};

/**
 * Creates an auto-sizing estimator with bounds
 */
export const autoSize = (config: AutoSizeConfig): SizeEstimator => {
  const { minSize = 0, maxSize = Infinity, defaultSize } = config;
  return () => {
    const size = defaultSize;
    return Math.max(minSize, Math.min(maxSize, size));
  };
};

/**
 * Creates a dynamic estimator based on content
 */
export const dynamicSize = (
  measureFn: (index: number) => number,
  cache: Map<number, number> = new Map()
): SizeEstimator => {
  return (idx: number) => {
    if (cache.has(idx)) {
      return cache.get(idx)!;
    }
    const size = measureFn(idx);
    cache.set(idx, size);
    return size;
  };
};

// ============================================================================
// Measurement Helpers
// ============================================================================

/**
 * Measures element size
 */
export const measureElement = (element: HTMLElement | null): number => {
  if (!element) return 0;
  const rect = element.getBoundingClientRect();
  return rect.height;
};

/**
 * Measures element width
 */
export const measureWidth = (element: HTMLElement | null): number => {
  if (!element) return 0;
  const rect = element.getBoundingClientRect();
  return rect.width;
};

/**
 * Batch measures multiple elements
 */
export const batchMeasure = (
  elements: (HTMLElement | null)[],
  horizontal = false
): MeasuredSize[] => {
  return elements
    .map((element, index) => ({
      index,
      size: horizontal ? measureWidth(element) : measureElement(element),
    }))
    .filter((item) => item.size > 0);
};

// ============================================================================
// Grid Calculations
// ============================================================================

/**
 * Calculates grid dimensions
 */
export const calculateGridDimensions = (
  totalItems: number,
  columns: number
): { rows: number; lastRowItems: number } => {
  const rows = Math.ceil(totalItems / columns);
  const lastRowItems = totalItems % columns || columns;
  return { rows, lastRowItems };
};

/**
 * Converts flat index to grid coordinates
 */
export const indexToGrid = (
  index: number,
  columns: number
): { row: number; col: number } => {
  return {
    row: Math.floor(index / columns),
    col: index % columns,
  };
};

/**
 * Converts grid coordinates to flat index
 */
export const gridToIndex = (row: number, col: number, columns: number): number => {
  return row * columns + col;
};

// ============================================================================
// Key Generators
// ============================================================================

/**
 * Generates a stable key for virtual items
 */
export const generateKey = (index: number, prefix = "item"): string => {
  return `${prefix}-${index}`;
};

/**
 * Generates a key from item id if available
 */
export const generateItemKey = <T extends { id?: string }>(
  item: T,
  index: number,
  prefix = "item"
): string => {
  return item.id ? `${prefix}-${item.id}` : generateKey(index, prefix);
};

// ============================================================================
// Performance Helpers
// ============================================================================

/**
 * Calculates optimal overscan count based on viewport
 */
export const calculateOverscan = (
  viewportSize: number,
  itemSize: number,
  min = 3,
  max = 10
): number => {
  const itemsInView = Math.ceil(viewportSize / itemSize);
  const overscan = Math.ceil(itemsInView * 0.5);
  return Math.max(min, Math.min(max, overscan));
};

/**
 * Throttles virtualization updates
 */
export const throttleScroll = <T extends (...args: any[]) => any>(
  fn: T,
  wait = 16
): ((...args: Parameters<T>) => void) => {
  let timeout: NodeJS.Timeout | null = null;
  let lastRan = 0;

  return (...args: Parameters<T>) => {
    const now = Date.now();

    if (timeout) {
      clearTimeout(timeout);
    }

    if (now - lastRan >= wait) {
      fn(...args);
      lastRan = now;
    } else {
      timeout = setTimeout(() => {
        fn(...args);
        lastRan = Date.now();
      }, wait - (now - lastRan));
    }
  };
};
