/**
 * AppShortcut Component Renderer
 * Renders dynamic app shortcut components
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";
import { AppShortcut as AppShortcutComponent } from "../../components/AppShortcut";

export const AppShortcut: React.FC<BaseComponentProps> = ({ component, state, executor }) => {
  const { handleEvent } = useComponent(component, state, executor);

  return (
    <AppShortcutComponent
      id={component.props?.app_id || component.id}
      name={component.props?.name || "App"}
      icon={component.props?.icon || "📦"}
      description={component.props?.description}
      category={component.props?.category}
      variant={component.props?.variant as "icon" | "card" | "list"}
      onClick={(appId: string) => handleEvent("click", { app_id: appId })}
    />
  );
};

AppShortcut.displayName = "AppShortcut";
