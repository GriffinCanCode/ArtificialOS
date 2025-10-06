/**
 * Textarea Component Renderer
 * Renders dynamic textarea components with high-performance state handling
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useInputState } from "../../hooks/useInputState";
import { textareaVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Textarea: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { value, onChange, onBlur } = useInputState(component, state, executor, {
    eventDebounce: 300, // Reduced from 500ms for better responsiveness
  });

  return (
    <textarea
      className={cn(
        textareaVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
          error: component.props?.error,
          disabled: component.props?.disabled,
          resize: component.props?.resize as any,
        })
      )}
      placeholder={component.props?.placeholder}
      value={value}
      disabled={component.props?.disabled}
      rows={component.props?.rows}
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
      style={component.props?.style}
    />
  );
};

Textarea.displayName = "Textarea";
