/**
 * Dock Store Tests
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useStore } from "../../../src/features/dnd/store/store";

describe("Dock Store", () => {
  beforeEach(() => {
    useStore.getState().reset();
  });

  describe("initial state", () => {
    it("should have default items", () => {
      const { items } = useStore.getState();
      expect(items.length).toBeGreaterThan(0);
      expect(items[0]).toHaveProperty("id");
      expect(items[0]).toHaveProperty("icon");
      expect(items[0]).toHaveProperty("label");
      expect(items[0]).toHaveProperty("action");
    });
  });

  describe("add", () => {
    it("should add new item", () => {
      const { add, items } = useStore.getState();
      const initialLength = items.length;

      add({
        id: "test",
        label: "Test",
        icon: "🧪",
        action: "test-action",
      });

      const { items: newItems } = useStore.getState();
      expect(newItems).toHaveLength(initialLength + 1);
      expect(newItems[newItems.length - 1].id).toBe("test");
    });

    it("should assign correct order", () => {
      const { add, items } = useStore.getState();
      const maxOrder = Math.max(...items.map((i) => i.order));

      add({
        id: "test",
        label: "Test",
        icon: "🧪",
        action: "test",
      });

      const { items: newItems } = useStore.getState();
      const newItem = newItems.find((i) => i.id === "test");
      expect(newItem?.order).toBe(maxOrder + 1);
    });
  });

  describe("remove", () => {
    it("should remove non-pinned item", () => {
      const { add, remove, items } = useStore.getState();

      add({
        id: "removable",
        label: "Removable",
        icon: "❌",
        action: "remove",
      });

      const beforeLength = useStore.getState().items.length;
      remove("removable");

      const { items: afterItems } = useStore.getState();
      expect(afterItems).toHaveLength(beforeLength - 1);
      expect(afterItems.find((i) => i.id === "removable")).toBeUndefined();
    });

    it("should not remove pinned items", () => {
      const { items } = useStore.getState();
      const pinnedItem = items.find((i) => i.pinned);

      if (pinnedItem) {
        const { remove } = useStore.getState();
        remove(pinnedItem.id);

        const { items: afterItems } = useStore.getState();
        expect(afterItems.find((i) => i.id === pinnedItem.id)).toBeDefined();
      }
    });
  });

  describe("reorder", () => {
    it("should reorder items", () => {
      const { reorder, items } = useStore.getState();
      const firstId = items[0].id;
      const lastIndex = items.length - 1;

      reorder(0, lastIndex);

      const { items: newItems } = useStore.getState();
      expect(newItems[lastIndex].id).toBe(firstId);
    });

    it("should update order values", () => {
      const { reorder, items } = useStore.getState();

      reorder(0, 2);

      const { items: newItems } = useStore.getState();
      newItems.forEach((item, index) => {
        expect(item.order).toBe(index);
      });
    });
  });

  describe("updateOrder", () => {
    it("should update all item orders", () => {
      const { updateOrder, items } = useStore.getState();
      const reversed = [...items].reverse();

      updateOrder(reversed);

      const { items: newItems } = useStore.getState();
      expect(newItems[0].id).toBe(reversed[0].id);
      newItems.forEach((item, index) => {
        expect(item.order).toBe(index);
      });
    });
  });

  describe("toggle", () => {
    it("should toggle pinned state", () => {
      const { toggle, items } = useStore.getState();
      const item = items[0];
      const wasPinned = item.pinned || false;

      toggle(item.id);

      const { items: newItems } = useStore.getState();
      const updatedItem = newItems.find((i) => i.id === item.id);
      expect(updatedItem?.pinned).toBe(!wasPinned);
    });
  });

  describe("get", () => {
    it("should return item by id", () => {
      const { get, items } = useStore.getState();
      const firstItem = items[0];

      const item = get(firstItem.id);
      expect(item).toEqual(firstItem);
    });

    it("should return undefined for non-existent id", () => {
      const { get } = useStore.getState();
      const item = get("non-existent");
      expect(item).toBeUndefined();
    });
  });

  describe("reset", () => {
    it("should reset to default items", () => {
      const { add, reset } = useStore.getState();
      const initialItems = useStore.getState().items;

      add({
        id: "temp",
        label: "Temp",
        icon: "🔥",
        action: "temp",
      });

      reset();

      const { items } = useStore.getState();
      expect(items).toEqual(initialItems);
    });
  });
});
