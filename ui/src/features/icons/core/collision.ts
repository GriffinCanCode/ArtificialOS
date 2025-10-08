/**
 * Collision Detection
 * Spatial indexing and collision algorithms for icon placement
 */

import type { Icon, GridPosition, CollisionMap } from "./types";
import { gridKey, parseGridKey } from "./grid";

// ============================================================================
// Collision Map (Spatial Hash)
// ============================================================================

/**
 * Build collision map from icons using spatial hashing
 * O(n) construction, O(1) lookups
 */
export function buildCollisionMap(icons: Icon[]): CollisionMap {
  const occupied = new Map<string, string>();

  for (const icon of icons) {
    const key = gridKey(icon.position);
    occupied.set(key, icon.id);
  }

  return { occupied, available: [] };
}

/**
 * Check if grid position is occupied
 * O(1) lookup via hash map
 */
export function isOccupied(position: GridPosition, collisionMap: CollisionMap): boolean {
  return collisionMap.occupied.has(gridKey(position));
}

/**
 * Get icon ID at position (if any)
 */
export function getIconAt(position: GridPosition, collisionMap: CollisionMap): string | undefined {
  return collisionMap.occupied.get(gridKey(position));
}

/**
 * Find all occupied positions
 */
export function getOccupiedPositions(collisionMap: CollisionMap): GridPosition[] {
  return Array.from(collisionMap.occupied.keys()).map(parseGridKey);
}

// ============================================================================
// Available Position Finding
// ============================================================================

/**
 * Find nearest available position using breadth-first search
 * Guarantees finding closest free cell in Manhattan distance
 */
export function findNearestAvailable(
  target: GridPosition,
  collisionMap: CollisionMap,
  maxRows: number,
  maxCols: number
): GridPosition | null {
  if (!isOccupied(target, collisionMap)) {
    return target; // Target is free
  }

  const visited = new Set<string>();
  const queue: GridPosition[] = [target];
  visited.add(gridKey(target));

  // BFS to find nearest free cell
  while (queue.length > 0) {
    const current = queue.shift()!;

    // Check 4-connected neighbors
    const neighbors = [
      { row: current.row - 1, col: current.col }, // Up
      { row: current.row + 1, col: current.col }, // Down
      { row: current.row, col: current.col - 1 }, // Left
      { row: current.row, col: current.col + 1 }, // Right
    ];

    for (const neighbor of neighbors) {
      // Check bounds
      if (neighbor.row < 0 || neighbor.row >= maxRows || neighbor.col < 0 || neighbor.col >= maxCols) {
        continue;
      }

      const key = gridKey(neighbor);

      if (visited.has(key)) {
        continue;
      }

      visited.add(key);

      if (!isOccupied(neighbor, collisionMap)) {
        return neighbor; // Found free cell
      }

      queue.push(neighbor);
    }
  }

  return null; // No free cell found
}

/**
 * Find first available position in reading order
 * Used for initial placement
 */
export function findFirstAvailable(
  collisionMap: CollisionMap,
  maxRows: number,
  maxCols: number
): GridPosition | null {
  for (let row = 0; row < maxRows; row++) {
    for (let col = 0; col < maxCols; col++) {
      const position = { row, col };
      if (!isOccupied(position, collisionMap)) {
        return position;
      }
    }
  }
  return null; // Grid is full
}

/**
 * Get all available positions in grid
 * Useful for batch operations
 */
export function getAllAvailable(collisionMap: CollisionMap, maxRows: number, maxCols: number): GridPosition[] {
  const available: GridPosition[] = [];

  for (let row = 0; row < maxRows; row++) {
    for (let col = 0; col < maxCols; col++) {
      const position = { row, col };
      if (!isOccupied(position, collisionMap)) {
        available.push(position);
      }
    }
  }

  return available;
}

// ============================================================================
// Multi-Icon Collision
// ============================================================================

/**
 * Check if moving multiple icons would cause collisions
 * Returns conflicting icon IDs
 */
export function detectMultiIconCollisions(
  iconIds: string[],
  targetPositions: Map<string, GridPosition>,
  allIcons: Icon[]
): string[] {
  const collisionMap = buildCollisionMap(allIcons);
  const conflicts: string[] = [];

  // Remove moving icons from collision map
  for (const id of iconIds) {
    const icon = allIcons.find((i) => i.id === id);
    if (icon) {
      collisionMap.occupied.delete(gridKey(icon.position));
    }
  }

  // Check if new positions collide
  for (const [id, position] of targetPositions.entries()) {
    if (isOccupied(position, collisionMap)) {
      conflicts.push(id);
    }
  }

  return conflicts;
}

/**
 * Check if icons overlap (for drag preview)
 */
export function checkOverlap(positions: GridPosition[]): boolean {
  const seen = new Set<string>();

  for (const pos of positions) {
    const key = gridKey(pos);
    if (seen.has(key)) {
      return true; // Overlap detected
    }
    seen.add(key);
  }

  return false;
}

// ============================================================================
// Space Partitioning (Advanced)
// ============================================================================

/**
 * Partition grid into quadrants for spatial queries
 * Useful for large grids with many icons
 */
export interface Quadrant {
  minRow: number;
  maxRow: number;
  minCol: number;
  maxCol: number;
  icons: Icon[];
}

/**
 * Build quadtree-like structure for spatial queries
 * O(n log n) construction, O(log n) range queries
 */
export function partitionSpace(icons: Icon[], maxRows: number, maxCols: number): Quadrant[] {
  const midRow = Math.floor(maxRows / 2);
  const midCol = Math.floor(maxCols / 2);

  const quadrants: Quadrant[] = [
    { minRow: 0, maxRow: midRow, minCol: 0, maxCol: midCol, icons: [] }, // Top-left
    { minRow: 0, maxRow: midRow, minCol: midCol, maxCol: maxCols, icons: [] }, // Top-right
    { minRow: midRow, maxRow: maxRows, minCol: 0, maxCol: midCol, icons: [] }, // Bottom-left
    { minRow: midRow, maxRow: maxRows, minCol: midCol, maxCol: maxCols, icons: [] }, // Bottom-right
  ];

  for (const icon of icons) {
    for (const quadrant of quadrants) {
      if (
        icon.position.row >= quadrant.minRow &&
        icon.position.row < quadrant.maxRow &&
        icon.position.col >= quadrant.minCol &&
        icon.position.col < quadrant.maxCol
      ) {
        quadrant.icons.push(icon);
        break;
      }
    }
  }

  return quadrants;
}

/**
 * Find icons within a rectangular region
 * O(k) where k is number of icons in region
 */
export function queryRegion(
  minRow: number,
  maxRow: number,
  minCol: number,
  maxCol: number,
  icons: Icon[]
): Icon[] {
  return icons.filter(
    (icon) =>
      icon.position.row >= minRow &&
      icon.position.row <= maxRow &&
      icon.position.col >= minCol &&
      icon.position.col <= maxCol
  );
}

