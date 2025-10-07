/**
 * Dock Item Component
 * Individual dock item with sortable support
 */

import React from "react";
import { SortableItem } from "../../../features/dnd";
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

export const DockItem: React.FC<DockItemProps> = ({ item, onClick, disabled = false }) => {
  return (
    <SortableItem id={item.id} disabled={disabled} className="dock-item-wrapper">
      <button
        className={`dock-item ${item.pinned ? "pinned" : ""}`}
        onClick={() => onClick(item.action)}
        title={item.label}
        disabled={disabled}
      >
        <span className="dock-icon">{item.icon}</span>
        {item.pinned && <span className="dock-pin">📌</span>}
      </button>
    </SortableItem>
  );
}
