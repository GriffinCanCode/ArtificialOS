/**
 * Button Component Renderer
 * Renders dynamic button components with variants
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { buttonVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Button: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <button
      className={cn(
        buttonVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
          fullWidth: component.props?.fullWidth,
        })
      )}
      onClick={() => handleEvent("click")}
      disabled={component.props?.disabled}
      style={component.props?.style}
    >
      {component.props?.text || "Button"}
    </button>
  );
};

Button.displayName = "Button";
