/**
 * Component State Tests
 * Tests the observer pattern state management
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { ComponentState } from "../../../src/components/dynamics/DynamicRenderer.state";

describe("ComponentState", () => {
  let state: ComponentState;

  beforeEach(() => {
    state = new ComponentState();
  });

  describe("Basic Operations", () => {
    it("sets and gets values", () => {
      state.set("key", "value");
      expect(state.get("key")).toBe("value");
    });

    it("returns default value when key doesn't exist", () => {
      expect(state.get("nonexistent", "default")).toBe("default");
    });

    it("checks if key exists", () => {
      state.set("key", "value");
      expect(state.has("key")).toBe(true);
      expect(state.has("other")).toBe(false);
    });

    it("deletes keys", () => {
      state.set("key", "value");
      state.delete("key");
      expect(state.has("key")).toBe(false);
    });

    it("gets all state as object", () => {
      state.set("key1", "value1");
      state.set("key2", "value2");

      const allState = state.getAll();
      expect(allState).toEqual({
        key1: "value1",
        key2: "value2",
      });
    });

    it("clears all state", () => {
      state.set("key1", "value1");
      state.set("key2", "value2");
      state.clear();

      expect(state.has("key1")).toBe(false);
      expect(state.has("key2")).toBe(false);
    });

    it("sets multiple values at once", () => {
      state.setMultiple({
        key1: "value1",
        key2: "value2",
        key3: "value3",
      });

      expect(state.get("key1")).toBe("value1");
      expect(state.get("key2")).toBe("value2");
      expect(state.get("key3")).toBe("value3");
    });
  });

  describe("Subscriptions", () => {
    it("notifies listeners on value change", () => {
      const listener = vi.fn();
      state.subscribe("key", listener);

      state.set("key", "value");

      expect(listener).toHaveBeenCalledWith("value", undefined);
    });

    it("provides old value to listeners", () => {
      const listener = vi.fn();
      state.set("key", "old");
      state.subscribe("key", listener);

      state.set("key", "new");

      expect(listener).toHaveBeenCalledWith("new", "old");
    });

    it("does not notify if value hasn't changed", () => {
      const listener = vi.fn();
      state.set("key", "value");
      state.subscribe("key", listener);

      state.set("key", "value");

      expect(listener).not.toHaveBeenCalled();
    });

    it("unsubscribes correctly", () => {
      const listener = vi.fn();
      const unsubscribe = state.subscribe("key", listener);

      state.set("key", "value1");
      expect(listener).toHaveBeenCalledTimes(1);

      unsubscribe();
      state.set("key", "value2");
      expect(listener).toHaveBeenCalledTimes(1); // Not called again
    });

    it("calls listener immediately with immediate option", () => {
      const listener = vi.fn();
      state.set("key", "value");

      state.subscribe("key", listener, { immediate: true });

      expect(listener).toHaveBeenCalledWith("value", undefined);
    });

    it("subscribes to multiple keys", () => {
      const listener = vi.fn();
      state.subscribeMultiple(["key1", "key2"], listener);

      state.set("key1", "value1");
      expect(listener).toHaveBeenCalledWith({ key1: "value1" });

      state.set("key2", "value2");
      expect(listener).toHaveBeenCalledWith({ key1: "value1", key2: "value2" });
    });
  });

  describe("Wildcard Subscriptions", () => {
    it("subscribes with wildcard pattern", () => {
      const listener = vi.fn();
      state.subscribeWildcard("user.*", listener);

      state.set("user.name", "John");
      state.set("user.age", 30);

      expect(listener).toHaveBeenCalledTimes(2);
    });

    it("does not trigger wildcard for non-matching keys", () => {
      const listener = vi.fn();
      state.subscribeWildcard("user.*", listener);

      state.set("product.name", "Widget");

      expect(listener).not.toHaveBeenCalled();
    });

    it("provides event object to wildcard listeners", () => {
      const listener = vi.fn();
      state.subscribeWildcard("user.*", listener);

      state.set("user.name", "John");

      expect(listener).toHaveBeenCalledWith(
        expect.objectContaining({
          key: "user.name",
          newValue: "John",
          oldValue: undefined,
          timestamp: expect.any(Number),
        })
      );
    });
  });

  describe("Batch Updates", () => {
    it("batches multiple changes", () => {
      const listener1 = vi.fn();
      const listener2 = vi.fn();

      state.subscribe("key1", listener1);
      state.subscribe("key2", listener2);

      state.batch(() => {
        state.set("key1", "value1");
        state.set("key2", "value2");
      });

      expect(listener1).toHaveBeenCalledTimes(1);
      expect(listener2).toHaveBeenCalledTimes(1);
    });

    it("notifies after batch completes", () => {
      const values: string[] = [];
      state.subscribe("key", (val) => values.push(val));

      state.batch(() => {
        state.set("key", "value1");
        state.set("key", "value2");
        state.set("key", "value3");
      });

      // Should have final value only
      expect(values).toEqual(["value3"]);
    });
  });

  describe("Computed Values", () => {
    it("defines and gets computed values", () => {
      state.set("firstName", "John");
      state.set("lastName", "Doe");

      state.defineComputed("fullName", ["firstName", "lastName"], (stateMap) => {
        return `${stateMap.get("firstName")} ${stateMap.get("lastName")}`;
      });

      expect(state.get("fullName")).toBe("John Doe");
    });

    it("recomputes when dependencies change", () => {
      state.set("count", 10);
      state.defineComputed("doubled", ["count"], (stateMap) => {
        return (stateMap.get("count") || 0) * 2;
      });

      expect(state.get("doubled")).toBe(20);

      state.set("count", 15);
      expect(state.get("doubled")).toBe(30);
    });

    it("caches computed values", () => {
      let callCount = 0;
      state.set("value", 5);

      state.defineComputed("expensive", ["value"], (stateMap) => {
        callCount++;
        return stateMap.get("value");
      });

      state.get("expensive");
      state.get("expensive");
      state.get("expensive");

      expect(callCount).toBe(1); // Only computed once
    });

    it("invalidates cache when dependency changes", () => {
      let callCount = 0;
      state.set("value", 5);

      state.defineComputed("expensive", ["value"], (stateMap) => {
        callCount++;
        return stateMap.get("value");
      });

      state.get("expensive"); // Call 1
      state.set("value", 10);
      state.get("expensive"); // Call 2

      expect(callCount).toBe(2);
    });
  });

  describe("Middleware", () => {
    it("transforms values through middleware", () => {
      state.use((event) => ({
        ...event,
        newValue: event.newValue.toUpperCase(),
      }));

      state.set("key", "hello");
      expect(state.get("key")).toBe("HELLO");
    });

    it("rejects changes by returning null", () => {
      state.use((event) => {
        if (event.newValue < 0) return null;
        return event;
      });

      state.set("number", 10);
      expect(state.get("number")).toBe(10);

      state.set("number", -5);
      expect(state.get("number")).toBe(10); // Unchanged
    });

    it("chains multiple middleware", () => {
      state.use((event) => ({ ...event, newValue: event.newValue * 2 }));
      state.use((event) => ({ ...event, newValue: event.newValue + 10 }));

      state.set("value", 5);
      expect(state.get("value")).toBe(20); // (5 * 2) + 10
    });

    it("removes middleware with unsubscribe", () => {
      const removeMiddleware = state.use((event) => ({
        ...event,
        newValue: event.newValue * 2,
      }));

      state.set("value", 5);
      expect(state.get("value")).toBe(10);

      removeMiddleware();
      state.set("value", 5);
      expect(state.get("value")).toBe(5); // No transformation
    });
  });

  describe("History", () => {
    it("records state changes in history", () => {
      state.set("key1", "value1");
      state.set("key2", "value2");

      const history = state.getHistory();
      expect(history).toHaveLength(2);
      expect(history[0].key).toBe("key1");
      expect(history[1].key).toBe("key2");
    });

    it("gets history for specific key", () => {
      state.set("key1", "value1");
      state.set("key2", "value2");
      state.set("key1", "value3");

      const history = state.getHistoryForKey("key1");
      expect(history).toHaveLength(2);
      expect(history[0].newValue).toBe("value1");
      expect(history[1].newValue).toBe("value3");
    });

    it("limits history size", () => {
      state.setMaxHistorySize(5);

      for (let i = 0; i < 10; i++) {
        state.set("key", `value${i}`);
      }

      const history = state.getHistory();
      expect(history.length).toBeLessThanOrEqual(5);
    });

    it("clears history", () => {
      state.set("key", "value");
      state.clearHistory();

      expect(state.getHistory()).toHaveLength(0);
    });
  });

  describe("Debug Info", () => {
    it("provides debug information", () => {
      state.set("key1", "value1");
      state.set("key2", "value2");
      state.subscribe("key1", () => {});
      state.subscribeWildcard("*", () => {});

      const debugInfo = state.getDebugInfo();

      expect(debugInfo.stateSize).toBe(2);
      expect(debugInfo.listenerCount).toBe(1);
      expect(debugInfo.wildcardListenerCount).toBe(1);
      expect(debugInfo.isBatching).toBe(false);
    });
  });

  describe("Debouncing", () => {
    it("debounces listener notifications", async () => {
      const listener = vi.fn();
      state.subscribe("key", listener, { debounce: 100 });

      state.set("key", "value1");
      state.set("key", "value2");
      state.set("key", "value3");

      expect(listener).not.toHaveBeenCalled();

      await new Promise((resolve) => setTimeout(resolve, 150));

      expect(listener).toHaveBeenCalledTimes(1);
      expect(listener).toHaveBeenCalledWith("value3", undefined);
    });
  });

  describe("Filtering", () => {
    it("filters listener notifications", () => {
      const listener = vi.fn();
      state.subscribe("key", listener, {
        filter: (value) => value > 5,
      });

      state.set("key", 3);
      expect(listener).not.toHaveBeenCalled();

      state.set("key", 10);
      expect(listener).toHaveBeenCalledWith(10, 3);
    });
  });
});
