/**
 * Slider Component Renderer
 * Renders dynamic slider/range input components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { sliderVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Slider: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);

  return (
    <input
      type="range"
      className={cn(
        sliderVariants({
          size: component.props?.size as any,
          variant: component.props?.variant as any,
          disabled: component.props?.disabled,
        })
      )}
      min={component.props?.min || 0}
      max={component.props?.max || 100}
      step={component.props?.step || 1}
      value={localState ?? component.props?.value ?? 0}
      disabled={component.props?.disabled}
      onChange={(e) => {
        const newValue = parseFloat(e.target.value);
        state.set(component.id, newValue);
        handleEvent("change", { value: newValue });
      }}
      style={component.props?.style}
    />
  );
};

Slider.displayName = "Slider";
