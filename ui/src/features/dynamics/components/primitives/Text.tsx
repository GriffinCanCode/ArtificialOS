/**
 * Text Component Renderer
 * Renders dynamic text components with variants
 */

import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { textVariants, cn } from "../../../../core/utils/animation/componentVariants";

export const Text: React.FC<BaseComponentProps> = ({ component }) => {
  const textVariant = component.props?.variant || "body";

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
      {component.props?.content}
    </Tag>
  );
};

Text.displayName = "Text";
