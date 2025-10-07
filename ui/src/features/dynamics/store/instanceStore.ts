/**
 * Dynamics Instance Store
 * Manages ComponentState and ToolExecutor instances per window
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { ComponentState } from "../state/state";
import { ToolExecutor } from "../execution/executor";

// ============================================================================
// Store Interface
// ============================================================================

interface InstanceCache {
  state: ComponentState;
  executor: ToolExecutor;
  appId: string;
}

interface Store {
  instances: Map<string, InstanceCache>;

  // Actions
  getOrCreate: (windowId: string, appId: string) => InstanceCache;
  remove: (windowId: string) => void;
  clear: () => void;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useInstanceStore = create<Store>()(
  devtools(
    (set, get) => ({
      instances: new Map(),

      getOrCreate: (windowId: string, appId: string) => {
        const state = get();

        // Return existing instance if available
        if (state.instances.has(windowId)) {
          return state.instances.get(windowId)!;
        }

        // Create new instances
        const componentState = new ComponentState();
        const executor = new ToolExecutor(componentState);
        executor.setAppId(appId);

        const cache: InstanceCache = {
          state: componentState,
          executor,
          appId,
        };

        // Store in map
        set(
          (state) => {
            const newInstances = new Map(state.instances);
            newInstances.set(windowId, cache);
            return { instances: newInstances };
          },
          false,
          "getOrCreate"
        );

        return cache;
      },

      remove: (windowId: string) => {
        set(
          (state) => {
            const newInstances = new Map(state.instances);
            newInstances.delete(windowId);
            return { instances: newInstances };
          },
          false,
          "remove"
        );
      },

      clear: () => {
        set({ instances: new Map() }, false, "clear");
      },
    }),
    { name: "DynamicsInstanceStore" }
  )
);

// ============================================================================
// Convenience Hooks
// ============================================================================

/**
 * Hook to get or create dynamics instances for a window
 */
export function useDynamicsInstance(windowId: string, appId: string) {
  const getOrCreate = useInstanceStore((state) => state.getOrCreate);
  return getOrCreate(windowId, appId);
}

/**
 * Hook to clean up dynamics instances when a window closes
 */
export function useInstanceCleanup() {
  return useInstanceStore((state) => state.remove);
}
