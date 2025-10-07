/**
 * Virtual Grid Component
 * High-performance grid virtualization with dynamic columns
 */

import React, { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { VirtualGridConfig, VirtualGridItemProps } from "./types";
import { calculateGridDimensions } from "./utils";

// ============================================================================
// Virtual Grid
// ============================================================================

interface VirtualGridProps<T = any> extends VirtualGridConfig<T> {
  children: (props: VirtualGridItemProps<T>) => React.ReactNode;
}

/**
 * Generic virtual grid with dynamic sizing support
 */
export const VirtualGrid = <T,>({
  items,
  columns,
  rowHeight = 200,
  height = 600,
  width = "100%",
  gap = 16,
  className = "",
  overscan = 3,
  children,
}: VirtualGridProps<T>) => {
  const parentRef = useRef<HTMLDivElement>(null);

  const { rows } = calculateGridDimensions(items.length, columns);

  // Virtualize rows
  const rowVirtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => parentRef.current,
    estimateSize: typeof rowHeight === "function" ? rowHeight : () => rowHeight,
    overscan,
  });

  const virtualRows = rowVirtualizer.getVirtualItems();

  return (
    <div
      ref={parentRef}
      className={className}
      style={{
        height: typeof height === "number" ? `${height}px` : height,
        width: typeof width === "number" ? `${width}px` : width,
        overflow: "auto",
        contain: "strict",
      }}
    >
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualRows.map((virtualRow) => {
          const rowIndex = virtualRow.index;
          const startIndex = rowIndex * columns;

          return (
            <div
              key={virtualRow.index}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
                display: "grid",
                gridTemplateColumns: `repeat(${columns}, 1fr)`,
                gap: `${gap}px`,
              }}
            >
              {Array.from({ length: columns }).map((_, colIndex) => {
                const itemIndex = startIndex + colIndex;
                if (itemIndex >= items.length) return null;

                const item = items[itemIndex];
                const style: React.CSSProperties = {
                  width: "100%",
                  height: "100%",
                };

                return (
                  <div key={colIndex}>
                    {children({
                      item,
                      rowIndex,
                      columnIndex: colIndex,
                      index: itemIndex,
                      style,
                    })}
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
    </div>
  );
};

VirtualGrid.displayName = "VirtualGrid";

// ============================================================================
// Masonry Grid (Variable Heights)
// ============================================================================

interface MasonryGridProps<T = any> extends VirtualGridConfig<T> {
  getItemHeight?: (item: T, index: number) => number;
  children: (props: VirtualGridItemProps<T>) => React.ReactNode;
}

/**
 * Virtual grid with variable row heights (masonry layout)
 */
export const MasonryGrid = <T,>({
  items,
  columns,
  getItemHeight,
  ...rest
}: MasonryGridProps<T>) => {
  const rowHeight = (rowIndex: number): number => {
    if (!getItemHeight) return 200;

    // Get max height in this row
    const startIndex = rowIndex * columns;
    let maxHeight = 0;

    for (let i = 0; i < columns; i++) {
      const itemIndex = startIndex + i;
      if (itemIndex >= items.length) break;
      const height = getItemHeight(items[itemIndex], itemIndex);
      maxHeight = Math.max(maxHeight, height);
    }

    return maxHeight || 200;
  };

  return <VirtualGrid items={items} columns={columns} rowHeight={rowHeight} {...rest} />;
};

MasonryGrid.displayName = "MasonryGrid";
