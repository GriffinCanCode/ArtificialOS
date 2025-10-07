/**
 * Floating UI Utilities
 * Helper functions for positioning and floating elements
 */

import {
  offset,
  flip,
  shift,
  arrow,
  autoUpdate,
  size,
  hide,
  type Middleware,
} from "@floating-ui/react";
import type { PositionConfig } from "./types";

// ============================================================================
// Middleware Builders
// ============================================================================

/**
 * Create default middleware for floating elements
 */
export function createDefaultMiddleware(config?: PositionConfig): Middleware[] {
  const middleware: Middleware[] = [];

  // Add offset
  if (config?.offset !== undefined) {
    middleware.push(offset(config.offset));
  } else {
    middleware.push(offset(8));
  }

  // Add flip to prevent overflow
  middleware.push(
    flip({
      fallbackAxisSideDirection: "start",
      padding: 8,
    })
  );

  // Add shift to keep in viewport
  middleware.push(
    shift({
      padding: 8,
    })
  );

  return middleware;
}

/**
 * Create middleware with arrow
 */
export function createArrowMiddleware(
  arrowRef: React.RefObject<HTMLElement>,
  config?: PositionConfig
): Middleware[] {
  const middleware = createDefaultMiddleware(config);

  if (arrowRef.current) {
    middleware.push(
      arrow({
        element: arrowRef.current,
        padding: 8,
      })
    );
  }

  return middleware;
}

/**
 * Create middleware for dropdown/select
 */
export function createDropdownMiddleware(config?: PositionConfig): Middleware[] {
  const middleware = createDefaultMiddleware(config);

  // Add size middleware to match reference width
  middleware.push(
    size({
      apply({ rects, elements }) {
        Object.assign(elements.floating.style, {
          minWidth: `${rects.reference.width}px`,
        });
      },
      padding: 8,
    })
  );

  return middleware;
}

/**
 * Create middleware with hide detection
 */
export function createHideMiddleware(config?: PositionConfig): Middleware[] {
  const middleware = createDefaultMiddleware(config);

  middleware.push(hide());

  return middleware;
}

// ============================================================================
// Auto Update
// ============================================================================

/**
 * Setup auto-update for floating element
 */
export function setupAutoUpdate(
  reference: HTMLElement | null,
  floating: HTMLElement | null,
  update: () => void
): (() => void) | undefined {
  if (reference && floating) {
    return autoUpdate(reference, floating, update, {
      ancestorScroll: true,
      ancestorResize: true,
      elementResize: true,
      layoutShift: true,
    });
  }
}

// ============================================================================
// Positioning Helpers
// ============================================================================

/**
 * Get arrow position styles
 */
export function getArrowStyles(
  middlewareData: any,
  placement: string
): React.CSSProperties {
  const { x, y } = middlewareData.arrow || {};

  const staticSide = {
    top: "bottom",
    right: "left",
    bottom: "top",
    left: "right",
  }[placement.split("-")[0]];

  return {
    left: x != null ? `${x}px` : "",
    top: y != null ? `${y}px` : "",
    right: "",
    bottom: "",
    [staticSide as string]: "-4px",
  };
}

/**
 * Check if element is hidden by middleware
 */
export function isElementHidden(middlewareData: any): boolean {
  return middlewareData.hide?.referenceHidden || middlewareData.hide?.escaped;
}

// ============================================================================
// Interaction Helpers
// ============================================================================

/**
 * Get delay configuration
 */
export function getDelay(
  delay?: number | { open?: number; close?: number }
): { open: number; close: number } {
  if (typeof delay === "number") {
    return { open: delay, close: delay };
  }
  return {
    open: delay?.open ?? 0,
    close: delay?.close ?? 0,
  };
}

/**
 * Create debounced callback
 */
export function createDebounced<T extends (...args: any[]) => any>(
  callback: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout;

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => callback(...args), delay);
  };
}

// ============================================================================
// Accessibility Helpers
// ============================================================================

/**
 * Generate unique ID for accessibility
 */
let idCounter = 0;
export function generateId(prefix: string = "floating"): string {
  return `${prefix}-${++idCounter}`;
}

/**
 * Get role props for floating element
 */
export function getRoleProps(role?: string): Record<string, any> {
  switch (role) {
    case "tooltip":
      return { role: "tooltip" };
    case "dialog":
      return { role: "dialog", "aria-modal": true };
    case "menu":
      return { role: "menu" };
    case "listbox":
      return { role: "listbox" };
    default:
      return {};
  }
}
