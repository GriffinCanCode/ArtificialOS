/**
 * Mouse Input Core
 * Pure functions for mouse event handling and detection
 */

import type { MouseButton, MouseEventData, DragState } from "./types";

/**
 * Extract structured data from mouse event
 */
export function extractMouseData(event: MouseEvent): MouseEventData {
  return {
    x: event.pageX,
    y: event.pageY,
    clientX: event.clientX,
    clientY: event.clientY,
    button: getMouseButton(event.button),
    target: event.target,
  };
}

/**
 * Convert mouse button number to semantic name
 */
export function getMouseButton(button: number): MouseButton {
  switch (button) {
    case 0:
      return "left";
    case 1:
      return "middle";
    case 2:
      return "right";
    default:
      return "left";
  }
}

/**
 * Calculate distance between two points
 */
export function calculateDistance(
  x1: number,
  y1: number,
  x2: number,
  y2: number
): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  return Math.sqrt(dx * dx + dy * dy);
}

/**
 * Calculate angle between two points in degrees
 */
export function calculateAngle(
  x1: number,
  y1: number,
  x2: number,
  y2: number
): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  return (Math.atan2(dy, dx) * 180) / Math.PI;
}

/**
 * Create initial drag state
 */
export function createDragState(x: number, y: number): DragState {
  return {
    isDragging: true,
    startX: x,
    startY: y,
    currentX: x,
    currentY: y,
    deltaX: 0,
    deltaY: 0,
  };
}

/**
 * Update drag state with new position
 */
export function updateDragState(state: DragState, x: number, y: number): DragState {
  return {
    ...state,
    currentX: x,
    currentY: y,
    deltaX: x - state.startX,
    deltaY: y - state.startY,
  };
}

/**
 * Check if drag exceeds threshold
 */
export function exceedsDragThreshold(
  state: DragState,
  threshold: number = 5
): boolean {
  const distance = calculateDistance(
    state.startX,
    state.startY,
    state.currentX,
    state.currentY
  );
  return distance > threshold;
}

/**
 * Check if element is under cursor
 */
export function isElementUnderCursor(
  element: HTMLElement,
  x: number,
  y: number
): boolean {
  const rect = element.getBoundingClientRect();
  return (
    x >= rect.left &&
    x <= rect.right &&
    y >= rect.top &&
    y <= rect.bottom
  );
}

/**
 * Get element at position
 */
export function getElementAtPosition(
  x: number,
  y: number
): HTMLElement | null {
  return document.elementFromPoint(x, y) as HTMLElement | null;
}

/**
 * Check if click is double click
 */
export function isDoubleClick(
  lastClickTime: number,
  currentClickTime: number,
  threshold: number = 300
): boolean {
  return currentClickTime - lastClickTime < threshold;
}

/**
 * Constrain value to bounds
 */
export function constrain(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Constrain point to bounds
 */
export function constrainToBounds(
  x: number,
  y: number,
  bounds: { left: number; top: number; right: number; bottom: number }
): { x: number; y: number } {
  return {
    x: constrain(x, bounds.left, bounds.right),
    y: constrain(y, bounds.top, bounds.bottom),
  };
}
