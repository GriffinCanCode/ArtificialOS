/**
 * Sortable Item Component
 * Individual item wrapper for sortable lists
 */

import React from "react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

// ============================================================================
// Types
// ============================================================================

interface SortableItemProps {
  id: string;
  children: React.ReactNode;
  disabled?: boolean;
  className?: string;
}

// ============================================================================
// Component
// ============================================================================

export const SortableItem: React.FC<SortableItemProps> = ({
  id,
  children,
  disabled = false,
  className,
}) => {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled,
  });

  const style: React.CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
    cursor: disabled ? "default" : "grab",
    // Ensure pointer events work properly for nested interactive elements
    touchAction: "none",
  };

  // Don't spread listeners on disabled items to allow tooltips and other interactions
  const dragHandlers = disabled ? {} : listeners;

  return (
    <div ref={setNodeRef} style={style} className={className} {...attributes} {...dragHandlers}>
      {children}
    </div>
  );
};
