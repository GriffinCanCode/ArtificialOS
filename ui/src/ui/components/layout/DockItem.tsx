/**
 * Dock Item Component
 * Individual dock item with sortable support and tooltip
 */

import React, { useCallback } from "react";
import { SortableItem } from "../../../features/dnd/components/SortableItem";
import { Tooltip } from "../../../features/floating";
import type { DockItem as DockItemType } from "../../../features/dnd";
import "./DockItem.css";

// ============================================================================
// Types
// ============================================================================

interface DockItemProps {
  item: DockItemType;
  onClick: (action: string) => void;
  disabled?: boolean;
}

// ============================================================================
// Component
// ============================================================================

export const DockItem: React.FC<DockItemProps> = React.memo(({ item, onClick, disabled = false }) => {
  const handleClick = useCallback(() => {
    onClick(item.action);
  }, [onClick, item.action]);

  return (
    <SortableItem id={item.id} disabled={disabled} className="dock-item-wrapper">
      <button
        className={`dock-item ${item.pinned ? "pinned" : ""}`}
        onClick={handleClick}
        disabled={disabled}
        title={item.label}
      >
        <span className="dock-icon">{item.icon}</span>
        {item.pinned && <span className="dock-pin">📌</span>}
      </button>
    </SortableItem>
  );
});

DockItem.displayName = "DockItem";
