/**
 * Textarea Component Renderer
 * Renders dynamic textarea components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { textareaVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Textarea: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleDebouncedEvent } = useComponent(component, state, executor);

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
      value={localState ?? component.props?.value ?? ""}
      disabled={component.props?.disabled}
      rows={component.props?.rows}
      onChange={(e) => {
        const newValue = e.target.value;
        // Update local state immediately for responsive typing
        state.set(component.id, newValue);
        // If there's a change event handler, debounce it to prevent lag
        if (component.on_event?.change) {
          handleDebouncedEvent("change", { value: newValue }, 500);
        }
      }}
      style={component.props?.style}
    />
  );
};

Textarea.displayName = "Textarea";
