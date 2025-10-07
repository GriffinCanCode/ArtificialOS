/**
 * Hover Card Hook
 * Rich hover card with delayed display
 */

import { useState, useMemo } from "react";
import {
  useFloating,
  useInteractions,
  useHover,
  useFocus,
  useDismiss,
  useRole,
} from "@floating-ui/react";
import { createDefaultMiddleware, getDelay, generateId } from "../core/utils";
import type { PositionConfig, InteractionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UseHoverConfig {
  position?: PositionConfig;
  interaction?: InteractionConfig;
  initialOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  openDelay?: number;
  closeDelay?: number;
}

interface UseHoverReturn {
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
  getReferenceProps: () => Record<string, any>;
  getFloatingProps: () => Record<string, any>;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating hover cards
 */
export function useHoverCard({
  position,
  interaction,
  initialOpen = false,
  onOpenChange,
  openDelay = 700,
  closeDelay = 300,
}: UseHoverConfig = {}): UseHoverReturn {
  const [isOpen, setIsOpen] = useState(initialOpen);

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      setIsOpen(open);
      onOpenChange?.(open);
    },
    placement: position?.placement ?? "top",
    strategy: position?.strategy ?? "absolute",
    middleware: position?.middleware ?? createDefaultMiddleware(position),
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

  const hover = useHover(data.context, {
    delay: {
      open: openDelay,
      close: closeDelay,
    },
    move: false,
    restMs: 100,
  });

  const focus = useFocus(data.context);

  const dismiss = useDismiss(data.context, {
    escapeKey: interaction?.closeOnEscape ?? true,
  });

  const role = useRole(data.context, { role: "dialog" });

  const interactions = useInteractions([hover, focus, dismiss, role]);

  const hoverCardId = useMemo(() => generateId("hovercard"), []);

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
    getReferenceProps: (userProps?: any) => ({
      ...interactions.getReferenceProps(userProps),
      "aria-describedby": isOpen ? hoverCardId : undefined,
    }),
    getFloatingProps: (userProps?: any) => ({
      ...interactions.getFloatingProps(userProps),
      id: hoverCardId,
    }),
  };
}
