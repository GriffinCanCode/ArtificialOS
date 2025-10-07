/**
 * Window Manager Component
 * Renders and manages all open windows
 */

import React from "react";
import { useVisibleWindows } from "../../../features/windows";
import { Window } from "./Window";
import { ComponentState } from "../../../features/dynamics/state/state";
import { ToolExecutor } from "../../../features/dynamics/execution/executor";
import { ComponentRenderer } from "../../../features/dynamics/rendering/renderer";
import { BuilderView } from "../../../features/dynamics/rendering/builder";
import { Renderer as NativeRenderer, AppType } from "../../../features/native";
import type { BlueprintComponent } from "../../../core/store/appStore";
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

        // Check if this is a builder window
        const isBuilderWindow = window.appId.startsWith("builder-");

        // Check if this is a native app
        const appType = (window.metadata as any)?.appType;
        const isNativeApp = appType === AppType.NATIVE || appType === 'native_web';

        return (
          <Window key={window.id} window={window}>
            {isBuilderWindow ? (
              // Render BuilderView for builder windows (shows build progress)
              <BuilderView state={state} executor={executor} />
            ) : isNativeApp ? (
              // Render native app
              <div className="windowed-app">
                <NativeRenderer
                  appId={window.appId}
                  packageId={(window.metadata as any)?.packageId}
                  bundlePath={(window.metadata as any)?.bundlePath}
                  windowId={window.id}
                />
              </div>
            ) : (
              // Render normal blueprint app content (existing system)
              <div className="windowed-app">
                <div className={`app-content app-layout-${window.uiSpec.layout || "vertical"}`}>
                  {window.uiSpec.components.map((component: BlueprintComponent, idx: number) => (
                    <ComponentRenderer
                      key={`${component.id}-${idx}`}
                      component={component}
                      state={state}
                      executor={executor}
                    />
                  ))}
                </div>
              </div>
            )}
          </Window>
        );
      })}
    </div>
  );
};
