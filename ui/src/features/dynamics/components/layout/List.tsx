/**
 * List Component Renderer
 * Renders dynamic list components with optional virtualization
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { listVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "../../core/constants";
import { ComponentRenderer } from "../../rendering/renderer";
import { VirtualizedList } from "../../rendering/virtual";

export const List: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const listClassName = cn(
    listVariants({
      variant: component.props?.variant as any,
      spacing: component.props?.spacing as any,
    })
  );

  // Use virtual scrolling for large lists
  if (component.children && component.children.length >= VIRTUAL_SCROLL_THRESHOLD) {
    return (
      <div className={listClassName} style={component.props?.style}>
        <VirtualizedList
          children={component.children}
          state={state}
          executor={executor}
          itemHeight={component.props?.itemHeight || DEFAULT_ITEM_HEIGHT}
          maxHeight={component.props?.maxHeight || 600}
          layout="vertical"
          className=""
          ComponentRenderer={ComponentRenderer}
        />
      </div>
    );
  }

  // Normal rendering for small lists
  return (
    <div className={listClassName} style={component.props?.style}>
      {component.children?.map((child) => (
        <div key={child.id} className="list-item">
          <ComponentRenderer component={child} state={state} executor={executor} />
        </div>
      ))}
    </div>
  );
};

List.displayName = "List";
