/**
 * Iframe Component Renderer
 * Renders dynamic iframe/embedded content components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { iframeVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Iframe: React.FC<BaseComponentProps> = ({ component }) => {
  return (
    <iframe
      className={cn(
        iframeVariants({
          bordered: component.props?.bordered,
          rounded: component.props?.rounded,
        })
      )}
      src={component.props?.src}
      title={component.props?.title || "iframe"}
      width={component.props?.width || "100%"}
      height={component.props?.height || 400}
      style={component.props?.style}
      sandbox={component.props?.sandbox}
    />
  );
};

Iframe.displayName = "Iframe";
