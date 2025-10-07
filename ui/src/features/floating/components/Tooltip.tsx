/**
 * Tooltip Component
 * Smart positioning tooltip with accessibility
 */

import React, { cloneElement } from "react";
import { FloatingPortal } from "@floating-ui/react";
import { useTooltip } from "../hooks/useTooltip";
import type { TooltipProps } from "../core/types";
import "./Tooltip.css";

// ============================================================================
// Component
// ============================================================================

export const Tooltip: React.FC<TooltipProps> = React.memo(
  ({ content, children, delay = 300, position, interaction, open, onOpenChange }) => {
    const tooltip = useTooltip({
      position,
      interaction: { ...interaction, delay },
      initialOpen: open,
      onOpenChange,
    });

    if (!content) {
      return children;
    }

    return (
      <>
        {cloneElement(
          children,
          tooltip.getReferenceProps({
            ref: tooltip.refs.setReference,
            ...children.props,
          })
        )}
        {tooltip.isOpen && (
          <FloatingPortal>
            <div
              ref={tooltip.refs.setFloating}
              style={tooltip.floatingStyles}
              className="tooltip"
              {...tooltip.getFloatingProps()}
            >
              {content}
            </div>
          </FloatingPortal>
        )}
      </>
    );
  }
);

Tooltip.displayName = "Tooltip";
