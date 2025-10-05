/**
 * Audio Component Renderer
 * Renders dynamic audio player components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { audioVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Audio: React.FC<BaseComponentProps> = ({ component }) => {
  return (
    <audio
      className={cn(
        audioVariants({
          variant: component.props?.variant as any,
        })
      )}
      src={component.props?.src}
      controls={component.props?.controls !== false}
      autoPlay={component.props?.autoPlay}
      loop={component.props?.loop}
      style={component.props?.style}
    />
  );
};

Audio.displayName = "Audio";
