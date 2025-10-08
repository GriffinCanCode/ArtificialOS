/**
 * Icon Grid Component
 * Container for desktop icons with drag-and-drop support
 */

import React, { useCallback, useRef } from "react";
import { Icon } from "./Icon";
import { useIcons } from "../store/store";
import { useGrid } from "../hooks/useGrid";
import { useSelect } from "../hooks/useSelect";
import { useDrag } from "../hooks/useDrag";
import { useDefaults } from "../hooks/useDefaults";
import type { Icon as IconType } from "../core/types";
import "./Grid.css";

// ============================================================================
// Grid Component Props
// ============================================================================

export interface GridProps {
  onIconDoubleClick?: (icon: IconType) => void;
  onContextMenu?: (icon: IconType, event: React.MouseEvent) => void;
  onBackgroundClick?: () => void;
}

// ============================================================================
// Grid Component
// ============================================================================

export const Grid: React.FC<GridProps> = ({ onIconDoubleClick, onContextMenu, onBackgroundClick }) => {
  const icons = useIcons();
  const gridRef = useRef<HTMLDivElement>(null);

  // Hooks
  useGrid(); // Initialize grid (handles viewport updates)
  useDefaults(); // Initialize default icons (Terminal, Files)
  const selection = useSelect();
  const drag = useDrag();

  // Handle icon selection
  const handleIconSelect = useCallback(
    (iconId: string, modifiers: { shift: boolean; ctrl: boolean; meta: boolean }) => {
      selection.select(iconId, modifiers);
    },
    [selection]
  );

  // Handle icon double-click
  const handleIconDoubleClick = useCallback(
    (iconId: string) => {
      const icon = icons.find((i) => i.id === iconId);
      if (icon && onIconDoubleClick) {
        onIconDoubleClick(icon);
      }
    },
    [icons, onIconDoubleClick]
  );

  // Handle drag start
  const handleDragStart = useCallback(
    (iconId: string, position: { x: number; y: number }) => {
      drag.startDrag(iconId, position);
    },
    [drag]
  );

  // Handle context menu
  const handleContextMenu = useCallback(
    (iconId: string, event: React.MouseEvent) => {
      const icon = icons.find((i) => i.id === iconId);
      if (icon && onContextMenu) {
        onContextMenu(icon, event);
      }
    },
    [icons, onContextMenu]
  );

  // Handle background click (deselect all)
  const handleBackgroundClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === gridRef.current) {
        selection.clearSelection();
        if (onBackgroundClick) {
          onBackgroundClick();
        }
      }
    },
    [selection, onBackgroundClick]
  );

  // Handle global mouse move (for dragging)
  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (drag.isDragging) {
        drag.updateDrag({ x: e.clientX, y: e.clientY });
      }
    },
    [drag]
  );

  // Handle global mouse up (end drag)
  const handleMouseUp = useCallback(() => {
    if (drag.isDragging) {
      drag.endDrag();
    }
  }, [drag]);

  // Global mouse listeners for drag
  React.useEffect(() => {
    if (drag.isDragging) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);

      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [drag.isDragging, handleMouseMove, handleMouseUp]);

  // Keyboard shortcuts
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Select all (Cmd/Ctrl+A)
      if ((e.metaKey || e.ctrlKey) && e.key === "a") {
        e.preventDefault();
        selection.selectAll();
      }

      // Deselect all (Escape)
      if (e.key === "Escape") {
        selection.clearSelection();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selection]);

  return (
    <div className="icon-grid" ref={gridRef} onClick={handleBackgroundClick}>
      {/* Render icons */}
      {icons.map((icon) => (
        <Icon
          key={icon.id}
          icon={icon}
          onSelect={handleIconSelect}
          onDoubleClick={handleIconDoubleClick}
          onDragStart={handleDragStart}
          onContextMenu={handleContextMenu}
        />
      ))}

      {/* Drag preview indicator */}
      {drag.isDragging && drag.previewPosition && (
        <div
          className="icon-grid-preview"
          style={{
            position: "absolute",
            left: `${drag.previewPosition.col * 92}px`,
            top: `${drag.previewPosition.row * 102}px`,
            width: "80px",
            height: "90px",
          }}
        />
      )}
    </div>
  );
};

Grid.displayName = "IconGrid";

