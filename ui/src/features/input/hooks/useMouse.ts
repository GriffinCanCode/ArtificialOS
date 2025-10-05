/**
 * Mouse Input Hook
 * React hook for mouse event handling
 */

import { useState, useCallback, useRef, useEffect } from "react";
import type { DragState, MouseOptions } from "../core/types";
import {
  extractMouseData,
  createDragState,
  updateDragState,
  exceedsDragThreshold,
  isDoubleClick,
} from "../core/mouse";

/**
 * Hook for drag handling
 */
export function useDrag(
  onDragStart?: (state: DragState) => void,
  onDrag?: (state: DragState) => void,
  onDragEnd?: (state: DragState) => void,
  options?: MouseOptions
) {
  const [dragState, setDragState] = useState<DragState | null>(null);

  const handleMouseDown = useCallback(
    (event: MouseEvent) => {
      if (options?.button && extractMouseData(event).button !== options.button) {
        return;
      }

      if (options?.preventDefault) {
        event.preventDefault();
      }

      const state = createDragState(event.clientX, event.clientY);
      setDragState(state);
      onDragStart?.(state);
    },
    [onDragStart, options]
  );

  const handleMouseMove = useCallback(
    (event: MouseEvent) => {
      if (!dragState) return;

      const newState = updateDragState(dragState, event.clientX, event.clientY);

      if (options?.threshold && !exceedsDragThreshold(newState, options.threshold)) {
        return;
      }

      setDragState(newState);
      onDrag?.(newState);
    },
    [dragState, onDrag, options]
  );

  const handleMouseUp = useCallback(
    (event: MouseEvent) => {
      if (!dragState) return;

      const finalState = updateDragState(dragState, event.clientX, event.clientY);
      onDragEnd?.(finalState);
      setDragState(null);
    },
    [dragState, onDragEnd]
  );

  useEffect(() => {
    if (dragState) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);

      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [dragState, handleMouseMove, handleMouseUp]);

  return {
    dragState,
    isDragging: dragState !== null,
    onMouseDown: handleMouseDown,
  };
}

/**
 * Hook for click handling with double-click detection
 */
export function useClick(
  onClick?: (event: MouseEvent) => void,
  onDoubleClick?: (event: MouseEvent) => void,
  threshold: number = 300
) {
  const lastClickTime = useRef(0);
  const clickTimer = useRef<NodeJS.Timeout | null>(null);

  const handleClick = useCallback(
    (event: MouseEvent) => {
      const currentTime = Date.now();

      if (onDoubleClick && isDoubleClick(lastClickTime.current, currentTime, threshold)) {
        if (clickTimer.current) {
          clearTimeout(clickTimer.current);
          clickTimer.current = null;
        }
        onDoubleClick(event);
        lastClickTime.current = 0;
      } else {
        if (onClick) {
          clickTimer.current = setTimeout(() => {
            onClick(event);
            clickTimer.current = null;
          }, threshold);
        }
        lastClickTime.current = currentTime;
      }
    },
    [onClick, onDoubleClick, threshold]
  );

  useEffect(() => {
    return () => {
      if (clickTimer.current) {
        clearTimeout(clickTimer.current);
      }
    };
  }, []);

  return handleClick;
}

/**
 * Hook for hover handling
 */
export function useHover<T extends HTMLElement>() {
  const [isHovered, setIsHovered] = useState(false);
  const ref = useRef<T>(null);

  const handleMouseEnter = useCallback(() => setIsHovered(true), []);
  const handleMouseLeave = useCallback(() => setIsHovered(false), []);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    element.addEventListener("mouseenter", handleMouseEnter);
    element.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      element.removeEventListener("mouseenter", handleMouseEnter);
      element.removeEventListener("mouseleave", handleMouseLeave);
    };
  }, [handleMouseEnter, handleMouseLeave]);

  return { ref, isHovered };
}

/**
 * Hook for mouse position tracking
 */
export function useMousePosition() {
  const [position, setPosition] = useState({ x: 0, y: 0 });

  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      setPosition({ x: event.clientX, y: event.clientY });
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, []);

  return position;
}
