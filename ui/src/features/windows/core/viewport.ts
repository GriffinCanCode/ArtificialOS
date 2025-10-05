/**
 * Viewport Core
 * Pure functions for viewport calculations
 */

import type { Size, Position, Bounds } from "./types";
import { MENUBAR_HEIGHT, TASKBAR_HEIGHT } from "./types";

/**
 * Get current viewport dimensions
 */
export function getViewport(): Size {
  return {
    width: window.innerWidth,
    height: window.innerHeight,
  };
}

/**
 * Get available space (excluding menubar and taskbar)
 */
export function getAvailable(): Bounds {
  const viewport = getViewport();

  return {
    position: { x: 0, y: MENUBAR_HEIGHT },
    size: {
      width: viewport.width,
      height: viewport.height - MENUBAR_HEIGHT - TASKBAR_HEIGHT,
    },
  };
}

/**
 * Check if position is within viewport
 */
export function isInViewport(position: Position, size: Size): boolean {
  const viewport = getViewport();

  return (
    position.x >= 0 &&
    position.y >= MENUBAR_HEIGHT &&
    position.x + size.width <= viewport.width &&
    position.y + size.height <= viewport.height - TASKBAR_HEIGHT
  );
}

/**
 * Get center position for given size
 */
export function getCenterPosition(size: Size): Position {
  const viewport = getViewport();

  return {
    x: Math.max(0, (viewport.width - size.width) / 2),
    y: Math.max(MENUBAR_HEIGHT, (viewport.height - size.height) / 2),
  };
}
