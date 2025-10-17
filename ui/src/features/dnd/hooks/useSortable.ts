/**
 * Sortable Hook
 * React hook for sortable drag and drop lists
 */

import { useState, useCallback } from "react";
import type { DragEndEvent, DragStartEvent, DragOverEvent, UniqueIdentifier } from "@dnd-kit/core";
import type { SortableItem, SortHandler } from "../core/types";
import { arrayMove } from "../core/utils";

interface UseSortableConfig<T extends SortableItem> {
  items: T[];
  onSort?: SortHandler;
  disabled?: boolean;
}

interface UseSortableReturn<T extends SortableItem> {
  items: T[];
  activeId: UniqueIdentifier | null;
  handleDragStart: (event: DragStartEvent) => void;
  handleDragEnd: (event: DragEndEvent) => void;
  handleDragCancel: () => void;
  handleDragOver: (event: DragOverEvent) => void;
  setItems: (items: T[]) => void;
  moveItem: (from: number, to: number) => void;
}

/**
 * Hook for managing sortable drag and drop lists
 */
export function useSortable<T extends SortableItem>({
  items: initialItems,
  onSort,
  disabled = false,
}: UseSortableConfig<T>): UseSortableReturn<T> {
  const [items, setItems] = useState<T[]>(initialItems);
  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);

  const handleDragStart = useCallback(
    (event: DragStartEvent) => {
      if (disabled) return;
      setActiveId(event.active.id);
    },
    [disabled]
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;

      if (!over || active.id === over.id) {
        setActiveId(null);
        return;
      }

      const oldIndex = items.findIndex((item) => item.id === active.id);
      const newIndex = items.findIndex((item) => item.id === over.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        const newItems = arrayMove(items, oldIndex, newIndex);
        setItems(newItems);

        if (onSort) {
          onSort({
            activeId: active.id,
            overId: over.id,
            oldIndex,
            newIndex,
          });
        }
      }

      setActiveId(null);
    },
    [items, onSort]
  );

  const handleDragCancel = useCallback(() => {
    setActiveId(null);
  }, []);

  const handleDragOver = useCallback(
    (event: DragOverEvent) => {
      const { active, over } = event;

      if (!over || active.id === over.id) return;

      const oldIndex = items.findIndex((item) => item.id === active.id);
      const newIndex = items.findIndex((item) => item.id === over.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        setItems((prev) => arrayMove(prev, oldIndex, newIndex));
      }
    },
    [items]
  );

  const moveItem = useCallback((from: number, to: number) => {
    setItems((prev) => arrayMove(prev, from, to));
  }, []);

  return {
    items,
    activeId,
    handleDragStart,
    handleDragEnd,
    handleDragCancel,
    handleDragOver,
    setItems,
    moveItem,
  };
}
