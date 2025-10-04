/**
 * Window Manager Component
 * Renders and manages all open windows
 */

import React from "react";
import { useVisibleWindows } from "../../store/windowStore";
import { Window } from "./Window";
import { ComponentState } from "../dynamics/DynamicRenderer.state";
import { ToolExecutor } from "../dynamics/DynamicRenderer.executor";
import { ComponentRenderer } from "../dynamics/DynamicRenderer.renderer";
import "./WindowManager.css";

export const WindowManager: React.FC = () => {
  const windows = useVisibleWindows();

  // Create component state and executor instances for each window
  // In a real implementation, you might want to share these or manage them differently
  const componentStateRef = React.useRef(new Map<string, ComponentState>());
  const toolExecutorRef = React.useRef(new Map<string, ToolExecutor>());

  const getComponentState = (windowId: string) => {
    if (!componentStateRef.current.has(windowId)) {
      componentStateRef.current.set(windowId, new ComponentState());
    }
    return componentStateRef.current.get(windowId)!;
  };

  const getToolExecutor = (windowId: string, appId: string) => {
    if (!toolExecutorRef.current.has(windowId)) {
      const state = getComponentState(windowId);
      const executor = new ToolExecutor(state);
      executor.setAppId(appId);
      toolExecutorRef.current.set(windowId, executor);
    }
    return toolExecutorRef.current.get(windowId)!;
  };

  return (
    <div className="window-manager">
      {windows.map((window) => {
        const state = getComponentState(window.id);
        const executor = getToolExecutor(window.id, window.appId);

        return (
          <Window key={window.id} window={window}>
            <div className="windowed-app">
              <div className={`app-content app-layout-${window.uiSpec.layout || "vertical"}`}>
                {window.uiSpec.components.map((component, idx) => (
                  <ComponentRenderer
                    key={`${component.id}-${idx}`}
                    component={component}
                    state={state}
                    executor={executor}
                  />
                ))}
              </div>
            </div>
          </Window>
        );
      })}
    </div>
  );
};

