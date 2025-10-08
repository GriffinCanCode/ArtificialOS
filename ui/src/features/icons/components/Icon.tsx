/**
 * Icon Component
 * Desktop icon with drag, selection, and double-click support
 */

import React, { useCallback, useState, useRef } from "react";
import type { Icon as IconType } from "../core/types";
import { gridToPixel } from "../core/grid";
import { DEFAULT_GRID_CONFIG, DOUBLE_CLICK_THRESHOLD } from "../core/types";
import "./Icon.css";

// ============================================================================
// Icon Component Props
// ============================================================================

export interface IconProps {
  icon: IconType;
  onSelect: (iconId: string, modifiers: { shift: boolean; ctrl: boolean; meta: boolean }) => void;
  onDoubleClick: (iconId: string) => void;
  onDragStart: (iconId: string, position: { x: number; y: number }) => void;
  onContextMenu?: (iconId: string, event: React.MouseEvent) => void;
}

// ============================================================================
// Icon Component
// ============================================================================

export const Icon: React.FC<IconProps> = React.memo(
  ({ icon, onSelect, onDoubleClick, onDragStart, onContextMenu }) => {
    const [isPressed, setIsPressed] = useState(false);
    const lastClickTime = useRef(0);
    const dragStartPos = useRef<{ x: number; y: number } | null>(null);

    // Convert grid position to pixels
    const pixelPos = gridToPixel(icon.position, DEFAULT_GRID_CONFIG);

    // Check if this is a duplicate (hide duplicates by making them invisible)
    const isDuplicate = icon.metadata.type === "native" && icon.position.row === 0 && icon.position.col > 1;

    // Handle mouse down (start of potential drag)
    const handleMouseDown = useCallback(
      (e: React.MouseEvent) => {
        e.preventDefault();

        setIsPressed(true);
        dragStartPos.current = { x: e.clientX, y: e.clientY };

        // Check for double-click
        const now = Date.now();
        if (now - lastClickTime.current < DOUBLE_CLICK_THRESHOLD) {
          onDoubleClick(icon.id);
          lastClickTime.current = 0;
          return;
        }
        lastClickTime.current = now;

        // Handle selection
        onSelect(icon.id, {
          shift: e.shiftKey,
          ctrl: e.ctrlKey,
          meta: e.metaKey,
        });
      },
      [icon.id, onSelect, onDoubleClick]
    );

    // Handle mouse move (drag)
    const handleMouseMove = useCallback(
      (e: MouseEvent) => {
        if (!isPressed || !dragStartPos.current) return;

        // Initiate drag if moved enough
        const dx = e.clientX - dragStartPos.current.x;
        const dy = e.clientY - dragStartPos.current.y;
        const distance = Math.sqrt(dx * dx + dy * dy);

        if (distance > 5) {
          onDragStart(icon.id, { x: e.clientX, y: e.clientY });
          setIsPressed(false);
          dragStartPos.current = null;
        }
      },
      [isPressed, icon.id, onDragStart]
    );

    // Handle mouse up
    const handleMouseUp = useCallback(() => {
      setIsPressed(false);
      dragStartPos.current = null;
    }, []);

    // Mouse move/up listeners
    React.useEffect(() => {
      if (isPressed) {
        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("mouseup", handleMouseUp);

        return () => {
          window.removeEventListener("mousemove", handleMouseMove);
          window.removeEventListener("mouseup", handleMouseUp);
        };
      }
    }, [isPressed, handleMouseMove, handleMouseUp]);

    // Handle context menu
    const handleContextMenu = useCallback(
      (e: React.MouseEvent) => {
        e.preventDefault();
        if (onContextMenu) {
          onContextMenu(icon.id, e);
        }
      },
      [icon.id, onContextMenu]
    );

    return (
      <div
        className={`desktop-icon ${icon.isSelected ? "selected" : ""} ${icon.isDragging ? "dragging" : ""} ${isDuplicate ? "duplicate-hidden" : ""}`}
        style={{
          position: "absolute",
          left: `${pixelPos.x}px`,
          top: `${pixelPos.y}px`,
          zIndex: icon.zIndex,
        }}
        onMouseDown={handleMouseDown}
        onContextMenu={handleContextMenu}
      >
        <div className="desktop-icon-image">{icon.icon}</div>
        <div className="desktop-icon-label">{icon.label}</div>
      </div>
    );
  }
);

Icon.displayName = "Icon";

