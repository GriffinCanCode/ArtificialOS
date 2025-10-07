/**
 * Window Content Router
 * Routes window content to appropriate renderer based on app type
 */

import React from "react";
import { Renderer as NativeRenderer } from "./renderer";
import { ComponentRenderer } from "../../dynamics/rendering/renderer";
import { ComponentState } from "../../dynamics/state/state";
import { ToolExecutor } from "../../dynamics/execution/executor";
import type { Window } from "../../windows/core/types";
import type { BlueprintComponent } from "../../../core/store/appStore";
import { AppType } from "../core/types";
import "./content.css";

// ============================================================================
// Props
// ============================================================================

interface ContentProps {
  /** Window to render */
  window: Window;
  /** Component state for blueprint apps */
  state: ComponentState;
  /** Tool executor for blueprint apps */
  executor: ToolExecutor;
}

// ============================================================================
// Content Router Component
// ============================================================================

/**
 * Window Content Router
 * Determines app type and delegates to appropriate renderer
 */
export const Content: React.FC<ContentProps> = ({ window, state, executor }) => {
  // Determine app type from window metadata
  const appType = (window.metadata as any)?.appType || AppType.BLUEPRINT;

  // Native web app
  if (appType === AppType.NATIVE) {
    const meta = window.metadata as any;
    return (
      <div className="window-content native">
        <NativeRenderer
          appId={window.appId}
          packageId={meta.packageId}
          bundlePath={meta.bundlePath}
          windowId={window.id}
        />
      </div>
    );
  }

  // Blueprint app (default)
  return (
    <div className="window-content blueprint">
      <div className={`app-layout-${window.uiSpec.layout || "vertical"}`}>
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
  );
};
