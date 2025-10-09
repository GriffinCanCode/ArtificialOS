/**
 * useDrag Hook
 * Icon drag-and-drop management
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { useActions, useDraggedIds, useIcons } from "../store/store";
import { snapToGrid } from "../core/grid";
import type { PixelPosition, GridPosition, GridConfig } from "../core/types";
import { DRAG_THRESHOLD, DEFAULT_GRID_CONFIG } from "../core/types";

// ============================================================================
// Drag Hook
// ============================================================================

export interface DragHook {
  isDragging: boolean;
  draggedIds: Set<string>;
  dragOffset: PixelPosition | null;
  previewPosition: GridPosition | null;
  startDrag: (iconId: string, startPosition: PixelPosition) => void;
  updateDrag: (currentPosition: PixelPosition) => void;
  endDrag: () => void;
  cancelDrag: () => void;
}

/**
 * Drag-and-drop management hook
 * Handles drag initiation, preview, and drop
 */
export function useDrag(config: GridConfig = DEFAULT_GRID_CONFIG): DragHook {
  const icons = useIcons();
  const draggedIds = useDraggedIds();
  const { startDrag: startDragStore, endDrag: endDragStore, updatePositions } = useActions();

  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState<PixelPosition | null>(null);
  const [previewPosition, setPreviewPosition] = useState<GridPosition | null>(null);
  const [dragStartPos, setDragStartPos] = useState<PixelPosition | null>(null);

  const dragThresholdMet = useRef(false);

  const startDrag = useCallback(
    (iconId: string, startPosition: PixelPosition) => {
      // Find icon and selected icons
      const icon = icons.find((i) => i.id === iconId);
      if (!icon) return;

      const selectedIcons = icons.filter((i) => i.isSelected);
      const draggedIconIds = selectedIcons.length > 0 && selectedIcons.some((i) => i.id === iconId)
        ? selectedIcons.map((i) => i.id)
        : [iconId];

      // Store drag start position
      setDragStartPos(startPosition);
      setDragOffset(startPosition);
      dragThresholdMet.current = false;

      // Mark icons as dragging
      startDragStore(draggedIconIds);
    },
    [icons, startDragStore]
  );

  const updateDrag = useCallback(
    (currentPosition: PixelPosition) => {
      if (!dragStartPos || !dragOffset) return;

      // Check if drag threshold is met
      const dx = currentPosition.x - dragStartPos.x;
      const dy = currentPosition.y - dragStartPos.y;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (!dragThresholdMet.current && distance < DRAG_THRESHOLD) {
        return; // Not dragging yet
      }

      if (!dragThresholdMet.current) {
        dragThresholdMet.current = true;
        setIsDragging(true);
      }

      // Update drag offset
      setDragOffset(currentPosition);

      // Calculate preview position (snap to grid)
      const gridPos = snapToGrid(currentPosition, config);
      setPreviewPosition(gridPos);
    },
    [dragStartPos, dragOffset, config]
  );

  const endDrag = useCallback(() => {
    if (!isDragging || !previewPosition) {
      // No actual drag occurred
      endDragStore();
      setIsDragging(false);
      setDragOffset(null);
      setPreviewPosition(null);
      setDragStartPos(null);
      dragThresholdMet.current = false;
      return;
    }

    // Update positions for dragged icons
    const newPositions = new Map<string, GridPosition>();

    // Get dragged icons and calculate relative positions
    const draggedIcons = icons.filter((i) => draggedIds.has(i.id));

    if (draggedIcons.length === 1) {
      // Single icon: simply move to preview position
      newPositions.set(draggedIcons[0].id, previewPosition);
    } else {
      // Multiple icons: maintain relative positions
      // Find the "anchor" icon (first selected or closest to drag start)
      const anchorIcon = draggedIcons[0];
      const deltaRow = previewPosition.row - anchorIcon.position.row;
      const deltaCol = previewPosition.col - anchorIcon.position.col;

      // Apply delta to all dragged icons
      draggedIcons.forEach((icon) => {
        const newPos = {
          row: Math.max(0, icon.position.row + deltaRow),
          col: Math.max(0, icon.position.col + deltaCol),
        };
        newPositions.set(icon.id, newPos);
      });
    }

    updatePositions(newPositions);

    // Clean up
    endDragStore();
    setIsDragging(false);
    setDragOffset(null);
    setPreviewPosition(null);
    setDragStartPos(null);
    dragThresholdMet.current = false;
  }, [isDragging, previewPosition, draggedIds, icons, updatePositions, endDragStore]);

  const cancelDrag = useCallback(() => {
    endDragStore();
    setIsDragging(false);
    setDragOffset(null);
    setPreviewPosition(null);
    setDragStartPos(null);
    dragThresholdMet.current = false;
  }, [endDragStore]);

  // Cancel drag on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isDragging) {
        cancelDrag();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isDragging, cancelDrag]);

  return {
    isDragging,
    draggedIds,
    dragOffset,
    previewPosition,
    startDrag,
    updateDrag,
    endDrag,
    cancelDrag,
  };
}

