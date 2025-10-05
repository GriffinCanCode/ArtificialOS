/**
 * Radio Component Renderer
 * Renders dynamic radio button components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { radioVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Radio: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);

  return (
    <label className="dynamic-radio-wrapper" style={component.props?.style}>
      <input
        type="radio"
        className={cn(
          radioVariants({
            size: component.props?.size as any,
            disabled: component.props?.disabled,
          })
        )}
        name={component.props?.name}
        value={component.props?.value}
        checked={localState === component.props?.value}
        disabled={component.props?.disabled}
        onChange={(e) => {
          const newValue = e.target.value;
          state.set(component.id, newValue);
          handleEvent("change", { value: newValue });
        }}
      />
      {component.props?.label && (
        <span className="radio-label">{component.props.label}</span>
      )}
    </label>
  );
};

Radio.displayName = "Radio";
