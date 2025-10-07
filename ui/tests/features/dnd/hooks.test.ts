/**
 * DND Hooks Tests
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSortable } from "../../../src/features/dnd/hooks/useSortable";
import { useDropzone } from "../../../src/features/dnd/hooks/useDropzone";

describe("useSortable", () => {
  const mockItems = [
    { id: "1", name: "Item 1" },
    { id: "2", name: "Item 2" },
    { id: "3", name: "Item 3" },
  ];

  it("should initialize with items", () => {
    const { result } = renderHook(() => useSortable({ items: mockItems }));
    expect(result.current.items).toEqual(mockItems);
  });

  it("should track active id during drag", () => {
    const { result } = renderHook(() => useSortable({ items: mockItems }));

    act(() => {
      result.current.handleDragStart({ active: { id: "1" } } as any);
    });

    expect(result.current.activeId).toBe("1");
  });

  it("should clear active id on drag end", () => {
    const { result } = renderHook(() => useSortable({ items: mockItems }));

    act(() => {
      result.current.handleDragStart({ active: { id: "1" } } as any);
    });

    act(() => {
      result.current.handleDragEnd({
        active: { id: "1" },
        over: { id: "2" },
      } as any);
    });

    expect(result.current.activeId).toBeNull();
  });

  it("should reorder items on drag end", () => {
    const { result } = renderHook(() => useSortable({ items: mockItems }));

    act(() => {
      result.current.handleDragEnd({
        active: { id: "1" },
        over: { id: "3" },
      } as any);
    });

    expect(result.current.items[0].id).toBe("2");
    expect(result.current.items[1].id).toBe("3");
    expect(result.current.items[2].id).toBe("1");
  });

  it("should call onSort callback", () => {
    const onSort = vi.fn();
    const { result } = renderHook(() =>
      useSortable({
        items: mockItems,
        onSort,
      })
    );

    act(() => {
      result.current.handleDragEnd({
        active: { id: "1" },
        over: { id: "2" },
      } as any);
    });

    expect(onSort).toHaveBeenCalledWith({
      activeId: "1",
      overId: "2",
      oldIndex: 0,
      newIndex: 1,
    });
  });

  it("should not reorder when disabled", () => {
    const onSort = vi.fn();
    const { result } = renderHook(() =>
      useSortable({
        items: mockItems,
        onSort,
        disabled: true,
      })
    );

    act(() => {
      result.current.handleDragStart({ active: { id: "1" } } as any);
    });

    expect(result.current.activeId).toBeNull();
  });

  it("should manually move items", () => {
    const { result } = renderHook(() => useSortable({ items: mockItems }));

    act(() => {
      result.current.moveItem(0, 2);
    });

    expect(result.current.items[0].id).toBe("2");
    expect(result.current.items[2].id).toBe("1");
  });
});

describe("useDropzone", () => {
  it("should initialize with default state", () => {
    const { result } = renderHook(() => useDropzone());

    expect(result.current.isDragging).toBe(false);
    expect(result.current.files).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it("should provide root props", () => {
    const { result } = renderHook(() => useDropzone());
    const rootProps = result.current.getRootProps();

    expect(rootProps).toHaveProperty("onDrop");
    expect(rootProps).toHaveProperty("onDragOver");
    expect(rootProps).toHaveProperty("onDragEnter");
    expect(rootProps).toHaveProperty("onDragLeave");
    expect(rootProps).toHaveProperty("onClick");
  });

  it("should provide input props", () => {
    const { result } = renderHook(() => useDropzone({ accept: ["image/*"], multiple: true }));
    const inputProps = result.current.getInputProps();

    expect(inputProps.type).toBe("file");
    expect(inputProps.multiple).toBe(true);
    expect(inputProps.accept).toBe("image/*");
  });

  it("should clear files", () => {
    const { result } = renderHook(() => useDropzone());

    // Simulate files being added
    act(() => {
      (result.current as any).files = [{ file: new File([""], "test.txt") }];
    });

    act(() => {
      result.current.clearFiles();
    });

    expect(result.current.files).toEqual([]);
  });

  it("should call onDrop callback", () => {
    const onDrop = vi.fn();
    const { result } = renderHook(() => useDropzone({ onDrop }));

    const file = new File(["content"], "test.txt", { type: "text/plain" });
    const dataTransfer = {
      files: [file] as any,
    };

    act(() => {
      const rootProps = result.current.getRootProps();
      rootProps.onDrop({
        preventDefault: vi.fn(),
        stopPropagation: vi.fn(),
        dataTransfer,
      } as any);
    });

    expect(onDrop).toHaveBeenCalled();
  });
});
