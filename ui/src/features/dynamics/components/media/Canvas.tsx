/**
 * Canvas Component Renderer
 * Renders dynamic canvas components for drawing
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { canvasVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Canvas: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <canvas
      ref={(ref) => {
        if (ref && component.props?.onMount) {
          state.set(`${component.id}_canvas`, ref);
          handleEvent("mount", { canvas: ref });
        }
      }}
      className={cn(
        canvasVariants({
          bordered: component.props?.bordered,
        })
      )}
      width={component.props?.width || 300}
      height={component.props?.height || 150}
      style={component.props?.style}
    />
  );
};

Canvas.displayName = "Canvas";
