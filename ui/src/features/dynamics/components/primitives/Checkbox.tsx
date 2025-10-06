/**
 * Checkbox Component Renderer
 * Renders dynamic checkbox components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { checkboxVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Checkbox: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);

  const checked = localState ?? component.props?.checked ?? false;

  return (
    <label className="dynamic-checkbox-wrapper" style={component.props?.style}>
      <input
        type="checkbox"
        className={cn(
          checkboxVariants({
            size: component.props?.size as any,
            variant: component.props?.variant as any,
            disabled: component.props?.disabled,
          })
        )}
        checked={checked}
        disabled={component.props?.disabled}
        onChange={(e) => {
          const newValue = e.target.checked;
          state.set(component.id, newValue);
          handleEvent("change", { checked: newValue });
        }}
      />
      {component.props?.label && <span className="checkbox-label">{component.props.label}</span>}
    </label>
  );
};

Checkbox.displayName = "Checkbox";
