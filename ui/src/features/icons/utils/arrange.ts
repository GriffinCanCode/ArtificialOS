/**
 * Icon Arrangement Algorithms
 * Auto-arrange, alignment, and layout optimization
 */

import type { Icon, GridPosition, ArrangeStrategy } from "../core/types";
import { buildCollisionMap, findFirstAvailable } from "../core/collision";
import { gridTraversal, euclideanDistance } from "../core/grid";

/**
 * Auto-arrange icons in reading order (left-to-right, top-to-bottom)
 * O(n log n) due to sorting
 */
export function arrangeInGrid(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();
  let index = 0;

  // Sort by current position (maintain relative order)
  const sorted = [...icons].sort((a, b) => {
    if (a.position.row !== b.position.row) {
      return a.position.row - b.position.row;
    }
    return a.position.col - b.position.col;
  });

  // Assign new positions in reading order
  for (const pos of gridTraversal(maxRows, maxCols)) {
    if (index >= sorted.length) break;
    newPositions.set(sorted[index].id, pos);
    index++;
  }

  return newPositions;
}

/**
 * Arrange icons alphabetically by name
 */
export function arrangeByName(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  // Sort alphabetically
  const sorted = [...icons].sort((a, b) => a.label.localeCompare(b.label));

  let index = 0;
  for (const pos of gridTraversal(maxRows, maxCols)) {
    if (index >= sorted.length) break;
    newPositions.set(sorted[index].id, pos);
    index++;
  }

  return newPositions;
}

/**
 * Arrange icons by type (apps, folders, files, shortcuts)
 */
export function arrangeByType(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  // Sort by type priority
  const typePriority: Record<string, number> = {
    app: 0,
    native: 1,
    folder: 2,
    file: 3,
    shortcut: 4,
  };

  const sorted = [...icons].sort((a, b) => {
    const aPriority = typePriority[a.type] ?? 99;
    const bPriority = typePriority[b.type] ?? 99;

    if (aPriority !== bPriority) {
      return aPriority - bPriority;
    }

    // Same type: sort by name
    return a.label.localeCompare(b.label);
  });

  let index = 0;
  for (const pos of gridTraversal(maxRows, maxCols)) {
    if (index >= sorted.length) break;
    newPositions.set(sorted[index].id, pos);
    index++;
  }

  return newPositions;
}

/**
 * Arrange icons by creation date (newest first)
 */
export function arrangeByDate(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  // Sort by creation date (descending)
  const sorted = [...icons].sort((a, b) => b.createdAt - a.createdAt);

  let index = 0;
  for (const pos of gridTraversal(maxRows, maxCols)) {
    if (index >= sorted.length) break;
    newPositions.set(sorted[index].id, pos);
    index++;
  }

  return newPositions;
}

/**
 * Arrange icons by file size (for file icons)
 */
export function arrangeBySize(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  // Sort by size (descending) - only for files
  const sorted = [...icons].sort((a, b) => {
    const aSize = a.metadata.type === "file" ? a.metadata.size : 0;
    const bSize = b.metadata.type === "file" ? b.metadata.size : 0;
    return bSize - aSize;
  });

  let index = 0;
  for (const pos of gridTraversal(maxRows, maxCols)) {
    if (index >= sorted.length) break;
    newPositions.set(sorted[index].id, pos);
    index++;
  }

  return newPositions;
}

/**
 * Unified arrange function with strategy selection
 */
export function arrange(
  icons: Icon[],
  strategy: ArrangeStrategy,
  maxRows: number,
  maxCols: number
): Map<string, GridPosition> {
  switch (strategy) {
    case "grid":
      return arrangeInGrid(icons, maxRows, maxCols);
    case "name":
      return arrangeByName(icons, maxRows, maxCols);
    case "type":
      return arrangeByType(icons, maxRows, maxCols);
    case "date":
      return arrangeByDate(icons, maxRows, maxCols);
    case "size":
      return arrangeBySize(icons, maxRows, maxCols);
    default:
      return arrangeInGrid(icons, maxRows, maxCols);
  }
}

// ============================================================================
// Compact Layout (Fill Gaps)
// ============================================================================

/**
 * Compact icons by filling gaps while preserving relative order
 * Uses greedy algorithm to minimize empty spaces
 */
export function compactLayout(icons: Icon[], maxRows: number, maxCols: number): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();
  const collisionMap = buildCollisionMap([]);

  // Sort by current position (reading order)
  const sorted = [...icons].sort((a, b) => {
    if (a.position.row !== b.position.row) {
      return a.position.row - b.position.row;
    }
    return a.position.col - b.position.col;
  });

  // Place each icon in first available position
  for (const icon of sorted) {
    const pos = findFirstAvailable(collisionMap, maxRows, maxCols);
    if (pos) {
      newPositions.set(icon.id, pos);
      collisionMap.occupied.set(`${pos.row}:${pos.col}`, icon.id);
    }
  }

  return newPositions;
}

// ============================================================================
// Alignment Operations
// ============================================================================

/**
 * Align selected icons to left edge
 */
export function alignLeft(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length === 0) return newPositions;

  // Find leftmost column
  const minCol = Math.min(...icons.map((i) => i.position.col));

  // Align all to that column
  for (const icon of icons) {
    newPositions.set(icon.id, { row: icon.position.row, col: minCol });
  }

  return newPositions;
}

