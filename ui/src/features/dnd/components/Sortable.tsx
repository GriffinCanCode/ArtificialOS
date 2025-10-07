/**
 * Sortable Component
 * Generic sortable list wrapper using @dnd-kit
 */

import React from "react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragOverlay,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  horizontalListSortingStrategy,
  verticalListSortingStrategy,
  rectSortingStrategy,
} from "@dnd-kit/sortable";
import { useSortable as useSortableHook } from "../hooks/useSortable";
import type { SortableItem, SortHandler } from "../core/types";

// ============================================================================
// Types
// ============================================================================

interface SortableProps<T extends SortableItem> {
  items: T[];
  onSort?: SortHandler;
  strategy?: "horizontal" | "vertical" | "grid";
  disabled?: boolean;
  renderItem: (item: T, index: number) => React.ReactNode;
  renderOverlay?: (item: T) => React.ReactNode;
  className?: string;
  children?: React.ReactNode;
}

// ============================================================================
// Component
// ============================================================================

export function Sortable<T extends SortableItem>({
  items: initialItems,
  onSort,
  strategy = "horizontal",
  disabled = false,
  renderItem,
  renderOverlay,
  className,
  children,
}: SortableProps<T>) {
  const { items, activeId, handleDragStart, handleDragEnd, handleDragCancel } = useSortableHook({
    items: initialItems,
    onSort,
    disabled,
  });

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const getStrategy = () => {
    switch (strategy) {
      case "horizontal":
        return horizontalListSortingStrategy;
      case "vertical":
        return verticalListSortingStrategy;
      case "grid":
        return rectSortingStrategy;
      default:
        return horizontalListSortingStrategy;
    }
  };

  const activeItem = items.find((item) => item.id === activeId);

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <SortableContext items={items} strategy={getStrategy()} disabled={disabled}>
        <div className={className}>{children || items.map((item, i) => renderItem(item, i))}</div>
      </SortableContext>

      <DragOverlay>
        {activeId && activeItem ? (
          renderOverlay ? (
            renderOverlay(activeItem)
          ) : (
            <div style={{ opacity: 0.8 }}>{renderItem(activeItem, -1)}</div>
          )
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
