/**
 * Drag Hook
 * Window drag state management
 */

import { useState, useCallback } from "react";
import type { Position } from "../core/types";

export interface DragState {
  isDragging: boolean;
  startPosition: Position | null;
  currentPosition: Position | null;
}

export interface DragHandlers {
  onStart: (position: Position) => void;
  onMove: (position: Position) => void;
  onEnd: (position: Position) => void;
}

export function useDrag(handlers: DragHandlers) {
  const [state, setState] = useState<DragState>({
    isDragging: false,
    startPosition: null,
    currentPosition: null,
  });

  const handleDragStart = useCallback(
    (position: Position) => {
      setState({
        isDragging: true,
        startPosition: position,
        currentPosition: position,
      });
      handlers.onStart(position);
    },
    [handlers]
  );

  const handleDragMove = useCallback(
    (position: Position) => {
      if (state.isDragging) {
        setState((prev) => ({
          ...prev,
          currentPosition: position,
        }));
        handlers.onMove(position);
      }
    },
    [state.isDragging, handlers]
  );

  const handleDragEnd = useCallback(
    (position: Position) => {
      setState({
        isDragging: false,
        startPosition: null,
        currentPosition: null,
      });
      handlers.onEnd(position);
    },
    [handlers]
  );

  return {
    state,
    handleDragStart,
    handleDragMove,
    handleDragEnd,
  };
}
