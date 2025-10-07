/**
 * Dock Item Component
 * Individual dock item with sortable support and tooltip
 */

import React, { useCallback, useMemo } from "react";
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

  // Memoize button to prevent re-creation
  const button = useMemo(() => (
    <button
      className={`dock-item ${item.pinned ? "pinned" : ""}`}
      onClick={handleClick}
      disabled={disabled}
    >
      <span className="dock-icon">{item.icon}</span>
      {item.pinned && <span className="dock-pin">📌</span>}
    </button>
  ), [item.pinned, item.icon, handleClick, disabled]);

  return (
    <SortableItem id={item.id} disabled={disabled} className="dock-item-wrapper">
      <Tooltip content={item.label} delay={500}>
        {button}
      </Tooltip>
    </SortableItem>
  );
}, (prevProps, nextProps) => {
  // Custom comparison function for memo
  return (
    prevProps.item.id === nextProps.item.id &&
    prevProps.item.label === nextProps.item.label &&
    prevProps.item.icon === nextProps.item.icon &&
    prevProps.item.pinned === nextProps.item.pinned &&
    prevProps.disabled === nextProps.disabled
  );
});

DockItem.displayName = "DockItem";
