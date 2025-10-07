/**
 * Tooltip Component
 * Smart positioning tooltip with accessibility
 */

import React, { cloneElement, useMemo } from "react";
import { FloatingPortal } from "@floating-ui/react";
import { useTooltip } from "../hooks/useTooltip";
import type { TooltipProps } from "../core/types";
import "./Tooltip.css";

// ============================================================================
// Component
// ============================================================================

export const Tooltip: React.FC<TooltipProps> = ({
  content,
  children,
  delay = 300,
  position,
  interaction,
  open,
  onOpenChange,
}) => {
  // Memoize the interaction config to prevent hook re-initialization
  const interactionConfig = useMemo(
    () => ({ ...interaction, delay }),
    [interaction?.trigger, interaction?.closeOnEscape, delay]
  );

  const tooltip = useTooltip({
    position,
    interaction: interactionConfig,
    initialOpen: open,
    onOpenChange,
  });

  if (!content) {
    return children;
  }

  const referenceProps = tooltip.getReferenceProps();

  return (
    <>
      {cloneElement(children, {
        ...children.props,
        ...referenceProps,
        ref: tooltip.refs.setReference,
      })}
      {tooltip.isOpen && content && (
        <FloatingPortal>
          <div
            ref={tooltip.refs.setFloating}
            style={{
              ...tooltip.floatingStyles,
              // Hide until positioned (not at 0,0)
              visibility:
                tooltip.floatingStyles.left === 0 && tooltip.floatingStyles.top === 0
                  ? "hidden"
                  : "visible",
            }}
            className="tooltip"
            {...tooltip.getFloatingProps()}
          >
            {content}
          </div>
        </FloatingPortal>
      )}
    </>
  );
};

Tooltip.displayName = "Tooltip";
