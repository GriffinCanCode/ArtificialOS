/**
 * useLogger Hook Tests
 * Tests custom logger hook functionality
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import { useLogger } from "../../src/utils/monitoring/useLogger";
import { logger } from "../../src/utils/monitoring/logger";

// Mock the logger
vi.mock("../../src/utils/monitoring/logger", () => ({
  logger: {
    info: vi.fn(),
    debug: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    verbose: vi.fn(),
    child: vi.fn(() => ({
      info: vi.fn(),
      debug: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
      verbose: vi.fn(),
    })),
  },
}));

describe("useLogger", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("creates a logger with component name", () => {
    renderHook(() => useLogger("TestComponent"));

    expect(logger.child).toHaveBeenCalledWith({
      component: "TestComponent",
    });
  });

  it("includes additional context", () => {
    renderHook(() =>
      useLogger("TestComponent", { userId: "123", action: "click" })
    );

    expect(logger.child).toHaveBeenCalledWith({
      component: "TestComponent",
      userId: "123",
      action: "click",
    });
  });

  it("returns logger methods", () => {
    const { result } = renderHook(() => useLogger("TestComponent"));

    expect(result.current).toHaveProperty("info");
    expect(result.current).toHaveProperty("debug");
    expect(result.current).toHaveProperty("warn");
    expect(result.current).toHaveProperty("error");
    expect(result.current).toHaveProperty("verbose");
  });

  it("uses the same logger instance across renders", () => {
    const { result, rerender } = renderHook(() =>
      useLogger("TestComponent")
    );

    const firstLogger = result.current;
    rerender();
    const secondLogger = result.current;

    expect(firstLogger).toBe(secondLogger);
  });

  it("creates new logger when component name changes", () => {
    const { result, rerender } = renderHook(
      ({ name }) => useLogger(name),
      { initialProps: { name: "Component1" } }
    );

    const firstLogger = result.current;

    rerender({ name: "Component2" });
    const secondLogger = result.current;

    expect(firstLogger).not.toBe(secondLogger);
  });

  it("updates logger when additional context changes", () => {
    const { result, rerender } = renderHook(
      ({ context }) => useLogger("TestComponent", context),
      { initialProps: { context: { userId: "123" } } }
    );

    const firstLogger = result.current;

    rerender({ context: { userId: "456" } });
    const secondLogger = result.current;

    expect(firstLogger).not.toBe(secondLogger);
  });
});

