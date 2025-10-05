/**
 * Grid Component Renderer
 * Renders dynamic grid layout components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { gridVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "../../core/constants";
import { ComponentRenderer } from "../../rendering/renderer";
import { VirtualizedList } from "../../rendering/virtual";

export const Grid: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const gridClassName = cn(
    gridVariants({
      columns: component.props?.columns as any,
      spacing: component.props?.spacing as any,
      responsive: component.props?.responsive,
    })
  );

  const gridStyle = {
    gridTemplateColumns: component.props?.columns
      ? `repeat(${component.props.columns}, 1fr)`
      : undefined,
    gap: component.props?.gap ? `${component.props.gap}px` : undefined,
    ...component.props?.style,
  };

  // Use virtual scrolling for large grids
  if (component.children && component.children.length >= VIRTUAL_SCROLL_THRESHOLD) {
    return (
      <div className={gridClassName} style={gridStyle}>
        <VirtualizedList
          children={component.children}
          state={state}
          executor={executor}
          itemHeight={component.props?.itemHeight || DEFAULT_ITEM_HEIGHT}
          maxHeight={component.props?.maxHeight || 600}
          layout="grid"
          columns={component.props?.columns || 3}
          className=""
          ComponentRenderer={ComponentRenderer}
        />
      </div>
    );
  }

  // Normal rendering for small grids
  return (
    <div className={gridClassName} style={gridStyle}>
      {component.children?.map((child) => (
        <ComponentRenderer key={child.id} component={child} state={state} executor={executor} />
      ))}
    </div>
  );
};

Grid.displayName = "Grid";
