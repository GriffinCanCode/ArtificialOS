/**
 * Context Menu Hook
 * Right-click context menu with positioning
 */

import { useState, useMemo, useRef, useCallback } from "react";
import {
  useFloating,
  useInteractions,
  useDismiss,
  useRole,
  useListNavigation,
  FloatingFocusManager,
} from "@floating-ui/react";
import { createDefaultMiddleware, generateId } from "../core/utils";
import type { PositionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UseContextConfig {
  position?: PositionConfig;
  onOpenChange?: (open: boolean) => void;
}

interface UseContextReturn {
  refs: {
    reference: React.RefObject<HTMLElement>;
    floating: React.RefObject<HTMLElement>;
    setReference: (node: HTMLElement | null) => void;
    setFloating: (node: HTMLElement | null) => void;
  };
  floatingStyles: React.CSSProperties;
  isOpen: boolean;
  open: (x: number, y: number) => void;
  close: () => void;
  getReferenceProps: () => Record<string, any>;
  getFloatingProps: () => Record<string, any>;
  getItemProps: (index: number) => Record<string, any>;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating context menus
 */
export function useContext({
  position,
  onOpenChange,
}: UseContextConfig = {}): UseContextReturn {
  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const [position2, setPosition] = useState({ x: 0, y: 0 });

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
    placement: "bottom-start",
    strategy: "fixed",
    middleware: position?.middleware ?? createDefaultMiddleware(position),
  });

  const dismiss = useDismiss(data.context, {
    escapeKey: true,
    outsidePress: true,
  });

  const role = useRole(data.context, { role: "menu" });

  const listNavigation = useListNavigation(data.context, {
    listRef,
    activeIndex,
    onNavigate: setActiveIndex,
    loop: true,
  });

  const interactions = useInteractions([dismiss, role, listNavigation]);

  const contextId = useMemo(() => generateId("context"), []);

  const open = useCallback(
    (x: number, y: number) => {
      setPosition({ x, y });
      setIsOpen(true);
    },
    []
  );

  return {
    refs: {
      reference: data.refs.reference as any,
      floating: data.refs.floating as any,
      setReference: data.refs.setReference,
      setFloating: data.refs.setFloating,
    },
    floatingStyles: {
      ...data.floatingStyles,
      position: "fixed",
      left: position2.x,
      top: position2.y,
    },
    isOpen,
    open,
    close: () => setIsOpen(false),
    getReferenceProps: () => interactions.getReferenceProps(),
    getFloatingProps: () => ({
      ...interactions.getFloatingProps(),
      id: contextId,
    }),
    getItemProps: (index: number) => ({
      ...interactions.getItemProps(),
      ref: (node: HTMLElement | null) => {
        listRef.current[index] = node;
      },
      tabIndex: activeIndex === index ? 0 : -1,
      role: "menuitem",
    }),
  };
}
