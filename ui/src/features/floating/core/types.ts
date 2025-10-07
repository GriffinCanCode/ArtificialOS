/**
 * Floating UI Core Types
 * Type definitions for positioning and floating elements
 */

import type { Placement, Strategy, Middleware } from "@floating-ui/react";

// ============================================================================
// Base Types
// ============================================================================

/**
 * Positioning configuration
 */
export interface PositionConfig {
  placement?: Placement;
  strategy?: Strategy;
  middleware?: Middleware[];
  offset?: number;
}

/**
 * Interaction configuration
 */
export interface InteractionConfig {
  trigger?: "hover" | "click" | "focus" | "manual";
  delay?: number | { open?: number; close?: number };
  closeOnClickOutside?: boolean;
  closeOnEscape?: boolean;
  closeOnScroll?: boolean;
}

/**
 * Accessibility configuration
 */
export interface AccessibilityConfig {
  role?: "tooltip" | "dialog" | "menu" | "listbox";
  describedBy?: boolean;
  labelledBy?: boolean;
}

/**
 * Animation configuration
 */
export interface AnimationConfig {
  duration?: number;
  easing?: string;
  scale?: boolean;
  fade?: boolean;
}

// ============================================================================
// Component Types
// ============================================================================

/**
 * Base floating element props
 */
export interface FloatingBaseProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  position?: PositionConfig;
  interaction?: InteractionConfig;
  accessibility?: AccessibilityConfig;
  animation?: AnimationConfig;
}

/**
 * Tooltip props
 */
export interface TooltipProps extends FloatingBaseProps {
  content: React.ReactNode;
  children: React.ReactElement;
  delay?: number;
}

/**
 * Popover props
 */
export interface PopoverProps extends FloatingBaseProps {
  content: React.ReactNode;
  children: React.ReactElement;
  arrow?: boolean;
  modal?: boolean;
}

/**
 * Dropdown props
 */
export interface DropdownProps extends FloatingBaseProps {
  items: DropdownItem[];
  children: React.ReactElement;
  onSelect?: (value: string) => void;
}

/**
 * Dropdown item
 */
export interface DropdownItem {
  value: string;
  label: React.ReactNode;
  icon?: React.ReactNode;
  disabled?: boolean;
  divider?: boolean;
}

/**
 * Context menu props
 */
export interface ContextMenuProps {
  items: DropdownItem[];
  children: React.ReactElement;
  onSelect?: (value: string) => void;
}

/**
 * Select props
 */
export interface SelectProps {
  options: SelectOption[];
  value?: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  searchable?: boolean;
}

/**
 * Select option
 */
export interface SelectOption {
  value: string;
  label: React.ReactNode;
  disabled?: boolean;
  group?: string;
}

/**
 * Hover card props
 */
export interface HoverCardProps extends FloatingBaseProps {
  content: React.ReactNode;
  children: React.ReactElement;
  openDelay?: number;
  closeDelay?: number;
}

// ============================================================================
// Hook Return Types
// ============================================================================

/**
 * Base floating hook return
 */
export interface UseFloatingReturn {
  refs: {
    reference: React.RefObject<HTMLElement>;
    floating: React.RefObject<HTMLElement>;
    setReference: (node: HTMLElement | null) => void;
    setFloating: (node: HTMLElement | null) => void;
  };
  floatingStyles: React.CSSProperties;
  isOpen: boolean;
  open: () => void;
  close: () => void;
  toggle: () => void;
}

/**
 * Tooltip hook return
 */
export interface UseTooltipReturn extends UseFloatingReturn {
  getReferenceProps: (userProps?: Record<string, any>) => Record<string, any>;
  getFloatingProps: (userProps?: Record<string, any>) => Record<string, any>;
}

/**
 * Popover hook return
 */
export interface UsePopoverReturn extends UseFloatingReturn {
  getReferenceProps: () => Record<string, any>;
  getFloatingProps: () => Record<string, any>;
  arrowRef: React.RefObject<HTMLElement>;
}

/**
 * Dropdown hook return
 */
export interface UseDropdownReturn extends UseFloatingReturn {
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
  getReferenceProps: () => Record<string, any>;
  getFloatingProps: () => Record<string, any>;
  getItemProps: (index: number) => Record<string, any>;
}
