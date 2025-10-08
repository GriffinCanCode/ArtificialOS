/**
 * Selection Box Utilities
 * Geometric intersection detection for box selection
 */

import type { Icon, SelectionBox, PixelPosition, Bounds } from "../core/types";
import { getCellBounds, boundsIntersect } from "../core/grid";
import { DEFAULT_GRID_CONFIG } from "../core/types";

// ============================================================================
// Selection Box Geometry
// ============================================================================

/**
 * Create selection box from start and current positions
 */
export function createSelectionBox(start: PixelPosition, current: PixelPosition): SelectionBox {
  return {
    start,
    end: current,
    current,
    isActive: true,
  };
}

/**
 * Get bounds of selection box
 */
export function getSelectionBounds(box: SelectionBox): Bounds {
  const minX = Math.min(box.start.x, box.end.x);
  const minY = Math.min(box.start.y, box.end.y);
  const maxX = Math.max(box.start.x, box.end.x);
  const maxY = Math.max(box.start.y, box.end.y);

  return {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

/**
 * Check if icon intersects with selection box
 * Uses axis-aligned bounding box (AABB) intersection
 */
export function iconIntersectsBox(icon: Icon, box: SelectionBox): boolean {
  const selectionBounds = getSelectionBounds(box);
  const iconBounds = getCellBounds(icon.position, DEFAULT_GRID_CONFIG);

  return boundsIntersect(selectionBounds, iconBounds);
}

/**
 * Get all icons that intersect with selection box
 */
export function getIconsInBox(icons: Icon[], box: SelectionBox): Icon[] {
  return icons.filter((icon) => iconIntersectsBox(icon, box));
}

/**
 * Get icon IDs that intersect with selection box
 */
export function getIconIdsInBox(icons: Icon[], box: SelectionBox): string[] {
  return getIconsInBox(icons, box).map((icon) => icon.id);
}

// ============================================================================
// Selection Box Validation
// ============================================================================

/**
 * Check if selection box is valid (has minimum size)
 */
export function isValidSelectionBox(box: SelectionBox, minSize: number = 5): boolean {
  const bounds = getSelectionBounds(box);
  return bounds.width >= minSize && bounds.height >= minSize;
}

/**
 * Check if point is inside selection box
 */
export function pointInSelectionBox(point: PixelPosition, box: SelectionBox): boolean {
  const bounds = getSelectionBounds(box);
  return (
    point.x >= bounds.x &&
    point.x <= bounds.x + bounds.width &&
    point.y >= bounds.y &&
    point.y <= bounds.y + bounds.height
  );
}

// ============================================================================
// Range Selection (Shift+Click)
// ============================================================================

/**
 * Get icons in range between two icons (reading order)
 */
export function getIconsInRange(icons: Icon[], startId: string, endId: string): Icon[] {
  const startIcon = icons.find((i) => i.id === startId);
  const endIcon = icons.find((i) => i.id === endId);

  if (!startIcon || !endIcon) {
    return [];
  }

  // Sort icons by reading order (row, then column)
  const sorted = [...icons].sort((a, b) => {
    if (a.position.row !== b.position.row) {
      return a.position.row - b.position.row;
    }
    return a.position.col - b.position.col;
  });

  const startIndex = sorted.indexOf(startIcon);
  const endIndex = sorted.indexOf(endIcon);

  if (startIndex === -1 || endIndex === -1) {
    return [];
  }

  const [minIndex, maxIndex] = startIndex < endIndex ? [startIndex, endIndex] : [endIndex, startIndex];

  return sorted.slice(minIndex, maxIndex + 1);
}

/**
 * Get icon IDs in range
 */
export function getIconIdsInRange(icons: Icon[], startId: string, endId: string): string[] {
  return getIconsInRange(icons, startId, endId).map((icon) => icon.id);
}

/**
 * Get rectangular selection between two icons
 * Creates a box from top-left to bottom-right
 */
export function getRectangularSelection(icons: Icon[], startId: string, endId: string): Icon[] {
  const startIcon = icons.find((i) => i.id === startId);
  const endIcon = icons.find((i) => i.id === endId);

  if (!startIcon || !endIcon) {
    return [];
  }

  const minRow = Math.min(startIcon.position.row, endIcon.position.row);
  const maxRow = Math.max(startIcon.position.row, endIcon.position.row);
  const minCol = Math.min(startIcon.position.col, endIcon.position.col);
  const maxCol = Math.max(startIcon.position.col, endIcon.position.col);

  return icons.filter(
    (icon) =>
      icon.position.row >= minRow &&
      icon.position.row <= maxRow &&
      icon.position.col >= minCol &&
      icon.position.col <= maxCol
  );
}

// ============================================================================
// Selection Utilities
// ============================================================================

/**
 * Toggle icon in selection
 */
export function toggleSelection(selectedIds: Set<string>, iconId: string): Set<string> {
  const newSelection = new Set(selectedIds);
  if (newSelection.has(iconId)) {
    newSelection.delete(iconId);
  } else {
    newSelection.add(iconId);
  }
  return newSelection;
}

/**
 * Add icons to selection
 */
export function addToSelection(selectedIds: Set<string>, iconIds: string[]): Set<string> {
  const newSelection = new Set(selectedIds);
  for (const id of iconIds) {
    newSelection.add(id);
  }
  return newSelection;
}

/**
 * Remove icons from selection
 */
export function removeFromSelection(selectedIds: Set<string>, iconIds: string[]): Set<string> {
  const newSelection = new Set(selectedIds);
  for (const id of iconIds) {
    newSelection.delete(id);
  }
  return newSelection;
}

/**
 * Invert selection
 */
export function invertSelection(allIconIds: string[], selectedIds: Set<string>): Set<string> {
  const newSelection = new Set<string>();
  for (const id of allIconIds) {
    if (!selectedIds.has(id)) {
      newSelection.add(id);
    }
  }
  return newSelection;
}

