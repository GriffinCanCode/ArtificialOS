/**
 * Select Hook
 * Advanced select/combobox with search and keyboard navigation
 */

import { useState, useMemo, useRef } from "react";
import {
  useFloating,
  useInteractions,
  useClick,
  useDismiss,
  useRole,
  useListNavigation,
  useTypeahead,
} from "@floating-ui/react";
import { createDropdownMiddleware, generateId } from "../core/utils";
import type { PositionConfig, InteractionConfig } from "../core/types";

// ============================================================================
// Hook Configuration
// ============================================================================

interface UseSelectConfig {
  position?: PositionConfig;
  interaction?: InteractionConfig;
  initialOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  searchable?: boolean;
}

interface UseSelectReturn {
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
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
  activeIndex: number | null;
  setActiveIndex: (index: number | null) => void;
  getReferenceProps: () => Record<string, any>;
  getFloatingProps: () => Record<string, any>;
  getItemProps: (index: number) => Record<string, any>;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for creating select/combobox with search
 */
export function useSelect({
  position,
  interaction,
  initialOpen = false,
  onOpenChange,
  searchable = false,
}: UseSelectConfig = {}): UseSelectReturn {
  const [isOpen, setIsOpen] = useState(initialOpen);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const listRef = useRef<(HTMLElement | null)[]>([]);
  const listContentRef = useRef<string[]>([]);

  const data = useFloating({
    open: isOpen,
    onOpenChange: (open) => {
      setIsOpen(open);
      onOpenChange?.(open);
      if (!open) {
        setActiveIndex(null);
        setSearchQuery("");
      }
    },
    placement: position?.placement ?? "bottom-start",
    strategy: position?.strategy ?? "absolute",
    middleware: position?.middleware ?? createDropdownMiddleware(position),
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

  const role = useRole(data.context, { role: "listbox" });

  const listNavigation = useListNavigation(data.context, {
    listRef,
    activeIndex,
    onNavigate: setActiveIndex,
    loop: true,
  });

  const typeahead = searchable
    ? useTypeahead(data.context, {
        listRef: listContentRef,
        activeIndex,
        onMatch: setActiveIndex,
      })
    : null;

  const interactions = useInteractions(
    typeahead
      ? [click, dismiss, role, listNavigation, typeahead]
      : [click, dismiss, role, listNavigation]
  );

  const selectId = useMemo(() => generateId("select"), []);

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
    activeIndex,
    setActiveIndex,
    searchQuery,
    setSearchQuery,
    getReferenceProps: () => ({
      ...interactions.getReferenceProps(),
      "aria-expanded": isOpen,
      "aria-controls": isOpen ? selectId : undefined,
    }),
    getFloatingProps: () => ({
      ...interactions.getFloatingProps(),
      id: selectId,
    }),
    getItemProps: (index: number) => ({
      ...interactions.getItemProps({
        onClick() {
          setSelectedIndex(index);
          setIsOpen(false);
        },
      }),
      ref: (node: HTMLElement | null) => {
        listRef.current[index] = node;
        if (node) {
          listContentRef.current[index] = node.textContent || "";
        }
      },
      tabIndex: activeIndex === index ? 0 : -1,
      role: "option",
      "aria-selected": selectedIndex === index,
    }),
  };
}
