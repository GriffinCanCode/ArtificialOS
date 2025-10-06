/**
 * Select Component Renderer
 * Renders dynamic select/dropdown components with efficient state handling
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useSyncState } from "../../hooks/useSyncState";
import { useComponent } from "../../hooks/useComponent";
import { selectVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Select: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const value = useSyncState(state, component.id, component.props?.value ?? "");
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <select
      className={cn(
        selectVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
          error: component.props?.error,
          disabled: component.props?.disabled,
        })
      )}
      value={value}
      disabled={component.props?.disabled}
      onChange={(e) => {
        const newValue = e.target.value;
        state.set(component.id, newValue);
        handleEvent("change", { value: newValue });
      }}
      style={component.props?.style}
    >
      {component.props?.options?.map((opt: any, idx: number) => (
        <option key={idx} value={opt.value || opt}>
          {opt.label || opt}
        </option>
      ))}
    </select>
  );
};

Select.displayName = "Select";
