/**
 * DynamicRenderer State Management
 * Full observer pattern implementation for component state
 */

import { logger } from "../../../core/utils/monitoring/logger";

// ============================================================================
// Type Definitions
// ============================================================================

/**
 * State change event details
 */
export interface StateChangeEvent<T = any> {
  key: string;
  oldValue: T | undefined;
  newValue: T;
  timestamp: number;
}

/**
 * Middleware function for intercepting state changes
 */
export type StateMiddleware = (event: StateChangeEvent) => StateChangeEvent | null;

/**
 * Computed value definition
 */
export interface ComputedValue<T = any> {
  dependencies: string[];
  compute: (state: Map<string, any>) => T;
  cached?: T;
  isDirty: boolean;
}

/**
 * Subscription options
 */
export interface SubscriptionOptions {
  immediate?: boolean; // Call listener immediately with current value
  debounce?: number; // Debounce notifications (ms)
  filter?: (value: any) => boolean; // Filter which values trigger notifications
}

// ============================================================================
// Component State Manager - Full Observer Pattern Implementation
// ============================================================================

/**
 * Full-featured Component State Manager with Observer Pattern
 *
 * Features:
 * - Observable state with pub/sub
 * - Batch updates for performance
 * - Computed/derived values
 * - Middleware for validation/transformation
 * - Wildcard subscriptions (e.g., "user.*")
 * - State history for time-travel debugging
 * - Debounced notifications
 * - Type-safe subscriptions
 */
export class ComponentState {
  private state: Map<string, any> = new Map();
  private listeners: Map<string, Set<(value: any, oldValue?: any) => void>> = new Map();
  private wildcardListeners: Map<string, Set<(event: StateChangeEvent) => void>> = new Map();
  private computed: Map<string, ComputedValue> = new Map();
  private middleware: StateMiddleware[] = [];
  private history: StateChangeEvent[] = [];
  private maxHistorySize = 50;
  private isBatching = false;
  private batchedChanges: StateChangeEvent[] = [];
  private debounceTimers: Map<string, number> = new Map();

  // ============================================================================
  // Core State Operations
  // ============================================================================

  /**
   * Get a value from state
   */
  get<T = any>(key: string, defaultValue?: T): T {
    // Check if it's a computed value
    if (this.computed.has(key)) {
      return this.getComputed<T>(key);
    }

    const value = this.state.get(key);
    return (value !== undefined ? value : defaultValue) as T;
  }

  /**
   * Set a value in state and notify listeners
   */
  set<T = any>(key: string, value: T): void {
    const oldValue = this.state.get(key);

    // Skip if value hasn't changed (shallow comparison)
    if (oldValue === value) {
      return;
    }

    // Create change event
    let event: StateChangeEvent<T> = {
      key,
      oldValue,
      newValue: value,
      timestamp: Date.now(),
    };

    // Run through middleware
    for (const mw of this.middleware) {
      const result = mw(event);
      if (result === null) {
        // Middleware rejected the change
        logger.debug("State change rejected by middleware", {
          component: "ComponentState",
          key,
        });
        return;
      }
      event = result;
    }

    // Update state
    this.state.set(key, event.newValue);

    // Mark dependent computed values as dirty
    this.invalidateComputedDependents(key);

    // Add to history
    this.addToHistory(event);

    // Handle batching
    if (this.isBatching) {
      this.batchedChanges.push(event);
    } else {
      this.notifyListeners(event);
    }
  }

  /**
   * Set multiple values at once (batched)
   */
  setMultiple(changes: Record<string, any>): void {
    this.batch(() => {
      Object.entries(changes).forEach(([key, value]) => {
        this.set(key, value);
      });
    });
  }

  /**
   * Check if a key exists in state
   */
  has(key: string): boolean {
    return this.state.has(key) || this.computed.has(key);
  }

  /**
   * Delete a key from state
   */
  delete(key: string): void {
    const oldValue = this.state.get(key);
    if (oldValue !== undefined) {
      this.state.delete(key);
      const event: StateChangeEvent = {
        key,
        oldValue,
        newValue: undefined as any,
        timestamp: Date.now(),
      };
      this.addToHistory(event);
      this.notifyListeners(event);
    }
  }

  /**
   * Get all state as object
   */
  getAll(): Record<string, any> {
    const result: Record<string, any> = {};
    this.state.forEach((value, key) => {
      result[key] = value;
    });
    // Include computed values
    this.computed.forEach((_, key) => {
      result[key] = this.getComputed(key);
    });
    return result;
  }

  /**
   * Clear all state and listeners
   */
  clear(): void {
    this.state.clear();
    this.listeners.clear();
    this.wildcardListeners.clear();
    this.computed.clear();
    this.history = [];
    this.batchedChanges = [];

    // Clear any active debounce timers
    this.debounceTimers.forEach((timerId) => clearTimeout(timerId));
    this.debounceTimers.clear();
  }

