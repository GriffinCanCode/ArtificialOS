/**
 * List Component Renderer
 * Renders dynamic list components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { listVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { ComponentRenderer } from "../../rendering/renderer";

export const List: React.FC<BaseComponentProps> = ({ component, state, executor }) => {

  return (
    <div
      className={cn(
        listVariants({
          variant: component.props?.variant as any,
          spacing: component.props?.spacing as any,
        })
      )}
      style={component.props?.style}
    >
      {component.children?.map((child) => (
        <div key={child.id} className="list-item">
          <ComponentRenderer component={child} state={state} executor={executor} />
        </div>
      ))}
    </div>
  );
};

List.displayName = "List";
