/**
 * Snap Core
 * Pure functions for snap-to-edge detection and positioning
 */

import type { Bounds, Position } from "./types";
import { SNAP_THRESHOLD, Zone } from "./types";
import { getViewport, getAvailable } from "./viewport";

/**
 * Calculate snap zone based on position
 */
export function detectZone(position: Position): Zone {
  const viewport = getViewport();
  const { width, height } = viewport;

  const nearLeft = position.x < SNAP_THRESHOLD;
  const nearRight = position.x > width - SNAP_THRESHOLD;
  const nearTop = position.y < SNAP_THRESHOLD;
  const nearBottom = position.y > height - SNAP_THRESHOLD;

  // Corners take priority
  if (nearTop && nearLeft) return Zone.TOP_LEFT;
  if (nearTop && nearRight) return Zone.TOP_RIGHT;
  if (nearBottom && nearLeft) return Zone.BOTTOM_LEFT;
  if (nearBottom && nearRight) return Zone.BOTTOM_RIGHT;

  // Edges
  if (nearTop) return Zone.TOP;
  if (nearLeft) return Zone.LEFT;
  if (nearRight) return Zone.RIGHT;
  if (nearBottom) return Zone.BOTTOM;

  return Zone.NONE;
}

/**
 * Get bounds for snap zone
 */
export function getZoneBounds(zone: Zone): Bounds {
  const available = getAvailable();
  const { position, size } = available;

  const halfWidth = size.width / 2;
  const halfHeight = size.height / 2;

  switch (zone) {
    case Zone.LEFT:
      return {
        position,
        size: { width: halfWidth, height: size.height },
      };

    case Zone.RIGHT:
      return {
        position: { x: position.x + halfWidth, y: position.y },
        size: { width: halfWidth, height: size.height },
      };

    case Zone.TOP:
      return available;

    case Zone.TOP_LEFT:
      return {
        position,
        size: { width: halfWidth, height: halfHeight },
      };

    case Zone.TOP_RIGHT:
      return {
        position: { x: position.x + halfWidth, y: position.y },
        size: { width: halfWidth, height: halfHeight },
      };

    case Zone.BOTTOM_LEFT:
      return {
        position: { x: position.x, y: position.y + halfHeight },
        size: { width: halfWidth, height: halfHeight },
      };

    case Zone.BOTTOM_RIGHT:
      return {
        position: { x: position.x + halfWidth, y: position.y + halfHeight },
        size: { width: halfWidth, height: halfHeight },
      };

    default:
      return available;
  }
}

/**
 * Check if zone is valid
 */
export function isValidZone(zone: Zone): boolean {
  return zone !== Zone.NONE;
}
