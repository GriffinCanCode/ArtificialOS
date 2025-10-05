/**
 * useComponent Hook
 * Shared logic for component state management and event handling
 * Extracted from monolithic renderer for reusability
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { BlueprintComponent } from "../../../core/store/appStore";
import type { ComponentState, SubscriptionOptions } from "../state/state";
import type { ToolExecutor } from "../execution/executor";
import type { UseComponentReturn } from "../core/types";

// ============================================================================
// Main Hook
// ============================================================================

/**
 * Provides state management and event handling for dynamic components
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
  const [localState, setLocalState] = useState<any>(null);
  const [, forceUpdate] = useState({});
  const changeDebounceTimerRef = useRef<number | null>(null);

  // ============================================================================
  // State Subscription
  // ============================================================================

  useEffect(() => {
    if (!component.id) return;

    // Initialize local state from component state manager
    setLocalState(state.get(component.id, component.props?.value));

    // Configure subscription options based on component type
    const subscriptionOptions: SubscriptionOptions = {
      immediate: true,
    };

    // Add debouncing for text inputs to reduce re-renders
    if (component.type === "input" && component.props?.type === "text") {
      subscriptionOptions.debounce = 100;
    }

    // Subscribe to state changes
    const unsubscribe = state.subscribe(
      component.id,
      (newValue, oldValue) => {
        // Only update if value actually changed
        if (newValue !== oldValue) {
          setLocalState(newValue);
          forceUpdate({});
        }
      },
      subscriptionOptions
    );

    return unsubscribe;
  }, [component.id, component.type, component.props?.value, component.props?.type, state]);

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