  // ============================================================================
  // Subscriptions
  // ============================================================================

  /**
   * Subscribe to state changes for a specific key
   * For wildcard patterns (e.g., "user.*"), listener signature is (event: StateChangeEvent) => void
   * For specific keys, listener signature is (value: any, oldValue?: any) => void
   */
  subscribe(
    key: string,
    listener: ((value: any, oldValue?: any) => void) | ((event: StateChangeEvent) => void),
    options?: SubscriptionOptions
  ): () => void {
    // Handle wildcard subscriptions (e.g., "user.*")
    if (key.includes("*")) {
      return this.subscribeWildcard(key, listener as (event: StateChangeEvent) => void, options);
    }

    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }

    // Wrap listener with options (debounce, filter)
    const wrappedListener = this.wrapListener(
      listener as (value: any, oldValue?: any) => void,
      options
    );
    this.listeners.get(key)!.add(wrappedListener);

    // Call immediately if requested
    if (options?.immediate) {
      const currentValue = this.get(key);
      (listener as (value: any, oldValue?: any) => void)(currentValue, undefined);
    }

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(key);
      if (listeners) {
        listeners.delete(wrappedListener);
        if (listeners.size === 0) {
          this.listeners.delete(key);
        }
      }
    };
  }

  /**
   * Subscribe to multiple keys at once
   */
  subscribeMultiple(
    keys: string[],
    listener: (changes: Record<string, any>) => void,
    options?: SubscriptionOptions
  ): () => void {
    const unsubscribers: (() => void)[] = [];
    const values: Record<string, any> = {};

    keys.forEach((key) => {
      const unsub = this.subscribe(
        key,
        (newValue: any) => {
          values[key] = newValue;
          listener({ ...values });
        },
        options
      );
      unsubscribers.push(unsub);
    });

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }

  /**
   * Subscribe with wildcard pattern (e.g., "user.*" matches "user.name", "user.age")
   */
  subscribeWildcard(
    pattern: string,
    listener: (event: StateChangeEvent) => void,
    options?: SubscriptionOptions
  ): () => void {
    if (!this.wildcardListeners.has(pattern)) {
      this.wildcardListeners.set(pattern, new Set());
    }

    const wrappedListener = this.wrapWildcardListener(listener, pattern, options);
    this.wildcardListeners.get(pattern)!.add(wrappedListener);

    return () => {
      const listeners = this.wildcardListeners.get(pattern);
      if (listeners) {
        listeners.delete(wrappedListener);
        if (listeners.size === 0) {
          this.wildcardListeners.delete(pattern);
        }
      }
    };
  }

  /**
   * Wrap listener with debouncing and filtering
   */
  private wrapListener(
    listener: (value: any, oldValue?: any) => void,
    options?: SubscriptionOptions
  ): (value: any, oldValue?: any) => void {
    let wrappedListener = listener;

    // Add filter
    if (options?.filter) {
      const originalListener = wrappedListener;
      wrappedListener = (value: any, oldValue?: any) => {
        if (options.filter!(value)) {
          originalListener(value, oldValue);
        }
      };
    }

    // Add debounce
    if (options?.debounce) {
      const originalListener = wrappedListener;
      const debounceMs = options.debounce;
      wrappedListener = (value: any, oldValue?: any) => {
        const timerId = this.debounceTimers.get(listener.toString());
        if (timerId !== undefined) {
          clearTimeout(timerId);
        }
        const newTimerId = window.setTimeout(() => {
          originalListener(value, oldValue);
          this.debounceTimers.delete(listener.toString());
        }, debounceMs);
        this.debounceTimers.set(listener.toString(), newTimerId);
      };
    }

    return wrappedListener;
  }

  /**
   * Wrap wildcard listener with pattern matching
   */
  private wrapWildcardListener(
    listener: (event: StateChangeEvent) => void,
    pattern: string,
    options?: SubscriptionOptions
  ): (event: StateChangeEvent) => void {
    const regex = new RegExp("^" + pattern.replace(/\*/g, ".*") + "$");

    return (event: StateChangeEvent) => {
      if (regex.test(event.key)) {
        if (!options?.filter || options.filter(event.newValue)) {
          listener(event);
        }
      }
    };
  }

  /**
   * Notify all listeners of a state change
   */
  private notifyListeners(event: StateChangeEvent): void {
    // Notify direct listeners
    const listeners = this.listeners.get(event.key);
    if (listeners) {
      listeners.forEach((listener) => {
        try {
          listener(event.newValue, event.oldValue);
        } catch (error) {
          logger.error("Error in state listener", error as Error, {
            component: "ComponentState",
            key: event.key,
          });
        }
      });
    }

    // Notify wildcard listeners
    this.wildcardListeners.forEach((listeners, pattern) => {
      const regex = new RegExp("^" + pattern.replace(/\*/g, ".*") + "$");
      if (regex.test(event.key)) {
        listeners.forEach((listener) => {
          try {
            listener(event);
          } catch (error) {
            logger.error("Error in wildcard listener", error as Error, {
              component: "ComponentState",
              pattern,
              key: event.key,
            });
          }
        });
      }
    });
  }

  // ============================================================================
  // Batch Updates
  // ============================================================================

  /**
   * Execute multiple state changes as a batch
   * Listeners are only notified once after all changes complete
   */
  batch(callback: () => void): void {
    const wasAlreadyBatching = this.isBatching;

    if (!wasAlreadyBatching) {
      this.isBatching = true;
      this.batchedChanges = [];
    }

    try {
      callback();
    } finally {
      if (!wasAlreadyBatching) {
        this.isBatching = false;

        // Notify all batched changes
        const changes = this.batchedChanges;
        this.batchedChanges = [];

        changes.forEach((event) => {
          this.notifyListeners(event);
        });
      }
    }
  }

  // ============================================================================
  // Computed Values
  // ============================================================================

  /**
   * Define a computed value that depends on other state values
   */
  defineComputed<T = any>(
    key: string,
    dependencies: string[],
    compute: (state: Map<string, any>) => T
  ): void {
    this.computed.set(key, {
      dependencies,
      compute,
      isDirty: true,
    });
  }

  /**
   * Get a computed value (with caching)
   */
  private getComputed<T = any>(key: string): T {
    const computed = this.computed.get(key);
    if (!computed) {
      throw new Error(`Computed value "${key}" not defined`);
    }

    // Recompute if dirty
    if (computed.isDirty) {
      computed.cached = computed.compute(this.state);
      computed.isDirty = false;
    }

    return computed.cached as T;
  }

  /**
   * Mark computed values that depend on a key as dirty
   */
  private invalidateComputedDependents(key: string): void {
    this.computed.forEach((computed, computedKey) => {
      if (computed.dependencies.includes(key)) {
        computed.isDirty = true;
        // Notify listeners of computed value change
        const event: StateChangeEvent = {
          key: computedKey,
          oldValue: computed.cached,
          newValue: undefined as any, // Will be recomputed on access
          timestamp: Date.now(),
        };
        this.notifyListeners(event);
      }
    });
  }

  // ============================================================================
  // Middleware
  // ============================================================================

  /**
   * Add middleware to intercept/transform state changes
   */
  use(middleware: StateMiddleware): () => void {
    this.middleware.push(middleware);

    // Return function to remove middleware
    return () => {
      const index = this.middleware.indexOf(middleware);
      if (index > -1) {
        this.middleware.splice(index, 1);
      }
    };
  }

  // ============================================================================
  // History / Time Travel
  // ============================================================================

  /**
   * Add event to history
   */
  private addToHistory(event: StateChangeEvent): void {
    this.history.push(event);

    // Limit history size
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
    }
  }

  /**
   * Get state change history
   */
  getHistory(): StateChangeEvent[] {
    return [...this.history];
  }

  /**
   * Get history for a specific key
   */
  getHistoryForKey(key: string): StateChangeEvent[] {
    return this.history.filter((event) => event.key === key);
  }

  /**
   * Restore state to a previous point in time
   */
  restoreToTimestamp(timestamp: number): void {
    const targetIndex = this.history.findIndex((e) => e.timestamp >= timestamp);
    if (targetIndex === -1) return;

    // Clear current state
    this.state.clear();

    // Replay history up to target timestamp
    const eventsToReplay = this.history.slice(0, targetIndex);
    this.batch(() => {
      eventsToReplay.forEach((event) => {
        if (event.newValue !== undefined) {
          this.state.set(event.key, event.newValue);
        }
      });
    });
  }

  /**
   * Clear history
   */
  clearHistory(): void {
    this.history = [];
  }

  /**
   * Set max history size
   */
  setMaxHistorySize(size: number): void {
    this.maxHistorySize = size;

    // Trim if needed
    if (this.history.length > size) {
      this.history = this.history.slice(-size);
    }
  }

  // ============================================================================
  // Debug Helpers
  // ============================================================================

  /**
   * Get debug info about the state manager
   */
  getDebugInfo(): {
    stateSize: number;
    listenerCount: number;
    wildcardListenerCount: number;
    computedCount: number;
    historySize: number;
    isBatching: boolean;
  } {
    let listenerCount = 0;
    this.listeners.forEach((set) => (listenerCount += set.size));

    let wildcardListenerCount = 0;
    this.wildcardListeners.forEach((set) => (wildcardListenerCount += set.size));

    return {
      stateSize: this.state.size,
      listenerCount,
      wildcardListenerCount,
      computedCount: this.computed.size,
      historySize: this.history.length,
      isBatching: this.isBatching,
    };
  }

  /**
   * Log current state (for debugging)
   */
  logState(): void {
    logger.debug("ComponentState snapshot", {
      component: "ComponentState",
      state: this.getAll(),
      debug: this.getDebugInfo(),
    });
  }
}
