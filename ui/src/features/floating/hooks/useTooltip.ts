/**
 * Tooltip Hook
 * Advanced tooltip with smart positioning
 */

import { useState, useMemo } from "react";
import {
  useFloating,
  useInteractions,
  useHover,
  useFocus,
  useRole,
  useDismiss,
  FloatingPortal,
} from "@floating-ui/react";
import { createDefaultMiddleware, getDelay, generateId } from "../core/utils";
import type { UseTooltipReturn, PositionConfig, InteractionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UseTooltipConfig {
  position?: PositionConfig;
  interaction?: InteractionConfig;
  initialOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating tooltips with smart positioning
 */
export function useTooltip({
  position,
  interaction,
  initialOpen = false,
  onOpenChange,
}: UseTooltipConfig = {}): UseTooltipReturn {
  const [isOpen, setIsOpen] = useState(initialOpen);

  // DEBUG
  console.log('[useTooltip] Hook initialized', { isOpen, delay: interaction?.delay });

  const delay = useMemo(
    () => getDelay(interaction?.delay ?? 300),
    [interaction?.delay]
  );

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      console.log('[useTooltip] Open state changed:', open);
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

  console.log('[useTooltip] useHover config:', { delay, context: data.context });

  const hover = useHover(data.context, {
    delay,
    move: false,
    handleClose: null,
  });

  console.log('[useTooltip] hover interactions:', hover);

  const focus = useFocus(data.context);

  const role = useRole(data.context, { role: "tooltip" });

  const dismiss = useDismiss(data.context, {
    escapeKey: interaction?.closeOnEscape ?? true,
  });

  const interactions = useInteractions([hover, focus, role, dismiss]);

  const tooltipId = useMemo(() => generateId("tooltip"), []);

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
    getReferenceProps: () => ({
      ...interactions.getReferenceProps(),
      "aria-describedby": isOpen ? tooltipId : undefined,
    }),
    getFloatingProps: () => ({
      ...interactions.getFloatingProps(),
      id: tooltipId,
    }),
  };
}
