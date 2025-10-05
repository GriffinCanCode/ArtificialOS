/**
 * Image Component Renderer
 * Renders dynamic image components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { imageVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Image: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <img
      className={cn(
        imageVariants({
          fit: component.props?.fit as any,
          rounded: component.props?.rounded as any,
        })
      )}
      src={component.props?.src}
      alt={component.props?.alt || ""}
      width={component.props?.width}
      height={component.props?.height}
      onClick={() => handleEvent("click")}
      style={component.props?.style}
    />
  );
};

Image.displayName = "Image";
