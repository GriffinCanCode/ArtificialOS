/**
 * Gesture Input Core
 * Pure functions for gesture detection and handling
 */

import type { SwipeDirection } from "./types";

/**
 * Detect swipe direction from velocity
 */
export function detectSwipeDirection(
  velocityX: number,
  velocityY: number,
  threshold: number = 0.5
): SwipeDirection {
  return {
    x: Math.abs(velocityX) > threshold
      ? velocityX > 0
        ? "right"
        : "left"
      : "none",
    y: Math.abs(velocityY) > threshold
      ? velocityY > 0
        ? "down"
        : "up"
      : "none",
  };
}

/**
 * Calculate gesture velocity
 */
export function calculateVelocity(
  distance: number,
  time: number
): number {
  return time > 0 ? distance / time : 0;
}

/**
 * Normalize direction vector
 */
export function normalizeDirection(
  x: number,
  y: number
): [number, number] {
  const length = Math.sqrt(x * x + y * y);
  return length > 0 ? [x / length, y / length] : [0, 0];
}

/**
 * Check if gesture is swipe
 */
export function isSwipeGesture(
  velocityX: number,
  velocityY: number,
  threshold: number = 0.5
): boolean {
  return Math.abs(velocityX) > threshold || Math.abs(velocityY) > threshold;
}

/**
 * Check if gesture is tap
 */
export function isTapGesture(
  distance: number,
  duration: number,
  distanceThreshold: number = 10,
  durationThreshold: number = 200
): boolean {
  return distance < distanceThreshold && duration < durationThreshold;
}

/**
 * Check if gesture is long press
 */
export function isLongPress(
  duration: number,
  threshold: number = 500
): boolean {
  return duration >= threshold;
}

/**
 * Calculate pinch scale
 */
export function calculatePinchScale(
  initialDistance: number,
  currentDistance: number
): number {
  return initialDistance > 0 ? currentDistance / initialDistance : 1;
}

/**
 * Calculate rotation angle from two touch points
 */
export function calculateRotation(
  x1: number,
  y1: number,
  x2: number,
  y2: number
): number {
  return Math.atan2(y2 - y1, x2 - x1) * (180 / Math.PI);
}

/**
 * Check if gesture is pinch
 */
export function isPinchGesture(touchCount: number): boolean {
  return touchCount === 2;
}

/**
 * Clamp gesture value with rubber band effect
 */
export function rubberBand(
  value: number,
  min: number,
  max: number,
  constant: number = 0.15
): number {
  if (value < min) {
    return min - Math.pow(min - value, constant);
  }
  if (value > max) {
    return max + Math.pow(value - max, constant);
  }
  return value;
}

/**
 * Apply momentum decay
 */
export function applyMomentum(
  velocity: number,
  decay: number = 0.95
): number {
  return velocity * decay;
}

/**
 * Check if momentum is negligible
 */
export function isNegligibleMomentum(
  velocity: number,
  threshold: number = 0.01
): boolean {
  return Math.abs(velocity) < threshold;
}
