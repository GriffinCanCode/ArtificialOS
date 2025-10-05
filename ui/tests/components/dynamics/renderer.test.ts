/**
 * Component Renderer Tests
 * Tests for the new component registry and renderer system
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "@testing-library/react";
import React from "react";
import { registry } from "../../../src/features/dynamics/core/registry";
import { ComponentRenderer } from "../../../src/features/dynamics/rendering/renderer";
import { ComponentState } from "../../../src/features/dynamics/state/state";
import { ToolExecutor } from "../../../src/features/dynamics/execution/executor";
import type { BlueprintComponent } from "../../../src/core/store/appStore";

describe("Component Registry", () => {
  beforeEach(() => {
    registry.clear();
  });

  it("registers a component renderer", () => {
    const mockRenderer = {
      type: "test-component",
      render: vi.fn(() => React.createElement("div", null, "Test")),
      category: "primitive" as const,
    };

    registry.register(mockRenderer);

    expect(registry.has("test-component")).toBe(true);
    expect(registry.get("test-component")).toEqual(mockRenderer);
  });

  it("registers multiple renderers at once", () => {
    const renderers = [
      { type: "comp-1", render: vi.fn(), category: "primitive" as const },
      { type: "comp-2", render: vi.fn(), category: "layout" as const },
    ];

    registry.registerAll(renderers);

    expect(registry.has("comp-1")).toBe(true);
    expect(registry.has("comp-2")).toBe(true);
  });

  it("returns undefined for unregistered component", () => {
    expect(registry.get("nonexistent")).toBeUndefined();
  });

  it("gets all registered types", () => {
    registry.register({ type: "button", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "input", render: vi.fn(), category: "primitive" as const });

    const types = registry.getTypes();
    expect(types).toContain("button");
    expect(types).toContain("input");
    expect(types).toHaveLength(2);
  });

  it("gets renderers by category", () => {
    registry.register({ type: "button", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "input", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "container", render: vi.fn(), category: "layout" as const });

    const primitives = registry.getByCategory("primitive");
    expect(primitives).toHaveLength(2);
    expect(primitives.map((r) => r.type)).toContain("button");
    expect(primitives.map((r) => r.type)).toContain("input");
  });

  it("provides registry statistics", () => {
    registry.register({ type: "button", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "input", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "container", render: vi.fn(), category: "layout" as const });

    const stats = registry.getStats();
    expect(stats.total).toBe(3);
    expect(stats.byCategory.primitive).toBe(2);
    expect(stats.byCategory.layout).toBe(1);
  });

  it("unregisters a component renderer", () => {
    registry.register({ type: "button", render: vi.fn(), category: "primitive" as const });
    expect(registry.has("button")).toBe(true);

    registry.unregister("button");
    expect(registry.has("button")).toBe(false);
  });

  it("clears all registrations", () => {
    registry.register({ type: "button", render: vi.fn(), category: "primitive" as const });
    registry.register({ type: "input", render: vi.fn(), category: "primitive" as const });

    registry.clear();

    expect(registry.getTypes()).toHaveLength(0);
  });
});

describe("Component Renderer", () => {
  let state: ComponentState;
  let executor: ToolExecutor;

  beforeEach(() => {
    registry.clear();
    state = new ComponentState();
    executor = new ToolExecutor(state);
  });

  it("renders a registered component", () => {
    const TestComponent = () => React.createElement("div", null, "Test Component");
    registry.register({
      type: "test",
      render: TestComponent,
      category: "primitive" as const,
    });

    const component: BlueprintComponent = {
      type: "test",
      id: "test-1",
      props: {},
    };

    const { container } = render(
      React.createElement(ComponentRenderer, {
        component,
        state,
        executor,
      })
    );

    expect(container.textContent).toBe("Test Component");
  });

  it("returns null for unknown component type", () => {
    const component: BlueprintComponent = {
      type: "nonexistent",
      id: "unknown-1",
      props: {},
    };

    const { container } = render(
      React.createElement(ComponentRenderer, {
        component,
        state,
        executor,
      })
    );

    expect(container.textContent).toBe("");
  });

  it("passes props to renderer", () => {
    const mockRender = vi.fn(() => React.createElement("div", null, "Rendered"));
    registry.register({
      type: "test",
      render: mockRender,
      category: "primitive" as const,
    });

    const component: BlueprintComponent = {
      type: "test",
      id: "test-1",
      props: { text: "Hello" },
    };

    render(
      React.createElement(ComponentRenderer, {
        component,
        state,
        executor,
      })
    );

    expect(mockRender).toHaveBeenCalledWith(
      expect.objectContaining({
        component,
        state,
        executor,
      }),
      expect.anything()
    );
  });
});
