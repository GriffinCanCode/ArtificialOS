/**
 * Modal Component Renderer
 * Renders dynamic modal/dialog components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { modalVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { ComponentRenderer } from "../../rendering/renderer";

export const Modal: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);

  const isOpen = localState ?? component.props?.open ?? false;

  if (!isOpen) return null;

  return (
    <div
      className="modal-overlay"
      onClick={() => {
        state.set(component.id, false);
        handleEvent("close");
      }}
    >
      <div
        className={cn(
          modalVariants({
            size: component.props?.size as any,
            centered: component.props?.centered,
          })
        )}
        onClick={(e) => e.stopPropagation()}
        style={component.props?.style}
      >
        {component.props?.title && (
          <div className="modal-header">
            <h3>{component.props.title}</h3>
            <button
              onClick={() => {
                state.set(component.id, false);
                handleEvent("close");
              }}
            >
              ×
            </button>
          </div>
        )}
        <div className="modal-body">
          {component.children?.map((child) => (
            <ComponentRenderer key={child.id} component={child} state={state} executor={executor} />
          ))}
        </div>
      </div>
    </div>
  );
};

Modal.displayName = "Modal";
