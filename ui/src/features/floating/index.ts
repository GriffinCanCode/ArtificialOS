/**
 * Floating UI Module
 * Advanced positioning for tooltips, popovers, dropdowns, and more
 */

// Types
export type {
  PositionConfig,
  InteractionConfig,
  AccessibilityConfig,
  AnimationConfig,
  FloatingBaseProps,
  TooltipProps,
  PopoverProps,
  DropdownProps,
  DropdownItem,
  ContextMenuProps,
  SelectProps,
  SelectOption,
  HoverCardProps,
  UseFloatingReturn,
  UseTooltipReturn,
  UsePopoverReturn,
  UseDropdownReturn,
} from "./core/types";

// Utilities
export {
  createDefaultMiddleware,
  createArrowMiddleware,
  createDropdownMiddleware,
  createHideMiddleware,
  setupAutoUpdate,
  getArrowStyles,
  isElementHidden,
  getDelay,
  createDebounced,
  generateId,
  getRoleProps,
} from "./core/utils";

// Hooks
export { useTooltip } from "./hooks/useTooltip";
export { usePopover } from "./hooks/usePopover";
export { useDropdown } from "./hooks/useDropdown";
export { useSelect } from "./hooks/useSelect";
export { useContext } from "./hooks/useContext";
export { useHoverCard } from "./hooks/useHover";

// Components
export { Tooltip } from "./components/Tooltip";
export { Popover } from "./components/Popover";
export { Dropdown } from "./components/Dropdown";
export { ContextMenu } from "./components/ContextMenu";
export { Select } from "./components/Select";
export { HoverCard } from "./components/HoverCard";
