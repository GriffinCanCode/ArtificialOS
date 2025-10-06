/**
 * Iframe Component Renderer
 * Renders dynamic iframe/embedded content components
 * Supports dynamic URL updates from browser.navigate tool
 */

import React, { useState, useEffect } from "react";
import type { BaseComponentProps } from "../../core/types";
import { iframeVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Iframe: React.FC<BaseComponentProps> = ({ component, state }) => {
  // Subscribe to dynamic URL changes from browser.navigate tool
  const urlKey = `${component.id}_url`;
  const [dynamicUrl, setDynamicUrl] = useState<string | undefined>(() => state.get(urlKey));

  useEffect(() => {
    // Subscribe to URL changes and force re-render
    const unsubscribe = state.subscribe(urlKey, (newUrl: string) => {
      console.log(`[Iframe] URL changed for ${component.id}:`, newUrl);
      setDynamicUrl(newUrl);
    });

    return unsubscribe;
  }, [urlKey, state, component.id]);

  const src = dynamicUrl || component.props?.src;

  return (
    <iframe
      className={cn(
        iframeVariants({
          bordered: component.props?.bordered,
          rounded: component.props?.rounded,
        })
      )}
      src={src}
      title={component.props?.title || "iframe"}
      width={component.props?.width || "100%"}
      height={component.props?.height || 400}
      style={component.props?.style}
      sandbox={component.props?.sandbox}
    />
  );
};

Iframe.displayName = "Iframe";
