/**
 * Input Component Renderer
 * Renders dynamic input components with high-performance state handling
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useInputState } from "../../hooks/useInputState";
import { inputVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Input: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { value, onChange, onBlur } = useInputState(component, state, executor, {
    eventDebounce: 300, // Reduced from 500ms for better responsiveness
  });

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
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
      style={component.props?.style}
    />
  );
};

Input.displayName = "Input";
