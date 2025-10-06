/**
 * useRegistry Hook
 * Access to component registry for rendering
 */

import { useMemo } from "react";
import { registry } from "../core/registry";
import type { ComponentRenderer } from "../core/types";

// ============================================================================
// Registry Hook
// ============================================================================

/**
 * Hook to access the component registry
 * Provides stable reference to registry operations
 *
 * @example
 * ```tsx
 * const { getRenderer, hasRenderer, getTypes } = useRegistry();
 * const renderer = getRenderer("button");
 * ```
 */
export function useRegistry() {
  return useMemo(
    () => ({
      getRenderer: (type: string): ComponentRenderer | undefined => registry.get(type),
      hasRenderer: (type: string): boolean => registry.has(type),
      getTypes: (): string[] => registry.getTypes(),
      getByCategory: (category: ComponentRenderer["category"]) => registry.getByCategory(category),
      getStats: () => registry.getStats(),
    }),
    []
  );
}
