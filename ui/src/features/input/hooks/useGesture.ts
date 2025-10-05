/**
 * Gesture Input Hook
 * React hook wrapping @use-gesture/react with typed utilities
 */

import { useDrag, useGesture as useGestureLib, usePinch } from "@use-gesture/react";
import type { SwipeDirection } from "../core/types";
import {
  detectSwipeDirection,
  isSwipeGesture,
  isTapGesture,
  isLongPress,
  rubberBand,
} from "../core/gesture";

/**
 * Hook for swipe gestures
 */
export function useSwipe(
  onSwipe: (direction: SwipeDirection) => void,
  threshold: number = 0.5
) {
  return useDrag(
    ({ velocity: [vx, vy], last }) => {
      if (last && isSwipeGesture(vx, vy, threshold)) {
        const direction = detectSwipeDirection(vx, vy, threshold);
        onSwipe(direction);
      }
    },
    {
      filterTaps: true,
      from: () => [0, 0],
    }
  );
}

/**
 * Hook for tap gestures
 */
export function useTap(onTap: () => void, threshold: { distance?: number; duration?: number } = {}) {
  const startTime = { current: 0 };

  return useDrag(
    ({ down, distance, elapsedTime }) => {
      if (down) {
        startTime.current = Date.now();
      } else {
        // Convert Vector2 distance to number (always an array in @use-gesture)
        const [dx, dy] = distance;
        const distanceValue = Math.sqrt(dx * dx + dy * dy);

        if (
          isTapGesture(
            distanceValue,
            elapsedTime,
            threshold.distance,
            threshold.duration
          )
        ) {
          onTap();
        }
      }
    },
    {
      filterTaps: true,
    }
  );
}

/**
 * Hook for long press gestures
 */
export function useLongPress(onLongPress: () => void, threshold: number = 500) {
  return useDrag(
    ({ down, elapsedTime }) => {
      if (down && isLongPress(elapsedTime, threshold)) {
        onLongPress();
      }
    },
    {
      filterTaps: true,
    }
  );
}

/**
 * Hook for pinch gestures
 */
export function usePinchGesture(
  onPinch: (scale: number, origin: [number, number]) => void
) {
  return usePinch(({ offset, origin }) => {
    const scale = Array.isArray(offset) ? offset[0] : offset;
    onPinch(scale, origin);
  });
}

/**
 * Hook for drag with rubber band effect
 */
export function useDragWithRubberBand(
  bounds: { left: number; top: number; right: number; bottom: number },
  onDrag?: (x: number, y: number) => void
) {
  return useDrag(
    ({ offset: [x, y] }) => {
      const boundedX = rubberBand(x, bounds.left, bounds.right);
      const boundedY = rubberBand(y, bounds.top, bounds.bottom);
      onDrag?.(boundedX, boundedY);
    },
    {
      from: () => [0, 0],
    }
  );
}

/**
 * Hook for unified gesture handling
 */
export function useGesture(handlers: {
  onDrag?: (state: any) => void;
  onPinch?: (state: any) => void;
  onWheel?: (state: any) => void;
  onScroll?: (state: any) => void;
  onMove?: (state: any) => void;
  onHover?: (state: any) => void;
}) {
  return useGestureLib(handlers, {
    drag: { filterTaps: true },
  });
}
