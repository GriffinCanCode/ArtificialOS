/**
 * Text Component Renderer
 * Renders dynamic text components with variants
 * Supports dynamic content updates via state
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useSyncState } from "../../hooks/useSyncState";
import { textVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Text: React.FC<BaseComponentProps> = ({ component, state }) => {
  const textVariant = component.props?.variant || "body";

  // Subscribe to state changes for dynamic content (falls back to props.content)
  const content = useSyncState(state, component.id, component.props?.content);

  // Map variant to HTML tag
  const Tag =
    textVariant === "h1" ? "h1" : textVariant === "h2" ? "h2" : textVariant === "h3" ? "h3" : "p";

  return (
    <Tag
      className={cn(
        textVariants({
          variant: textVariant as any,
          weight: component.props?.weight as any,
          color: component.props?.color as any,
          align: component.props?.align as any,
        })
      )}
      style={component.props?.style}
    >
      {content}
    </Tag>
  );
};

Text.displayName = "Text";
