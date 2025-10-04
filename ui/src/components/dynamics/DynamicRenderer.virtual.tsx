/**
 * DynamicRenderer Virtual Scrolling
 * Smart wrapper for rendering large lists efficiently
 */

import React, { useRef, useCallback } from "react";
import { List as VirtualList } from "react-window";
import { UIComponent } from "../../store/appStore";
import { ComponentState } from "./DynamicRenderer.state";
import { ToolExecutor } from "./DynamicRenderer.executor";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "./DynamicRenderer.constants";
import { logger } from "../../utils/monitoring/logger";

// Forward declaration to avoid circular dependency
// ComponentRenderer will be passed as a prop
type ComponentRendererType = React.ComponentType<{
  component: UIComponent;
  state: ComponentState;
  executor: ToolExecutor;
}>;

// ============================================================================
// Virtual Scrolling Wrapper
// ============================================================================

interface VirtualizedListProps {
  children: UIComponent[];
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
 * Only activates when children count exceeds threshold
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
    const containerRef = useRef<HTMLDivElement>(null);

    // Don't use virtual scrolling for small lists
    if (children.length < VIRTUAL_SCROLL_THRESHOLD) {
      return (
        <div className={className} ref={containerRef}>
          {children.map((child: UIComponent, idx: number) => (
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
    const itemsPerRow = isGrid ? columns : 1;
    const rowCount = isGrid ? Math.ceil(children.length / itemsPerRow) : children.length;
    const actualItemHeight = isGrid ? itemHeight : itemHeight;

    // Memoized row renderer for virtual list - prevents unnecessary re-renders
    const Row = useCallback(
      ({ index, style }: { index: number; style: React.CSSProperties }) => {
        if (isGrid) {
          // Render a row of grid items
          const startIdx = index * itemsPerRow;
          const rowItems = children.slice(startIdx, startIdx + itemsPerRow);

          return (
            <div
              style={{
                ...style,
                display: "grid",
                gridTemplateColumns: `repeat(${itemsPerRow}, 1fr)`,
                gap: "1rem",
              }}
            >
              {rowItems.map((child: UIComponent, idx: number) => (
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
          // Render a single item
          const child = children[index];
          if (!child) return null; // Safety check

          return (
            <div style={style}>
              <ComponentRendererComponent
                key={`${child.id}-${index}`}
                component={child}
                state={state}
                executor={executor}
              />
            </div>
          );
        }
      },
      [isGrid, itemsPerRow, children, state, executor, ComponentRendererComponent]
    );

    logger.info("Using virtual scrolling", {
      component: "VirtualizedList",
      itemCount: children.length,
      rowCount,
      itemsPerRow,
      layout,
    });

    return (
      <div className={className} ref={containerRef}>
        <VirtualList
          defaultHeight={maxHeight}
          rowCount={rowCount}
          rowHeight={actualItemHeight}
          overscanCount={5} // Render 5 extra items above/below viewport for smooth scrolling
          rowComponent={Row as any}
          rowProps={{} as any}
        >
          {null}
        </VirtualList>
      </div>
    );
  }
);

VirtualizedList.displayName = "VirtualizedList";
