/**
 * Hover Card Component
 * Rich hover card with delayed display
 */

import React, { cloneElement } from "react";
import { FloatingPortal } from "@floating-ui/react";
import { useHoverCard } from "../hooks/useHover";
import type { HoverCardProps } from "../core/types";
import "./HoverCard.css";

// ============================================================================
// Component
// ============================================================================

export const HoverCard: React.FC<HoverCardProps> = React.memo(
  ({
    content,
    children,
    openDelay = 700,
    closeDelay = 300,
    position,
    interaction,
    open,
    onOpenChange,
  }) => {
    const hoverCard = useHoverCard({
      position,
      interaction,
      initialOpen: open,
      onOpenChange,
      openDelay,
      closeDelay,
    });

    if (!content) {
      return children;
    }

    return (
      <>
        {cloneElement(
          children,
          hoverCard.getReferenceProps({
            ref: hoverCard.refs.setReference,
            ...children.props,
          })
        )}
        {hoverCard.isOpen && (
          <FloatingPortal>
            <div
              ref={hoverCard.refs.setFloating}
              style={hoverCard.floatingStyles}
              className="hover-card"
              {...hoverCard.getFloatingProps()}
            >
              {content}
            </div>
          </FloatingPortal>
        )}
      </>
    );
  }
);

HoverCard.displayName = "HoverCard";
