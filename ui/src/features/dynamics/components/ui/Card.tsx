/**
 * Card Component Renderer
 * Renders dynamic card components with header/body/footer
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { ComponentRenderer } from "../../rendering/renderer";

export const Card: React.FC<BaseComponentProps> = ({ component, state, executor }) => {

  return (
    <div className="dynamic-card" style={component.props?.style}>
      {component.props?.title && (
        <div className="card-header">
          <h4>{component.props.title}</h4>
        </div>
      )}
      <div className="card-body">
        {component.children?.map((child) => (
          <ComponentRenderer key={child.id} component={child} state={state} executor={executor} />
        ))}
      </div>
      {component.props?.footer && <div className="card-footer">{component.props.footer}</div>}
    </div>
  );
};

Card.displayName = "Card";
