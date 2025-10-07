/**
 * Popover Hook
 * Advanced popover with interactions and arrow
 */

import { useState, useMemo, useRef } from "react";
import {
  useFloating,
  useInteractions,
  useClick,
  useDismiss,
  useRole,
  FloatingFocusManager,
} from "@floating-ui/react";
import { createArrowMiddleware, generateId } from "../core/utils";
import type { UsePopoverReturn, PositionConfig, InteractionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UsePopoverConfig {
  position?: PositionConfig;
  interaction?: InteractionConfig;
  initialOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  modal?: boolean;
  arrow?: boolean;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating popovers with smart positioning
 */
export function usePopover({
  position,
  interaction,
  initialOpen = false,
  onOpenChange,
  modal = false,
  arrow: showArrow = true,
}: UsePopoverConfig = {}): UsePopoverReturn {
  const [isOpen, setIsOpen] = useState(initialOpen);
  const arrowRef = useRef<HTMLElement>(null);

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      setIsOpen(open);
      onOpenChange?.(open);
    },
    placement: position?.placement ?? "bottom",
    strategy: position?.strategy ?? "absolute",
    middleware: showArrow
      ? createArrowMiddleware(arrowRef, position)
      : position?.middleware,
    whileElementsMounted: (reference, floating, update) => {
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

  const role = useRole(data.context, { role: "dialog" });

  const interactions = useInteractions([click, dismiss, role]);

  const popoverId = useMemo(() => generateId("popover"), []);

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
    arrowRef,
    getReferenceProps: () => ({
      ...interactions.getReferenceProps(),
      "aria-expanded": isOpen,
      "aria-controls": isOpen ? popoverId : undefined,
    }),
    getFloatingProps: () => ({
      ...interactions.getFloatingProps(),
      id: popoverId,
    }),
  };
}
