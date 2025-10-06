/**
 * Dynamics Module - Main Exports
 * Organized exports for the dynamics system
 */

// Core
export { default as DynamicRenderer } from "./core/DynamicRenderer";
export * from "./core/types";
export * from "./core/constants";
export { registry } from "./core/registry";

// State Management
export { ComponentState } from "./state/state";
export type {
  StateChangeEvent,
  StateMiddleware,
  ComputedValue,
  SubscriptionOptions,
} from "./state/state";

// Execution
export { ToolExecutor } from "./execution/executor";

// Hooks
export { useComponent, useRegistry, useSyncState, useInputState } from "./hooks";
export type { UseComponentReturn, InputStateOptions, InputStateReturn } from "./hooks";

// Rendering (includes auto-registration)
export { ComponentRenderer, BuilderView, VirtualizedList } from "./rendering";

// Components (for direct usage or custom renderers)
export * from "./components";

// Validation
export * from "./schemas";
export { validateComponentProps, safeParseProps, formatValidationErrors } from "./core/validation";
export type { ValidationResult } from "./core/validation";
