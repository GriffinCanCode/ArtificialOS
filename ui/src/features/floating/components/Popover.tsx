/**
 * Popover Component
 * Smart positioning popover with arrow and interactions
 */

import React, { cloneElement } from "react";
import { FloatingPortal, FloatingFocusManager } from "@floating-ui/react";
import { usePopover } from "../hooks/usePopover";
import type { PopoverProps } from "../core/types";
import "./Popover.css";

// ============================================================================
// Component
// ============================================================================

export const Popover: React.FC<PopoverProps> = React.memo(
  ({
    content,
    children,
    arrow = true,
    modal = false,
    position,
    interaction,
    open,
    onOpenChange,
  }) => {
    const popover = usePopover({
      position,
      interaction,
      initialOpen: open,
      onOpenChange,
      modal,
      arrow,
    });

    const referenceProps = popover.getReferenceProps();

    return (
      <>
        {cloneElement(children, {
          ...children.props,
          ...referenceProps,
          ref: popover.refs.setReference,
        })}
        {popover.isOpen && (
          <FloatingPortal>
            <FloatingFocusManager context={popover as any} modal={modal}>
              <div
                ref={popover.refs.setFloating}
                style={popover.floatingStyles}
                className="popover"
                {...popover.getFloatingProps()}
              >
                {content}
                {arrow && <div ref={popover.arrowRef as any} className="popover-arrow" />}
              </div>
            </FloatingFocusManager>
          </FloatingPortal>
        )}
      </>
    );
  }
);

Popover.displayName = "Popover";
