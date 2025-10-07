/**
 * Dropdown Hook
 * Advanced dropdown with keyboard navigation
 */

import { useState, useMemo, useRef } from "react";
import {
  useFloating,
  useInteractions,
  useClick,
  useDismiss,
  useRole,
  useListNavigation,
} from "@floating-ui/react";
import { createDropdownMiddleware, generateId } from "../core/utils";
import type { UseDropdownReturn, PositionConfig, InteractionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UseDropdownConfig {
  position?: PositionConfig;
  interaction?: InteractionConfig;
  initialOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  itemCount?: number;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating dropdowns with keyboard navigation
 */
export function useDropdown({
  position,
  interaction,
  initialOpen = false,
  onOpenChange,
}: UseDropdownConfig = {}): UseDropdownReturn {
  const [isOpen, setIsOpen] = useState(initialOpen);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);

  const listRef = useRef<(HTMLElement | null)[]>([]);

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      setIsOpen(open);
      onOpenChange?.(open);
      if (!open) {
        setActiveIndex(null);
      }
    },
    placement: position?.placement ?? "bottom-start",
    strategy: position?.strategy ?? "absolute",
    middleware: position?.middleware ?? createDropdownMiddleware(position),
    whileElementsMounted: (_reference, _floating, update) => {
      const cleanup = () => update();
      window.addEventListener("scroll", cleanup, true);
      window.addEventListener("resize", cleanup);
      return () => {
        window.removeEventListener("scroll", cleanup, true);
        window.removeEventListener("resize", cleanup);
      };
    },
  });

  const click = useClick(data.context, {
    enabled: interaction?.trigger !== "manual",
  });

  const dismiss = useDismiss(data.context, {
    escapeKey: interaction?.closeOnEscape ?? true,
    outsidePress: interaction?.closeOnClickOutside ?? true,
  });

  const role = useRole(data.context, { role: "menu" });

  const listNavigation = useListNavigation(data.context, {
    listRef,
    activeIndex,
    onNavigate: setActiveIndex,
    loop: true,
  });

  const interactions = useInteractions([click, dismiss, role, listNavigation]);

  const dropdownId = useMemo(() => generateId("dropdown"), []);

  return {
    refs: {
      reference: data.refs.reference as any,
      floating: data.refs.floating as any,
      setReference: data.refs.setReference,
      setFloating: data.refs.setFloating,
    },
    floatingStyles: data.floatingStyles,
    isOpen,
    open: () => setIsOpen(true),
    close: () => setIsOpen(false),
    toggle: () => setIsOpen(!isOpen),
    selectedIndex,
    setSelectedIndex,
    getReferenceProps: () => ({
      ...interactions.getReferenceProps(),
      "aria-expanded": isOpen,
      "aria-controls": isOpen ? dropdownId : undefined,
    }),
    getFloatingProps: () => ({
      ...interactions.getFloatingProps(),
      id: dropdownId,
    }),
    getItemProps: (index: number) => ({
      ...interactions.getItemProps({
        onClick() {
          setSelectedIndex(index);
        },
      }),
      ref: (node: HTMLElement | null) => {
        listRef.current[index] = node;
      },
      tabIndex: activeIndex === index ? 0 : -1,
      role: "menuitem",
    }),
  };
}
