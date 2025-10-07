/**
 * Context Menu Component
 * Right-click context menu with smart positioning
 */

import React, { cloneElement, useCallback } from "react";
import { FloatingPortal, FloatingFocusManager } from "@floating-ui/react";
import { useContext } from "../hooks/useContext";
import type { ContextMenuProps } from "../core/types";
import "./ContextMenu.css";

// ============================================================================
// Component
// ============================================================================

export const ContextMenu: React.FC<ContextMenuProps> = React.memo(
  ({ items, children, onSelect }) => {
    const context = useContext();

    const handleContextMenu = useCallback(
      (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        context.open(e.clientX, e.clientY);
      },
      [context]
    );

    const handleSelect = (value: string) => {
      onSelect?.(value);
      context.close();
    };

    // Get the existing props from children and merge with onContextMenu
    const childProps = children.props || {};
    const mergedProps = {
      ...childProps,
      onContextMenu: (e: React.MouseEvent) => {
        handleContextMenu(e);
        // Also call original if it exists
        childProps.onContextMenu?.(e);
      },
    };

    return (
      <>
        {cloneElement(children, mergedProps)}
        {context.isOpen && (
          <FloatingPortal>
            <FloatingFocusManager context={context.context} modal={false}>
              <div
                ref={context.refs.setFloating}
                style={context.floatingStyles}
                className="context-menu"
                {...context.getFloatingProps()}
              >
                {items.map((item, index) => {
                  if (item.divider) {
                    return <div key={`divider-${index}`} className="context-menu-divider" />;
                  }

                  return (
                    <button
                      key={item.value}
                      type="button"
                      className={`context-menu-item ${item.disabled ? "disabled" : ""}`}
                      disabled={item.disabled}
                      onClick={() => handleSelect(item.value)}
                      {...context.getItemProps(index)}
                    >
                      {item.icon && <span className="context-menu-item-icon">{item.icon}</span>}
                      <span className="context-menu-item-label">{item.label}</span>
                    </button>
                  );
                })}
              </div>
            </FloatingFocusManager>
          </FloatingPortal>
        )}
      </>
    );
  }
);

ContextMenu.displayName = "ContextMenu";