/**
 * Align selected icons to right edge
 */
export function alignRight(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length === 0) return newPositions;

  // Find rightmost column
  const maxCol = Math.max(...icons.map((i) => i.position.col));

  // Align all to that column
  for (const icon of icons) {
    newPositions.set(icon.id, { row: icon.position.row, col: maxCol });
  }

  return newPositions;
}

/**
 * Align selected icons to top edge
 */
export function alignTop(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length === 0) return newPositions;

  // Find topmost row
  const minRow = Math.min(...icons.map((i) => i.position.row));

  // Align all to that row
  for (const icon of icons) {
    newPositions.set(icon.id, { row: minRow, col: icon.position.col });
  }

  return newPositions;
}

/**
 * Align selected icons to bottom edge
 */
export function alignBottom(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length === 0) return newPositions;

  // Find bottommost row
  const maxRow = Math.max(...icons.map((i) => i.position.row));

  // Align all to that row
  for (const icon of icons) {
    newPositions.set(icon.id, { row: maxRow, col: icon.position.col });
  }

  return newPositions;
}

// ============================================================================
// Distribution
// ============================================================================

/**
 * Distribute icons evenly horizontally
 */
export function distributeHorizontally(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length < 3) return newPositions; // Need at least 3 icons

  // Sort by column
  const sorted = [...icons].sort((a, b) => a.position.col - b.position.col);

  const minCol = sorted[0].position.col;
  const maxCol = sorted[sorted.length - 1].position.col;
  const spacing = (maxCol - minCol) / (sorted.length - 1);

  // Distribute evenly
  for (let i = 0; i < sorted.length; i++) {
    const icon = sorted[i];
    const newCol = Math.round(minCol + i * spacing);
    newPositions.set(icon.id, { row: icon.position.row, col: newCol });
  }

  return newPositions;
}

/**
 * Distribute icons evenly vertically
 */
export function distributeVertically(icons: Icon[]): Map<string, GridPosition> {
  const newPositions = new Map<string, GridPosition>();

  if (icons.length < 3) return newPositions; // Need at least 3 icons

  // Sort by row
  const sorted = [...icons].sort((a, b) => a.position.row - b.position.row);

  const minRow = sorted[0].position.row;
  const maxRow = sorted[sorted.length - 1].position.row;
  const spacing = (maxRow - minRow) / (sorted.length - 1);

  // Distribute evenly
  for (let i = 0; i < sorted.length; i++) {
    const icon = sorted[i];
    const newRow = Math.round(minRow + i * spacing);
    newPositions.set(icon.id, { row: newRow, col: icon.position.col });
  }

  return newPositions;
}

// ============================================================================
// Clustering (Advanced)
// ============================================================================

/**
 * Group icons by proximity using k-means clustering
 * Useful for "smart arrange" that respects manual groupings
 */
export function clusterByProximity(
  icons: Icon[],
  k: number = 3
): Map<string, number> {
  const clusters = new Map<string, number>();

  if (icons.length === 0) return clusters;

  // Initialize centroids randomly
  const centroids: GridPosition[] = [];
  const shuffled = [...icons].sort(() => Math.random() - 0.5);

  for (let i = 0; i < Math.min(k, icons.length); i++) {
    centroids.push({ ...shuffled[i].position });
  }

  // K-means iterations (max 10)
  for (let iter = 0; iter < 10; iter++) {
    // Assign icons to nearest centroid
    const assignments = new Map<string, number>();

    for (const icon of icons) {
      let minDist = Infinity;
      let closestCluster = 0;

      for (let c = 0; c < centroids.length; c++) {
        const dist = euclideanDistance(icon.position, centroids[c]);
        if (dist < minDist) {
          minDist = dist;
          closestCluster = c;
        }
      }

      assignments.set(icon.id, closestCluster);
    }

    // Update centroids
    const clusterIcons: Icon[][] = Array.from({ length: k }, () => []);
    for (const icon of icons) {
      const cluster = assignments.get(icon.id)!;
      clusterIcons[cluster].push(icon);
    }

    let changed = false;
    for (let c = 0; c < k; c++) {
      if (clusterIcons[c].length === 0) continue;

      const avgRow =
        clusterIcons[c].reduce((sum, icon) => sum + icon.position.row, 0) / clusterIcons[c].length;
      const avgCol =
        clusterIcons[c].reduce((sum, icon) => sum + icon.position.col, 0) / clusterIcons[c].length;

      const newCentroid = { row: Math.round(avgRow), col: Math.round(avgCol) };

      if (centroids[c].row !== newCentroid.row || centroids[c].col !== newCentroid.col) {
        centroids[c] = newCentroid;
        changed = true;
      }
    }

    if (!changed) {
      break; // Converged
    }
  }

  // Final assignments
  for (const icon of icons) {
    let minDist = Infinity;
    let closestCluster = 0;

    for (let c = 0; c < centroids.length; c++) {
      const dist = euclideanDistance(icon.position, centroids[c]);
      if (dist < minDist) {
        minDist = dist;
        closestCluster = c;
      }
    }

    clusters.set(icon.id, closestCluster);
  }

  return clusters;
}

