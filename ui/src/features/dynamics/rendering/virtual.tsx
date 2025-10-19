/**
 * DynamicRenderer Virtual Scrolling
 * Enhanced wrapper using TanStack Virtual for better performance
 */

import React, { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { BlueprintComponent } from "../../../core/store/appStore";
import { ComponentState } from "../state/state";
import { ToolExecutor } from "../execution/executor";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "../core/constants";
import { logger } from "../../../core/monitoring/core/logger";

// Forward declaration to avoid circular dependency
type ComponentRendererType = React.ComponentType<{
  component: BlueprintComponent;
  state: ComponentState;
  executor: ToolExecutor;
}>;

// ============================================================================
// Virtual Scrolling Wrapper
// ============================================================================

interface VirtualizedListProps {
  children: BlueprintComponent[];
  state: ComponentState;
  executor: ToolExecutor;
  itemHeight?: number;
  maxHeight?: number;
  className?: string;
  layout?: "vertical" | "horizontal" | "grid";
  columns?: number;
  ComponentRenderer: ComponentRendererType;
}

/**
 * Smart wrapper that enables virtual scrolling for large lists
 * Uses TanStack Virtual for better performance and flexibility
 */
export const VirtualizedList: React.FC<VirtualizedListProps> = React.memo(
  ({
    children,
    state,
    executor,
    itemHeight = DEFAULT_ITEM_HEIGHT,
    maxHeight = 600,
    className = "",
    layout = "vertical",
    columns = 1,
    ComponentRenderer: ComponentRendererComponent,
  }) => {
    const parentRef = useRef<HTMLDivElement>(null);

    // Don't use virtual scrolling for small lists
    if (children.length < VIRTUAL_SCROLL_THRESHOLD) {
      return (
        <div className={className} ref={parentRef}>
          {children.map((child: BlueprintComponent, idx: number) => (
            <ComponentRendererComponent
              key={`${child.id}-${idx}`}
              component={child}
              state={state}
              executor={executor}
            />
          ))}
        </div>
      );
    }

    // For grid layouts, calculate rows
    const isGrid = layout === "grid";
    const isHorizontal = layout === "horizontal";
    const itemsPerRow = isGrid ? columns : 1;
    const rowCount = isGrid ? Math.ceil(children.length / itemsPerRow) : children.length;

    // Setup virtualizer with TanStack Virtual
    const virtualizer = useVirtualizer({
      count: rowCount,
      getScrollElement: () => parentRef.current,
      estimateSize: () => itemHeight,
      overscan: 5,
      horizontal: isHorizontal && !isGrid,
    });

    const virtualItems = virtualizer.getVirtualItems();

    logger.info("Using virtual scrolling", {
      component: "VirtualizedList",
      itemCount: children.length,
      rowCount,
      itemsPerRow,
      layout,
      library: "tanstack-virtual",
    });

    return (
      <div
        ref={parentRef}
        className={className}
        style={{
          height: `${maxHeight}px`,
          overflow: "auto",
          contain: "strict",
        }}
      >
        <div
          style={{
            height: isHorizontal && !isGrid ? "100%" : `${virtualizer.getTotalSize()}px`,
            width: isHorizontal && !isGrid ? `${virtualizer.getTotalSize()}px` : "100%",
            position: "relative",
          }}
        >
          {virtualItems.map((virtualRow) => {
            if (isGrid) {
              // Render grid row
              const startIdx = virtualRow.index * itemsPerRow;
              const rowItems = children.slice(startIdx, startIdx + itemsPerRow);

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
                    gridTemplateColumns: `repeat(${itemsPerRow}, 1fr)`,
                    gap: "1rem",
                  }}
                >
                  {rowItems.map((child: BlueprintComponent, idx: number) => (
                    <ComponentRendererComponent
                      key={`${child.id}-${startIdx + idx}`}
                      component={child}
                      state={state}
                      executor={executor}
                    />
                  ))}
                </div>
              );
            } else {
              // Render single item (list or horizontal)
              const child = children[virtualRow.index];
              if (!child) return null;

              return (
                <div
                  key={virtualRow.index}
                  style={{
                    position: "absolute",
                    top: isHorizontal ? 0 : `${virtualRow.start}px`,
                    left: isHorizontal ? `${virtualRow.start}px` : 0,
                    height: isHorizontal ? "100%" : `${virtualRow.size}px`,
                    width: isHorizontal ? `${virtualRow.size}px` : "100%",
                  }}
                >
                  <ComponentRendererComponent
                    key={`${child.id}-${virtualRow.index}`}
                    component={child}
                    state={state}
                    executor={executor}
                  />
                </div>
              );
            }
          })}
        </div>
      </div>
    );
  }
);

VirtualizedList.displayName = "VirtualizedList";
