/**
 * Dropdown Component
 * Smart positioning dropdown with keyboard navigation
 */

import React, { cloneElement } from "react";
import { FloatingPortal, FloatingFocusManager } from "@floating-ui/react";
import { useDropdown } from "../hooks/useDropdown";
import type { DropdownProps } from "../core/types";
import "./Dropdown.css";

// ============================================================================
// Component
// ============================================================================

export const Dropdown: React.FC<DropdownProps> = React.memo(
  ({ items, children, onSelect, position, interaction, open, onOpenChange }) => {
    const dropdown = useDropdown({
      position,
      interaction,
      initialOpen: open,
      onOpenChange,
      itemCount: items.length,
    });

    const handleSelect = (value: string, index: number) => {
      dropdown.setSelectedIndex(index);
      onSelect?.(value);
      dropdown.close();
    };

    const referenceProps = dropdown.getReferenceProps();

    return (
      <>
        {cloneElement(children, {
          ...children.props,
          ...referenceProps,
          ref: dropdown.refs.setReference,
        })}
        {dropdown.isOpen && (
          <FloatingPortal>
            <FloatingFocusManager context={dropdown as any} modal={false}>
              <div
                ref={dropdown.refs.setFloating}
                style={dropdown.floatingStyles}
                className="dropdown"
                {...dropdown.getFloatingProps()}
              >
                {items.map((item, index) => {
                  if (item.divider) {
                    return <div key={`divider-${index}`} className="dropdown-divider" />;
                  }

                  return (
                    <button
                      key={item.value}
                      type="button"
                      className={`dropdown-item ${item.disabled ? "disabled" : ""} ${
                        dropdown.selectedIndex === index ? "selected" : ""
                      }`}
                      disabled={item.disabled}
                      onClick={() => handleSelect(item.value, index)}
                      {...dropdown.getItemProps(index)}
                    >
                      {item.icon && <span className="dropdown-item-icon">{item.icon}</span>}
                      <span className="dropdown-item-label">{item.label}</span>
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

Dropdown.displayName = "Dropdown";
