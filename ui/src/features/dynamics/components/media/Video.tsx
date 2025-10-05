/**
 * Video Component Renderer
 * Renders dynamic video components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { videoVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Video: React.FC<BaseComponentProps> = ({ component }) => {
  return (
    <video
      className={cn(
        videoVariants({
          fit: component.props?.fit as any,
          rounded: component.props?.rounded as any,
        })
      )}
      src={component.props?.src}
      controls={component.props?.controls !== false}
      autoPlay={component.props?.autoPlay}
      loop={component.props?.loop}
      muted={component.props?.muted}
      width={component.props?.width}
      height={component.props?.height}
      style={component.props?.style}
    />
  );
};

Video.displayName = "Video";
