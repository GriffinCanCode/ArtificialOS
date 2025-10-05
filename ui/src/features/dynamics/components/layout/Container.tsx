/**
 * Container Component Renderer
 * Renders dynamic container components with layout options
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { containerVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "../../core/constants";
import { ComponentRenderer } from "../../rendering/renderer";
import { VirtualizedList } from "../../rendering/virtual";

export const Container: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const containerClassName = cn(
    containerVariants({
      layout: component.props?.layout as any,
      spacing: component.props?.spacing as any,
      padding: component.props?.padding as any,
      align: component.props?.align as any,
      justify: component.props?.justify as any,
    }),
    // Add semantic role class if present
    component.props?.role ? `semantic-${component.props.role}` : null
  );

  const containerStyle = {
    gap: component.props?.gap ? `${component.props.gap}px` : undefined,
    ...component.props?.style,
  };

  // Use virtual scrolling for large lists
  if (component.children && component.children.length >= VIRTUAL_SCROLL_THRESHOLD) {
    return (
      <div className={containerClassName} style={containerStyle}>
        <VirtualizedList
          children={component.children}
          state={state}
          executor={executor}
          itemHeight={component.props?.itemHeight || DEFAULT_ITEM_HEIGHT}
          maxHeight={component.props?.maxHeight || 600}
          layout={(component.props?.layout || "vertical") as "vertical" | "horizontal" | "grid"}
          className=""
          ComponentRenderer={ComponentRenderer}
        />
      </div>
    );
  }

  // Normal rendering for small lists
  return (
    <div
      className={containerClassName}
      style={containerStyle}
      data-role={component.props?.role || undefined}
    >
      {component.children?.map((child) => (
        <ComponentRenderer key={child.id} component={child} state={state} executor={executor} />
      ))}
    </div>
  );
};

Container.displayName = "Container";
