/**
 * Component Renderer Factory
 * Lightweight factory that routes components to registered renderers
 */

import React from "react";
import type { BaseComponentProps } from "../core/types";
import { registry } from "../core/registry";
import { logger } from "../../../core/monitoring/core/logger";
import { safeParseProps } from "../core/validation";

// ============================================================================
// Component Renderer Factory
// ============================================================================

/**
 * Renders a dynamic component by delegating to registered renderer
 *
 * @example
 * ```tsx
 * <ComponentRenderer
 *   component={blueprintComponent}
 *   state={componentState}
 *   executor={toolExecutor}
 * />
 * ```
 */
export const ComponentRenderer: React.FC<BaseComponentProps> = React.memo(
  ({ component, state, executor }) => {
    // Get renderer from registry
    const rendererEntry = registry.get(component.type);

    // Handle unknown component types
    if (!rendererEntry) {
      if (process.env.NODE_ENV === "development") {
        logger.warn("Unknown component type", {
          component: "ComponentRenderer",
          type: component.type,
          id: component.id,
          availableTypes: registry.getTypes(),
        });
      }
      return null;
    }

    // Validate props if schema is present
    let validatedComponent = component;
    if (rendererEntry.schema) {
      const validatedProps = safeParseProps(component.props, rendererEntry.schema) as Record<
        string,
        any
      >;
      validatedComponent = { ...component, props: validatedProps };
    }

    // Delegate to registered renderer
    const Renderer = rendererEntry.render;
    return <Renderer component={validatedComponent} state={state} executor={executor} />;
  },
  (prevProps, nextProps) => {
    // Custom comparison for better memoization
    // Return true if props are equal (skip re-render)
    if (prevProps.component.id !== nextProps.component.id) return false;
    if (prevProps.component.type !== nextProps.component.type) return false;
    if (prevProps.component.children?.length !== nextProps.component.children?.length) return false;

    // Deep compare props - but only if they're different objects
    if (prevProps.component.props === nextProps.component.props) return true;

    // Compare props by key (more efficient than JSON.stringify for large objects)
    const prevKeys = Object.keys(prevProps.component.props || {});
    const nextKeys = Object.keys(nextProps.component.props || {});

    if (prevKeys.length !== nextKeys.length) return false;

    for (const key of prevKeys) {
      const prevVal = prevProps.component.props?.[key];
      const nextVal = nextProps.component.props?.[key];

      // For primitive values and simple objects
      if (typeof prevVal !== "object" || typeof nextVal !== "object") {
        if (prevVal !== nextVal) return false;
      } else {
        // For complex objects, fall back to JSON comparison (cached)
        if (JSON.stringify(prevVal) !== JSON.stringify(nextVal)) return false;
      }
    }

    return true;
  }
);

ComponentRenderer.displayName = "ComponentRenderer";
