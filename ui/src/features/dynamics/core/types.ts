/**
 * Core Type Definitions
 * Shared types for dynamic component rendering system
 */

import type { ComponentState } from "../state/state";
import type { ToolExecutor } from "../execution/executor";
import type { BlueprintComponent } from "../../../core/store/appStore";
import type { z } from "zod";

// ============================================================================
// Component Renderer Types
// ============================================================================

/**
 * Base props passed to all component renderers
 */
export interface BaseComponentProps {
  component: BlueprintComponent;
  state: ComponentState;
  executor: ToolExecutor;
}

/**
 * Component renderer function type
 */
export type ComponentRendererFn = React.FC<BaseComponentProps>;

/**
 * Registry entry for a component renderer
 */
export interface ComponentRenderer {
  type: string;
  render: ComponentRendererFn;
  schema?: z.ZodSchema;
  category?: ComponentCategory;
}

/**
 * Component categories for organization
 */
export type ComponentCategory =
  | "primitive"
  | "layout"
  | "form"
  | "media"
  | "ui"
  | "special";

// ============================================================================
// Event Handler Types
// ============================================================================

/**
 * Event handler function
 */
export type EventHandler = (eventName: string, eventData?: any) => Promise<void>;

/**
 * Debounced event handler function
 */
export type DebouncedEventHandler = (
  eventName: string,
  eventData?: any,
  debounceMs?: number
) => void;

// ============================================================================
// Hook Return Types
// ============================================================================

/**
 * Return type for useComponent hook
 */
export interface UseComponentReturn {
  localState: any;
  handleEvent: EventHandler;
  handleDebouncedEvent: DebouncedEventHandler;
}
