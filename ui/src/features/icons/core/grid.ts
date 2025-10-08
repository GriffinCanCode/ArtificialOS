/**
 * Grid Mathematics
 * Pure functions for grid calculations and transformations
 * Inspired by computational geometry and spatial data structures
 */

import type { GridPosition, PixelPosition, GridConfig, Bounds } from "./types";
import { DEFAULT_GRID_CONFIG } from "./types";

// ============================================================================
// Grid ⟷ Pixel Conversions (Bijective Mapping)
// ============================================================================

/**
 * Convert grid position to pixel position
 * F: ℤ² → ℝ² (Grid space to pixel space)
 */
export function gridToPixel(grid: GridPosition, config: GridConfig = DEFAULT_GRID_CONFIG): PixelPosition {
  return {
    x: config.marginLeft + grid.col * (config.cellWidth + config.padding),
    y: config.marginTop + grid.row * (config.cellHeight + config.padding),
  };
}

/**
 * Convert pixel position to grid position (with snapping)
 * F⁻¹: ℝ² → ℤ² (Pixel space to grid space)
 */
export function pixelToGrid(pixel: PixelPosition, config: GridConfig = DEFAULT_GRID_CONFIG): GridPosition {
  const col = Math.floor((pixel.x - config.marginLeft) / (config.cellWidth + config.padding));
  const row = Math.floor((pixel.y - config.marginTop) / (config.cellHeight + config.padding));

  return {
    row: Math.max(0, row),
    col: Math.max(0, col),
  };
}

/**
 * Snap pixel position to nearest grid cell
 * Implements nearest-neighbor rounding in 2D space
 */
export function snapToGrid(pixel: PixelPosition, config: GridConfig = DEFAULT_GRID_CONFIG): GridPosition {
  const col = Math.round((pixel.x - config.marginLeft) / (config.cellWidth + config.padding));
  const row = Math.round((pixel.y - config.marginTop) / (config.cellHeight + config.padding));

  return {
    row: Math.max(0, row),
    col: Math.max(0, col),
  };
}

// ============================================================================
// Grid Dimensions (Viewport-Aware)
// ============================================================================

/**
 * Calculate maximum grid dimensions for viewport
 * Returns the grid capacity (rows × cols)
 */
export function calculateGridDimensions(
  viewportWidth: number,
  viewportHeight: number,
  config: GridConfig = DEFAULT_GRID_CONFIG
): { rows: number; cols: number; capacity: number } {
  const availableWidth = viewportWidth - config.marginLeft - config.marginRight;
  const availableHeight = viewportHeight - config.marginTop - config.marginBottom;

  const cols = Math.floor(availableWidth / (config.cellWidth + config.padding));
  const rows = Math.floor(availableHeight / (config.cellHeight + config.padding));

  return {
    rows: Math.max(1, rows),
    cols: Math.max(1, cols),
    capacity: Math.max(1, rows * cols),
  };
}

/**
 * Check if grid position is within viewport bounds
 */
export function isInBounds(
  grid: GridPosition,
  viewportWidth: number,
  viewportHeight: number,
  config: GridConfig = DEFAULT_GRID_CONFIG
): boolean {
  const { rows, cols } = calculateGridDimensions(viewportWidth, viewportHeight, config);
  return grid.row >= 0 && grid.row < rows && grid.col >= 0 && grid.col < cols;
}

// ============================================================================
// Grid Key Generation (Spatial Hashing)
// ============================================================================

/**
 * Generate unique string key for grid position
 * Used for O(1) collision detection via hash map
 */
export function gridKey(grid: GridPosition): string {
  return `${grid.row}:${grid.col}`;
}

/**
 * Parse grid key back to position
 */
export function parseGridKey(key: string): GridPosition {
  const [row, col] = key.split(":").map(Number);
  return { row, col };
}

// ============================================================================
// Distance Metrics
// ============================================================================

/**
 * Manhattan distance (L1 norm) between grid positions
 * d(p, q) = |p.row - q.row| + |p.col - q.col|
 */
export function manhattanDistance(a: GridPosition, b: GridPosition): number {
  return Math.abs(a.row - b.row) + Math.abs(a.col - b.col);
}

/**
 * Euclidean distance (L2 norm) between grid positions
 * d(p, q) = √[(p.row - q.row)² + (p.col - q.col)²]
 */
export function euclideanDistance(a: GridPosition, b: GridPosition): number {
  const dr = a.row - b.row;
  const dc = a.col - b.col;
  return Math.sqrt(dr * dr + dc * dc);
}

