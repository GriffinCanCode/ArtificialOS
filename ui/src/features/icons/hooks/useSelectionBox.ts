/**
 * useSelectionBox Hook
 * Box selection (drag-to-select) management
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { useActions, useSelectionBox as useSelectionBoxState } from "../store/store";

// ============================================================================
// Selection Box Hook
// ============================================================================

export interface SelectionBoxHook {
  isSelecting: boolean;
  selectionBox: ReturnType<typeof useSelectionBoxState>;
  startSelection: (position: { x: number; y: number }) => void;
  updateSelection: (position: { x: number; y: number }) => void;
  endSelection: () => void;
  cancelSelection: () => void;
}

const SELECTION_THRESHOLD = 5; // Minimum drag distance to start selection

/**
 * Box selection hook
 * Handles drag-to-select multiple icons
 */
export function useSelectionBox(): SelectionBoxHook {
  const selectionBox = useSelectionBoxState();
  const { startSelectionBox, updateSelectionBox, endSelectionBox, cancelSelectionBox } = useActions();

  const [isSelecting, setIsSelecting] = useState(false);
  const startPos = useRef<{ x: number; y: number } | null>(null);
  const thresholdMet = useRef(false);

  const startSelection = useCallback(
    (position: { x: number; y: number }) => {
      startPos.current = position;
      thresholdMet.current = false;
      setIsSelecting(false); // Don't activate until threshold is met
    },
    []
  );

  const updateSelection = useCallback(
    (position: { x: number; y: number }) => {
      if (!startPos.current) return;

      // Check if drag threshold is met
      if (!thresholdMet.current) {
        const dx = position.x - startPos.current.x;
        const dy = position.y - startPos.current.y;
        const distance = Math.sqrt(dx * dx + dy * dy);

        if (distance < SELECTION_THRESHOLD) {
          return; // Not selecting yet
        }

        // Threshold met - start selection
        thresholdMet.current = true;
        setIsSelecting(true);
        startSelectionBox(startPos.current);
      }

      // Update selection box
      if (thresholdMet.current) {
        updateSelectionBox(position);
      }
    },
    [startSelectionBox, updateSelectionBox]
  );

  const endSelection = useCallback(() => {
    if (thresholdMet.current) {
      endSelectionBox();
    }
    setIsSelecting(false);
    startPos.current = null;
    thresholdMet.current = false;
  }, [endSelectionBox]);

  const cancelSelection = useCallback(() => {
    if (thresholdMet.current) {
      cancelSelectionBox();
    }
    setIsSelecting(false);
    startPos.current = null;
    thresholdMet.current = false;
  }, [cancelSelectionBox]);

  // Cancel on Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isSelecting) {
        cancelSelection();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isSelecting, cancelSelection]);

  return {
    isSelecting,
    selectionBox,
    startSelection,
    updateSelection,
    endSelection,
    cancelSelection,
  };
}

