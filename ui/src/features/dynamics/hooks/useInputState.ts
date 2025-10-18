/**
 * useInputState Hook
 * High-performance input handling with RAF batching and smart debouncing
 * Provides instant visual feedback while batching backend updates
 */

import { useRef, useCallback, useEffect } from "react";
import type { ComponentState } from "../state/state";
import type { ToolExecutor } from "../execution/executor";
import type { BlueprintComponent } from "../../../core/store/appStore";
import { useSyncState } from "./useSyncState";

export interface InputStateOptions {
  /** Debounce time for backend event calls (ms). Default: 300 */
  eventDebounce?: number;
}

export interface InputStateReturn {
  /** Current input value */
  value: string;
  /** Handle input change - call this in onChange */
  onChange: (newValue: string) => void;
  /** Handle blur event - flushes pending updates */
  onBlur: () => void;
}

/**
 * High-performance input state management hook
 *
 * Key optimizations:
 * - Uses useSyncExternalStore for efficient subscriptions (no forceUpdate)
 * - Synchronous state updates for controlled input (required for space bar, etc.)
 * - Separates visual updates (instant) from backend calls (debounced)
 * - Smart debouncing only for backend events, not typing
 *
 * @param component - Blueprint component definition
 * @param state - Component state manager
 * @param executor - Tool executor for backend events
 * @param options - Configuration options
 *
 * @example
 * ```tsx
 * const { value, onChange } = useInputState(component, state, executor);
 * <input value={value} onChange={(e) => onChange(e.target.value)} />
 * ```
 */
export function useInputState(
  component: BlueprintComponent,
  state: ComponentState,
  executor: ToolExecutor,
  options: InputStateOptions = {}
): InputStateReturn {
  const { eventDebounce = 300 } = options;

  // Use efficient external store subscription
  const syncedValue = useSyncState(state, component.id, component.props?.value ?? "");

  // Debounce ref for backend events only
  const eventTimerRef = useRef<number | null>(null);

  /**
   * Execute backend event handler (debounced)
   */
  const executeEvent = useCallback(
    async (value: string) => {
      const eventHandler = component.on_event?.change;
      if (!eventHandler) return;

      const params = {
        value,
        componentId: component.id,
        id: component.id,
        key: component.id,
      };

      try {
        await executor.execute(eventHandler, params);
      } catch (error) {
        // Error handling moved to executor error boundary
      }
    },
    [component, executor]
  );

  /**
   * Handle input change - synchronous for controlled inputs
   */
  const onChange = useCallback(
    (newValue: string) => {
      // CRITICAL: Update state synchronously for controlled inputs
      // This is required for React to accept all keystrokes (especially space)
      state.set(component.id, newValue);

      // Debounce backend event call (not visual update!)
      if (component.on_event?.change) {
        if (eventTimerRef.current !== null) {
          clearTimeout(eventTimerRef.current);
        }

        eventTimerRef.current = window.setTimeout(() => {
          executeEvent(newValue);
          eventTimerRef.current = null;
        }, eventDebounce);
      }
    },
    [component.id, state, eventDebounce, executeEvent]
  );

  /**
   * Handle blur - execute any pending backend events immediately
   */
  const onBlur = useCallback(() => {
    // Execute pending event immediately on blur
    if (eventTimerRef.current !== null) {
      clearTimeout(eventTimerRef.current);
      eventTimerRef.current = null;

      // Execute with current synced value
      executeEvent(state.get(component.id, ""));
    }
  }, [executeEvent, state, component.id]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (eventTimerRef.current !== null) {
        clearTimeout(eventTimerRef.current);
      }
    };
  }, []);

  return {
    value: syncedValue,
    onChange,
    onBlur,
  };
}
