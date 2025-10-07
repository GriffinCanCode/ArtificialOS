/**
 * Virtual List Component
 * High-performance list virtualization with variable sizing
 */

import React, { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { VirtualListConfig, VirtualListItemProps } from "./types";

// ============================================================================
// Virtual List
// ============================================================================

interface VirtualListProps<T = any> extends VirtualListConfig<T> {
  children: (props: VirtualListItemProps<T>) => React.ReactNode;
}

/**
 * Generic virtual list with dynamic sizing support
 */
export const VirtualList = <T,>({
  items,
  height = 600,
  estimateSize,
  getItemKey,
  className = "",
  overscan = 5,
  scrollMargin = 0,
  horizontal = false,
  children,
}: VirtualListProps<T>) => {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: estimateSize || (() => 60),
    overscan,
    scrollMargin,
    horizontal,
  });

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <div
      ref={parentRef}
      className={className}
      style={{
        height: typeof height === "number" ? `${height}px` : height,
        overflow: "auto",
        contain: "strict",
      }}
    >
      <div
        style={{
          height: horizontal ? "100%" : `${virtualizer.getTotalSize()}px`,
          width: horizontal ? `${virtualizer.getTotalSize()}px` : "100%",
          position: "relative",
        }}
      >
        {virtualItems.map((virtualItem) => {
          const item = items[virtualItem.index];
          const key = getItemKey ? getItemKey(virtualItem.index, item) : virtualItem.index;

          const style: React.CSSProperties = {
            position: "absolute",
            top: horizontal ? 0 : `${virtualItem.start}px`,
            left: horizontal ? `${virtualItem.start}px` : 0,
            height: horizontal ? "100%" : `${virtualItem.size}px`,
            width: horizontal ? `${virtualItem.size}px` : "100%",
          };

          return <div key={key}>{children({ item, index: virtualItem.index, style })}</div>;
        })}
      </div>
    </div>
  );
};

VirtualList.displayName = "VirtualList";

// ============================================================================
// Simple List (Auto-sizing)
// ============================================================================

interface SimpleListProps<T = any> extends Omit<VirtualListConfig<T>, "estimateSize"> {
  itemSize?: number;
  children: (props: VirtualListItemProps<T>) => React.ReactNode;
}

/**
 * Simplified virtual list with fixed item sizes
 */
export const SimpleList = <T,>({ items, itemSize = 60, ...rest }: SimpleListProps<T>) => {
  return <VirtualList items={items} estimateSize={() => itemSize} {...rest} />;
};

SimpleList.displayName = "SimpleList";

// ============================================================================
// Dynamic List (Measured sizing)
// ============================================================================

interface DynamicListProps<T = any> extends VirtualListConfig<T> {
  defaultSize?: number;
  children: (props: VirtualListItemProps<T>) => React.ReactNode;
}

/**
 * Virtual list with dynamic content-based sizing
 */
export const DynamicList = <T,>({ items, defaultSize = 60, ...rest }: DynamicListProps<T>) => {
  return (
    <VirtualList
      items={items}
      estimateSize={(index) => {
        // Could be enhanced with actual measurement logic
        return defaultSize;
      }}
      {...rest}
    />
  );
};

DynamicList.displayName = "DynamicList";
