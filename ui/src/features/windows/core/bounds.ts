/**
 * Bounds Core
 * Pure functions for bounds calculations and constraints
 */

import type { Bounds, Position, Size } from "./types";
import { getViewport, getAvailable } from "./viewport";
import { MENUBAR_HEIGHT, TASKBAR_HEIGHT } from "./types";

/**
 * Constrain bounds to viewport
 */
export function constrainBounds(bounds: Bounds): Bounds {
  const viewport = getViewport();

  return {
    position: {
      x: Math.max(0, Math.min(bounds.position.x, viewport.width - bounds.size.width)),
      y: Math.max(
        MENUBAR_HEIGHT,
        Math.min(
          bounds.position.y,
          viewport.height - TASKBAR_HEIGHT - bounds.size.height
        )
      ),
    },
    size: {
      width: Math.min(bounds.size.width, viewport.width),
      height: Math.min(
        bounds.size.height,
        viewport.height - MENUBAR_HEIGHT - TASKBAR_HEIGHT
      ),
    },
  };
}

/**
 * Get maximized bounds
 */
export function getMaximizedBounds(): Bounds {
  return getAvailable();
}

/**
 * Calculate cascade position
 */
export function getCascadePosition(count: number, offset: number = 30): Position {
  const viewport = getViewport();

  const baseX = Math.min(100, viewport.width * 0.1);
  const baseY = Math.min(80, MENUBAR_HEIGHT + viewport.height * 0.1);

  return {
    x: baseX + count * offset,
    y: baseY + count * offset,
  };
}

/**
 * Merge position and size into bounds
 */
export function mergeBounds(position: Position, size: Size): Bounds {
  return { position, size };
}

/**
 * Split bounds into position and size
 */
export function splitBounds(bounds: Bounds): [Position, Size] {
  return [bounds.position, bounds.size];
}
