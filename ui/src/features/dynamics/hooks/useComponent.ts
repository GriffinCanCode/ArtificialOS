/**
 * useComponent Hook
 * Shared logic for component state management and event handling
 * Extracted from monolithic renderer for reusability
 *
 * PERFORMANCE NOTE: Uses useSyncExternalStore via useSyncState for efficient subscriptions
 */

import { useEffect, useCallback, useRef } from "react";
import type { BlueprintComponent } from "../../../core/store/appStore";
import type { ComponentState } from "../state/state";
import type { ToolExecutor } from "../execution/executor";
import type { UseComponentReturn } from "../core/types";
import { useSyncState } from "./useSyncState";

// ============================================================================
// Main Hook
// ============================================================================

/**
 * Provides state management and event handling for dynamic components
 *
 * Uses React 18's useSyncExternalStore pattern for optimal performance:
 * - Zero unnecessary re-renders (only updates when subscribed value changes)
 * - Eliminates forceUpdate anti-pattern
 * - Concurrent mode safe
 *
 * @param component - Blueprint component definition
 * @param state - Component state manager
 * @param executor - Tool executor for backend integration
 * @returns Local state and event handlers
 *
 * @example
 * ```tsx
 * const { localState, handleEvent, handleDebouncedEvent } = useComponent(
 *   component,
 *   state,
 *   executor
 * );
 * ```
 */
export function useComponent(
  component: BlueprintComponent,
  state: ComponentState,
  executor: ToolExecutor
): UseComponentReturn {
  // Use efficient external store subscription via useSyncExternalStore
  const localState = useSyncState(state, component.id, component.props?.value);

  const changeDebounceTimerRef = useRef<number | null>(null);

  // ============================================================================
  // Cleanup Debounce Timers
  // ============================================================================

  useEffect(() => {
    return () => {
      if (changeDebounceTimerRef.current !== null) {
        window.clearTimeout(changeDebounceTimerRef.current);
      }
    };
  }, []);

  // ============================================================================
  // Event Handler
  // ============================================================================

  const handleEvent = useCallback(
    async (eventName: string, eventData?: any): Promise<void> => {
      const toolId = component.on_event?.[eventName];
      if (!toolId) return;

      // Extract params from event and component
      const params = {
        ...component.props, // Include all component props (e.g., noteId, data attributes)
        ...eventData,
        componentId: component.id,
        id: component.id,
        key: component.id,
        target: component.id,
        // Pass button text as multiple param names for flexibility
        value: component.props?.text || component.props?.value,
        text: component.props?.text,
        digit: component.props?.text,
      };

      try {
        await executor.execute(toolId, params);
      } catch (error) {
        console.error("Tool execution failed:", error);
      }
    },
    [component, executor]
  );

  // ============================================================================
  // Debounced Event Handler
  // ============================================================================

  const handleDebouncedEvent = useCallback(
    (eventName: string, eventData?: any, debounceMs: number = 500): void => {
      // Clear any existing timer
      if (changeDebounceTimerRef.current !== null) {
        window.clearTimeout(changeDebounceTimerRef.current);
      }

      // Set new timer
      changeDebounceTimerRef.current = window.setTimeout(() => {
        handleEvent(eventName, eventData);
        changeDebounceTimerRef.current = null;
      }, debounceMs);
    },
    [handleEvent]
  );

  return {
    localState,
    handleEvent,
    handleDebouncedEvent,
  };
}
