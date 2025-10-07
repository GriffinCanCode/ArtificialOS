/**
 * Window Manager Component
 * Renders and manages all open windows
 */

import React from "react";
import { useVisibleWindows } from "../../../features/windows";
import { Window } from "./Window";
import { ComponentRenderer } from "../../../features/dynamics/rendering/renderer";
import { BuilderView } from "../../../features/dynamics/rendering/builder";
import { useDynamicsInstance } from "../../../features/dynamics/store/instanceStore";
import { Renderer as NativeRenderer, AppType } from "../../../features/native";
import type { BlueprintComponent } from "../../../core/store/appStore";
import "./WindowManager.css";

export const WindowManager: React.FC = () => {
  const windows = useVisibleWindows();

  return (
    <div className="window-manager">
      {windows.map((window) => (
        <WindowContent key={window.id} window={window} />
      ))}
    </div>
  );
};

/**
 * Individual window content component
 * Manages its own dynamics instances via the store
 */
const WindowContent: React.FC<{ window: any }> = ({ window }) => {
  const { state, executor } = useDynamicsInstance(window.id, window.appId);

  // Cleanup instances when component unmounts (window closes)
  React.useEffect(() => {
    return () => {
      // Cleanup will be handled by the window manager when window is actually removed
      // This is just a safety cleanup
    };
  }, [window.id]);

  // Check if this is a builder window
  const isBuilderWindow = window.appId.startsWith("builder-");

  // Check if this is a native app
  const appType = (window.metadata as any)?.appType;
  const isNativeApp = appType === AppType.NATIVE || appType === "native_web";

  return (
    <Window window={window}>
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
};
