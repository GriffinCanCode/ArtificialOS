/**
 * Tabs Component Renderer
 * Renders dynamic tabbed interface components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { tabVariants, cn } from "../../../../core/utils/animation/componentVariants";
import { ComponentRenderer } from "../../rendering/renderer";

export const Tabs: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);

  const activeTab = localState ?? component.props?.defaultTab ?? component.children?.[0]?.id;

  return (
    <div className="dynamic-tabs" style={component.props?.style}>
      <div
        className={cn(
          tabVariants({
            variant: component.props?.variant as any,
            size: component.props?.size as any,
          }),
          "tabs-header"
        )}
      >
        {component.children?.map((tab) => (
          <button
            key={tab.id}
            className={activeTab === tab.id ? "tab-active" : "tab-inactive"}
            onClick={() => {
              state.set(component.id, tab.id);
              handleEvent("tabChange", { tabId: tab.id });
            }}
          >
            {tab.props?.label || tab.id}
          </button>
        ))}
      </div>
      <div className="tabs-content">
        {component.children?.map((tab) =>
          activeTab === tab.id ? (
            <div key={tab.id} className="tab-panel">
              {/* Render the tab container itself, not its children */}
              <ComponentRenderer key={tab.id} component={tab} state={state} executor={executor} />
            </div>
          ) : null
        )}
      </div>
    </div>
  );
};

Tabs.displayName = "Tabs";
