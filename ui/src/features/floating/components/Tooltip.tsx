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

    console.log('[Tooltip] Render', { content, isOpen: tooltip.isOpen, hasChildren: !!children });

    if (!content) {
      return children;
    }

    const referenceProps = tooltip.getReferenceProps({
      ref: tooltip.refs.setReference,
      ...children.props,
    });

    console.log('[Tooltip] Reference props:', Object.keys(referenceProps));

    // Wrap handlers with logging
    const wrappedProps = {
      ...referenceProps,
      onPointerEnter: (e: any) => {
        console.log('[Tooltip] onPointerEnter fired!', content);
        referenceProps.onPointerEnter?.(e);
      },
      onMouseMove: (e: any) => {
        console.log('[Tooltip] onMouseMove fired!', content);
        referenceProps.onMouseMove?.(e);
      },
    };

    return (
      <>
        {cloneElement(children, wrappedProps)}
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
