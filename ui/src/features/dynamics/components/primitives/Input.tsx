/**
 * Input Component Renderer
 * Renders dynamic input components with variants
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { inputVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Input: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleDebouncedEvent } = useComponent(component, state, executor);

  const value = localState ?? component.props?.value ?? "";

  return (
    <input
      className={cn(
        inputVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
          error: component.props?.error,
          disabled: component.props?.disabled,
        })
      )}
      type={component.props?.type || "text"}
      placeholder={component.props?.placeholder}
      value={value}
      readOnly={component.props?.readonly}
      disabled={component.props?.disabled}
      onChange={(e) => {
        const newValue = e.target.value;
        // Update local state immediately for responsive typing
        state.set(component.id, newValue);
        // If there's a change event handler, debounce it
        if (component.on_event?.change) {
          handleDebouncedEvent("change", { value: newValue }, 500);
        }
      }}
      style={component.props?.style}
    />
  );
};

Input.displayName = "Input";
