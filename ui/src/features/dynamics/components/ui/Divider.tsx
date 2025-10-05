/**
 * Divider Component Renderer
 * Renders dynamic divider/separator components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { dividerVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Divider: React.FC<BaseComponentProps> = ({ component }) => {
  return (
    <div
      className={cn(
        dividerVariants({
          orientation: component.props?.orientation as any,
          variant: component.props?.variant as any,
        })
      )}
      style={component.props?.style}
    />
  );
};

Divider.displayName = "Divider";
