/**
 * Badge Component Renderer
 * Renders dynamic badge/tag components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { badgeVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Badge: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <span
      className={cn(
        badgeVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
        })
      )}
      onClick={() => handleEvent("click")}
      style={component.props?.style}
    >
      {component.props?.text || component.props?.content}
    </span>
  );
};

Badge.displayName = "Badge";
