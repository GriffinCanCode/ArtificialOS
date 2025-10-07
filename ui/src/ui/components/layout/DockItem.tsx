/**
 * Dock Item Component
 * Individual dock item with sortable support and tooltip
 */

import React, { useCallback, useMemo } from "react";
import { SortableItem } from "../../../features/dnd/components/SortableItem";
import { Tooltip, ContextMenu } from "../../../features/floating";
import type { DockItem as DockItemType } from "../../../features/dnd";
import "./DockItem.css";

// ============================================================================
// Types
// ============================================================================

interface DockItemProps {
  item: DockItemType;
  onClick: (action: string) => void;
  onContextMenuAction?: (action: string, itemId: string) => void;
  disabled?: boolean;
}

// ============================================================================
// Component
// ============================================================================

export const DockItem: React.FC<DockItemProps> = React.memo(({ item, onClick, onContextMenuAction, disabled = false }) => {
  const handleClick = useCallback(() => {
    onClick(item.action);
  }, [onClick, item.action]);

  const handleContextMenuSelect = useCallback((action: string) => {
    onContextMenuAction?.(action, item.id);
  }, [onContextMenuAction, item.id]);

  // Context menu items
  const contextMenuItems = useMemo(() => [
    {
      value: 'toggle-pin',
      label: item.pinned ? 'Unpin from Dock' : 'Pin to Dock',
      icon: item.pinned ? '📍' : '📌',
    },
    {
      divider: true,
      value: 'divider-1',
      label: '',
    },
    {
      value: 'remove',
      label: 'Remove from Dock',
      icon: '🗑️',
    },
  ], [item.pinned]);

  return (
    <SortableItem id={item.id} disabled={disabled} className="dock-item-wrapper">
      <ContextMenu items={contextMenuItems} onSelect={handleContextMenuSelect}>
        <Tooltip content={item.label} delay={500}>
          <button
            className={`dock-item ${item.pinned ? "pinned" : ""}`}
            onClick={handleClick}
            disabled={disabled}
          >
            <span className="dock-icon">{item.icon}</span>
            {item.pinned && <span className="dock-pin">📌</span>}
          </button>
        </Tooltip>
      </ContextMenu>
    </SortableItem>
  );
}, (prevProps, nextProps) => {
  // Custom comparison function for memo
  return (
    prevProps.item.id === nextProps.item.id &&
    prevProps.item.label === nextProps.item.label &&
    prevProps.item.icon === nextProps.item.icon &&
    prevProps.item.pinned === nextProps.item.pinned &&
    prevProps.disabled === nextProps.disabled &&
    prevProps.onContextMenuAction === nextProps.onContextMenuAction
  );
});

DockItem.displayName = "DockItem";
