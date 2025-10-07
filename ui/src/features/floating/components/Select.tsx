/**
 * Select Component
 * Smart select/combobox with search and keyboard navigation
 */

import React, { useMemo } from "react";
import { FloatingPortal, FloatingFocusManager } from "@floating-ui/react";
import { ChevronDown, Check } from "lucide-react";
import { useSelect } from "../hooks/useSelect";
import type { SelectProps } from "../core/types";
import "./Select.css";

// ============================================================================
// Component
// ============================================================================

export const Select: React.FC<SelectProps> = React.memo(
  ({ options, value, onChange, placeholder = "Select...", disabled = false, searchable = false }) => {
    const select = useSelect({
      searchable,
    });

    const selectedOption = useMemo(
      () => options.find((opt) => opt.value === value),
      [options, value]
    );

    const filteredOptions = useMemo(() => {
      if (!searchable || !select.searchQuery) {
        return options;
      }
      const query = select.searchQuery.toLowerCase();
      return options.filter(
        (opt) =>
          typeof opt.label === "string" && opt.label.toLowerCase().includes(query)
      );
    }, [options, select.searchQuery, searchable]);

    const handleSelect = (optionValue: string, index: number) => {
      select.setSelectedIndex(index);
      onChange?.(optionValue);
      select.close();
    };

    return (
      <>
        <button
          ref={select.refs.setReference}
          type="button"
          className={`select-trigger ${select.isOpen ? "open" : ""} ${disabled ? "disabled" : ""}`}
          disabled={disabled}
          {...select.getReferenceProps()}
        >
          <span className="select-value">
            {selectedOption?.label || placeholder}
          </span>
          <ChevronDown
            size={16}
            className={`select-icon ${select.isOpen ? "rotate" : ""}`}
          />
        </button>
        {select.isOpen && (
          <FloatingPortal>
            <FloatingFocusManager context={select as any} modal={false}>
              <div
                ref={select.refs.setFloating}
                style={select.floatingStyles}
                className="select-dropdown"
                {...select.getFloatingProps()}
              >
                {searchable && (
                  <div className="select-search">
                    <input
                      type="text"
                      className="select-search-input"
                      placeholder="Search..."
                      value={select.searchQuery}
                      onChange={(e) => select.setSearchQuery(e.target.value)}
                      autoFocus
                    />
                  </div>
                )}
                <div className="select-options">
                  {filteredOptions.length === 0 ? (
                    <div className="select-empty">No options found</div>
                  ) : (
                    filteredOptions.map((option, index) => (
                      <button
                        key={option.value}
                        type="button"
                        className={`select-option ${
                          option.disabled ? "disabled" : ""
                        } ${option.value === value ? "selected" : ""}`}
                        disabled={option.disabled}
                        onClick={() => handleSelect(option.value, index)}
                        {...select.getItemProps(index)}
                      >
                        <span className="select-option-label">{option.label}</span>
                        {option.value === value && (
                          <Check size={16} className="select-option-check" />
                        )}
                      </button>
                    ))
                  )}
                </div>
              </div>
            </FloatingFocusManager>
          </FloatingPortal>
        )}
      </>
    );
  }
);

Select.displayName = "Select";
