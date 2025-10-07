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
  autoUpdate,
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

  const delay = useMemo(() => getDelay(interaction?.delay ?? 300), [interaction?.delay]);

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      setIsOpen(open);
      onOpenChange?.(open);
    },
    placement: position?.placement ?? "top",
    strategy: position?.strategy ?? "absolute",
    middleware: position?.middleware ?? createDefaultMiddleware(position),
    whileElementsMounted: autoUpdate,
    transform: false, // Use top/left instead of transform for smoother initial positioning
  });

  const hover = useHover(data.context, {
    delay,
    move: false,
  });

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
    getReferenceProps: (userProps?: any) => ({
      ...interactions.getReferenceProps(userProps),
      "aria-describedby": isOpen ? tooltipId : undefined,
    }),
    getFloatingProps: (userProps?: any) => ({
      ...interactions.getFloatingProps(userProps),
      id: tooltipId,
    }),
  };
}
