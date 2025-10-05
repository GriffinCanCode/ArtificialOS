/**
 * Progress Component Renderer
 * Renders dynamic progress bar components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { progressVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Progress: React.FC<BaseComponentProps> = ({ component }) => {
  const progressValue = component.props?.value || 0;
  const progressMax = component.props?.max || 100;
  const progressPercent = (progressValue / progressMax) * 100;

  return (
    <div
      className={cn(
        progressVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
        })
      )}
      style={component.props?.style}
    >
      <div className="progress-bar" style={{ width: `${progressPercent}%` }} />
    </div>
  );
};

Progress.displayName = "Progress";
