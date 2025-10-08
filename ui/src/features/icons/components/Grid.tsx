/**
 * Icon Grid Component
 * Container for desktop icons with drag-and-drop support
 */

import React, { useCallback, useRef, useState } from "react";
import { Icon } from "./Icon";
import { Search } from "./Search";
import { useIcons, useSearchState } from "../store/store";
import { useGrid } from "../hooks/useGrid";
import { useSelect } from "../hooks/useSelect";
import { useDrag } from "../hooks/useDrag";
import { useDefaults } from "../hooks/useDefaults";
import { useSelectionBox } from "../hooks/useSelectionBox";
import { useKeyboard } from "../hooks/useKeyboard";
import { getSelectionBounds } from "../utils/selection";
import type { Icon as IconType } from "../core/types";
import "./Grid.css";

// ============================================================================
// Grid Component Props
// ============================================================================

export interface GridProps {
  onIconDoubleClick?: (icon: IconType) => void;
  onContextMenu?: (icon: IconType, event: React.MouseEvent) => void;
  onBackgroundClick?: () => void;
  enableSearch?: boolean;
}

// ============================================================================
// Grid Component
// ============================================================================

export const Grid: React.FC<GridProps> = ({
  onIconDoubleClick,
  onContextMenu,
  onBackgroundClick,
  enableSearch = true,
}) => {
  const icons = useIcons();
  const searchState = useSearchState();
  const gridRef = useRef<HTMLDivElement>(null);
  const [showSearch, setShowSearch] = useState(false);

  // Hooks
  useGrid(); // Initialize grid (handles viewport updates)
  useDefaults(); // Initialize default icons (Terminal, Files)
  const selection = useSelect();
  const drag = useDrag();
  const selectionBox = useSelectionBox();

  // Keyboard navigation
  useKeyboard({
    onSearch: enableSearch ? () => setShowSearch(true) : undefined,
    onEscape: () => setShowSearch(false),
    disabled: drag.isDragging || selectionBox.isSelecting,
  });

  // Filter icons by search
  const visibleIcons = searchState.isActive
    ? icons.filter((icon) => searchState.results.includes(icon.id))
    : icons;

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

  // Handle background mouse down (start selection box)
  const handleBackgroundMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === gridRef.current && e.button === 0) {
        // Left click only
        const rect = gridRef.current.getBoundingClientRect();
        selectionBox.startSelection({
          x: e.clientX - rect.left,
          y: e.clientY - rect.top,
        });
      }
    },
    [selectionBox]
  );

  // Handle background click (deselect all)
  const handleBackgroundClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === gridRef.current && !selectionBox.isSelecting) {
        selection.clearSelection();
        if (onBackgroundClick) {
          onBackgroundClick();
        }
      }
    },
    [selection, onBackgroundClick, selectionBox.isSelecting]
  );

  // Handle global mouse move (for dragging and selection box)
  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (drag.isDragging) {
        drag.updateDrag({ x: e.clientX, y: e.clientY });
      } else if (selectionBox.isSelecting && gridRef.current) {
        const rect = gridRef.current.getBoundingClientRect();
        selectionBox.updateSelection({
          x: e.clientX - rect.left,
          y: e.clientY - rect.top,
        });
      }
    },
    [drag, selectionBox]
  );

  // Handle global mouse up (end drag or selection)
  const handleMouseUp = useCallback(() => {
    if (drag.isDragging) {
      drag.endDrag();
    } else if (selectionBox.isSelecting) {
      selectionBox.endSelection();
    }
  }, [drag, selectionBox]);

  // Global mouse listeners for drag and selection
  React.useEffect(() => {
    if (drag.isDragging || selectionBox.isSelecting) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);

      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [drag.isDragging, selectionBox.isSelecting, handleMouseMove, handleMouseUp]);

  return (
    <>
      {/* Search Bar */}
      {enableSearch && (showSearch || searchState.isActive) && (
        <Search
          onEscape={() => setShowSearch(false)}
          onBlur={() => !searchState.isActive && setShowSearch(false)}
          autoFocus={showSearch}
        />
      )}

      {/* Icon Grid */}
      <div
        className="icon-grid"
        ref={gridRef}
        onClick={handleBackgroundClick}
        onMouseDown={handleBackgroundMouseDown}
      >
        {/* Render icons */}
        {visibleIcons.map((icon) => (
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

        {/* Selection box */}
        {selectionBox.selectionBox && selectionBox.isSelecting && (
          <div
            className="icon-grid-selection-box"
            style={{
              position: "absolute",
              ...getSelectionBounds(selectionBox.selectionBox),
            }}
          />
        )}
      </div>
    </>
  );
};

Grid.displayName = "IconGrid";