/**
 * Chebyshev distance (L∞ norm) - maximum coordinate difference
 * Useful for "king's move" distance in grid
 */
export function chebyshevDistance(a: GridPosition, b: GridPosition): number {
  return Math.max(Math.abs(a.row - b.row), Math.abs(a.col - b.col));
}

// ============================================================================
// Bounds Calculations
// ============================================================================

/**
 * Get pixel bounds for a grid cell
 */
export function getCellBounds(grid: GridPosition, config: GridConfig = DEFAULT_GRID_CONFIG): Bounds {
  const pixel = gridToPixel(grid, config);
  return {
    x: pixel.x,
    y: pixel.y,
    width: config.cellWidth,
    height: config.cellHeight,
  };
}

/**
 * Check if bounds intersect (for collision detection)
 */
export function boundsIntersect(a: Bounds, b: Bounds): boolean {
  return !(a.x + a.width < b.x || b.x + b.width < a.x || a.y + a.height < b.y || b.y + b.height < a.y);
}

/**
 * Check if point is inside bounds
 */
export function pointInBounds(point: PixelPosition, bounds: Bounds): boolean {
  return (
    point.x >= bounds.x &&
    point.x <= bounds.x + bounds.width &&
    point.y >= bounds.y &&
    point.y <= bounds.y + bounds.height
  );
}

// ============================================================================
// Grid Traversal Algorithms
// ============================================================================

/**
 * Generate positions in reading order (left-to-right, top-to-bottom)
 * Useful for auto-arrange algorithms
 */
export function* gridTraversal(
  rows: number,
  cols: number,
  startRow: number = 0,
  startCol: number = 0
): Generator<GridPosition> {
  for (let row = startRow; row < rows; row++) {
    for (let col = startCol; col < cols; col++) {
      yield { row, col };
    }
  }
}

/**
 * Generate positions in spiral order (useful for alternative layouts)
 * Starts from center and spirals outward
 */
export function* spiralTraversal(rows: number, cols: number): Generator<GridPosition> {
  const visited = new Set<string>();
  const centerRow = Math.floor(rows / 2);
  const centerCol = Math.floor(cols / 2);

  let row = centerRow;
  let col = centerCol;
  let steps = 1;
  let direction = 0; // 0: right, 1: down, 2: left, 3: up

  yield { row, col };
  visited.add(gridKey({ row, col }));

  while (visited.size < rows * cols) {
    for (let i = 0; i < 2; i++) {
      for (let j = 0; j < steps; j++) {
        switch (direction) {
          case 0:
            col++;
            break;
          case 1:
            row++;
            break;
          case 2:
            col--;
            break;
          case 3:
            row--;
            break;
        }

        if (row >= 0 && row < rows && col >= 0 && col < cols) {
          const key = gridKey({ row, col });
          if (!visited.has(key)) {
            yield { row, col };
            visited.add(key);
          }
        }
      }
      direction = (direction + 1) % 4;
    }
    steps++;
  }
}

/**
 * Get neighboring positions (4-connected grid)
 */
export function getNeighbors(grid: GridPosition, rows: number, cols: number): GridPosition[] {
  const neighbors: GridPosition[] = [];
  const deltas = [
    { row: -1, col: 0 }, // Up
    { row: 1, col: 0 }, // Down
    { row: 0, col: -1 }, // Left
    { row: 0, col: 1 }, // Right
  ];

  for (const delta of deltas) {
    const neighbor = {
      row: grid.row + delta.row,
      col: grid.col + delta.col,
    };

    if (neighbor.row >= 0 && neighbor.row < rows && neighbor.col >= 0 && neighbor.col < cols) {
      neighbors.push(neighbor);
    }
  }

  return neighbors;
}

/**
 * Get all positions in 8-connected neighborhood (Moore neighborhood)
 */
export function getMooreNeighborhood(grid: GridPosition, rows: number, cols: number): GridPosition[] {
  const neighbors: GridPosition[] = [];

  for (let dr = -1; dr <= 1; dr++) {
    for (let dc = -1; dc <= 1; dc++) {
      if (dr === 0 && dc === 0) continue; // Skip center

      const neighbor = {
        row: grid.row + dr,
        col: grid.col + dc,
      };

      if (neighbor.row >= 0 && neighbor.row < rows && neighbor.col >= 0 && neighbor.col < cols) {
        neighbors.push(neighbor);
      }
    }
  }

  return neighbors;
}

