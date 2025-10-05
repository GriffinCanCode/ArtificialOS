/**
 * Window Bounds Utilities Tests
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  calculateSnapZone,
  getSnapBounds,
  getMaximizedBounds,
  calculateCascadePosition,
  constrainToViewport,
  getCenterPosition,
} from "../../src/utils/windows/bounds";
import { SnapZone } from "../../src/types/windows";

// Mock window dimensions
const mockWindow = {
  innerWidth: 1920,
  innerHeight: 1080,
};

describe("Window Bounds Utilities", () => {
  beforeEach(() => {
    Object.defineProperty(window, "innerWidth", {
      writable: true,
      configurable: true,
      value: mockWindow.innerWidth,
    });
    Object.defineProperty(window, "innerHeight", {
      writable: true,
      configurable: true,
      value: mockWindow.innerHeight,
    });
  });

  describe("calculateSnapZone", () => {
    it("should detect left edge snap", () => {
      expect(calculateSnapZone(10, 500)).toBe(SnapZone.LEFT);
    });

    it("should detect right edge snap", () => {
      expect(calculateSnapZone(1910, 500)).toBe(SnapZone.RIGHT);
    });

    it("should detect top edge snap", () => {
      expect(calculateSnapZone(960, 10)).toBe(SnapZone.TOP);
    });

    it("should detect top-left corner snap", () => {
      expect(calculateSnapZone(10, 10)).toBe(SnapZone.TOP_LEFT);
    });

    it("should detect top-right corner snap", () => {
      expect(calculateSnapZone(1910, 10)).toBe(SnapZone.TOP_RIGHT);
    });

    it("should detect bottom-left corner snap", () => {
      expect(calculateSnapZone(10, 1070)).toBe(SnapZone.BOTTOM_LEFT);
    });

    it("should detect bottom-right corner snap", () => {
      expect(calculateSnapZone(1910, 1070)).toBe(SnapZone.BOTTOM_RIGHT);
    });

    it("should return NONE for center position", () => {
      expect(calculateSnapZone(960, 540)).toBe(SnapZone.NONE);
    });
  });

  describe("getSnapBounds", () => {
    it("should return left half bounds for LEFT zone", () => {
      const bounds = getSnapBounds(SnapZone.LEFT);
      expect(bounds.position.x).toBe(0);
      expect(bounds.size.width).toBe(mockWindow.innerWidth / 2);
    });

    it("should return right half bounds for RIGHT zone", () => {
      const bounds = getSnapBounds(SnapZone.RIGHT);
      expect(bounds.position.x).toBe(mockWindow.innerWidth / 2);
      expect(bounds.size.width).toBe(mockWindow.innerWidth / 2);
    });

    it("should return full width for TOP zone", () => {
      const bounds = getSnapBounds(SnapZone.TOP);
      expect(bounds.size.width).toBe(mockWindow.innerWidth);
    });

    it("should return quarter size for TOP_LEFT zone", () => {
      const bounds = getSnapBounds(SnapZone.TOP_LEFT);
      expect(bounds.position.x).toBe(0);
      expect(bounds.size.width).toBe(mockWindow.innerWidth / 2);
      expect(bounds.size.height).toBeLessThan(mockWindow.innerHeight / 2);
    });
  });

  describe("getMaximizedBounds", () => {
    it("should return full viewport bounds minus bars", () => {
      const bounds = getMaximizedBounds();
      expect(bounds.position.x).toBe(0);
      expect(bounds.size.width).toBe(mockWindow.innerWidth);
      expect(bounds.size.height).toBe(mockWindow.innerHeight - 40 - 60); // menubar + taskbar
    });
  });

  describe("calculateCascadePosition", () => {
    it("should offset position based on window count", () => {
      const pos1 = calculateCascadePosition(0);
      const pos2 = calculateCascadePosition(1);
      const pos3 = calculateCascadePosition(2);

      expect(pos2.x).toBeGreaterThan(pos1.x);
      expect(pos2.y).toBeGreaterThan(pos1.y);
      expect(pos3.x).toBeGreaterThan(pos2.x);
      expect(pos3.y).toBeGreaterThan(pos2.y);
    });

    it("should use custom offset", () => {
      const pos1 = calculateCascadePosition(1, 50);
      const pos2 = calculateCascadePosition(1, 30);

      expect(pos1.x - pos2.x).toBe(20);
      expect(pos1.y - pos2.y).toBe(20);
    });
  });

  describe("constrainToViewport", () => {
    it("should constrain position within viewport", () => {
      const bounds = {
        position: { x: -100, y: -100 },
        size: { width: 800, height: 600 },
      };

      const constrained = constrainToViewport(bounds);
      expect(constrained.position.x).toBeGreaterThanOrEqual(0);
      expect(constrained.position.y).toBeGreaterThanOrEqual(40); // menubar height
    });

    it("should constrain size to viewport", () => {
      const bounds = {
        position: { x: 100, y: 100 },
        size: { width: 3000, height: 2000 },
      };

      const constrained = constrainToViewport(bounds);
      expect(constrained.size.width).toBeLessThanOrEqual(mockWindow.innerWidth);
      expect(constrained.size.height).toBeLessThanOrEqual(mockWindow.innerHeight - 40 - 60);
    });
  });

  describe("getCenterPosition", () => {
    it("should calculate center position for given size", () => {
      const size = { width: 800, height: 600 };
      const position = getCenterPosition(size);

      expect(position.x).toBe((mockWindow.innerWidth - 800) / 2);
      expect(position.y).toBeGreaterThanOrEqual(40); // at least below menubar
    });
  });
});
