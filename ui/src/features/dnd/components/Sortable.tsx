/**
 * Sortable Component
 * Generic sortable list wrapper using @dnd-kit
 * Optimized for performance with many items
 */

import React, { useMemo } from "react";
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

  const sortingStrategy = useMemo(() => {
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
  }, [strategy]);

  const activeItem = useMemo(
    () => items.find((item) => item.id === activeId),
    [items, activeId]
  );

  // Memoize rendered items to prevent unnecessary re-renders
  const renderedItems = useMemo(
    () => items.map((item, i) => renderItem(item, i)),
    [items, renderItem]
  );

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <SortableContext items={items} strategy={sortingStrategy} disabled={disabled}>
        <div className={className}>{children || renderedItems}</div>
      </SortableContext>

      <DragOverlay dropAnimation={null}>
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
