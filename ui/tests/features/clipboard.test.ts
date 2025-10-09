/**
 * Clipboard Tests
 * Tests for clipboard hooks and components
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useClipboard } from "@/features/clipboard/hooks/useClipboard";
import { clipboardManager, ClipboardManager } from "@/features/clipboard/core/manager";
import type { ClipboardEntry } from "@/features/clipboard/core/types";

// Mock service client
const createMockService = () => ({
  execute: vi.fn(),
});

// Mock clipboard API
const createMockClipboardAPI = () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
  readText: vi.fn().mockResolvedValue("Mocked text"),
});

describe("ClipboardManager", () => {
  let manager: ClipboardManager;

  beforeEach(() => {
    manager = new ClipboardManager();
  });

  it("should initialize with empty state", () => {
    const state = manager.getState();
    expect(state.current).toBeNull();
    expect(state.history).toEqual([]);
    expect(state.stats).toBeNull();
    expect(state.subscribed).toBe(false);
  });

  it("should update current entry", () => {
    const entry: ClipboardEntry = {
      id: 1,
      data: { type: "Text", data: "Test" },
      source_pid: 100,
      timestamp: Date.now(),
    };

    manager.setCurrent(entry);
    const state = manager.getState();
    expect(state.current).toEqual(entry);
  });

  it("should update history", () => {
    const entries: ClipboardEntry[] = [
      { id: 1, data: { type: "Text", data: "First" }, source_pid: 100, timestamp: Date.now() },
      { id: 2, data: { type: "Text", data: "Second" }, source_pid: 100, timestamp: Date.now() },
    ];

    manager.setHistory(entries);
    const state = manager.getState();
    expect(state.history).toEqual(entries);
  });

  it("should notify subscribers on state change", () => {
    const listener = vi.fn();
    const unsubscribe = manager.subscribe(listener);

    manager.setCurrent({
      id: 1,
      data: { type: "Text", data: "Test" },
      source_pid: 100,
      timestamp: Date.now(),
    });

    expect(listener).toHaveBeenCalledTimes(1);
    unsubscribe();
  });

  it("should add entry to history", () => {
    const entry: ClipboardEntry = {
      id: 1,
      data: { type: "Text", data: "Test" },
      source_pid: 100,
      timestamp: Date.now(),
    };

    manager.addToHistory(entry);
    const state = manager.getState();
    expect(state.current).toEqual(entry);
    expect(state.history).toHaveLength(0); // First entry, no history yet

    // Add another entry
    const entry2: ClipboardEntry = {
      id: 2,
      data: { type: "Text", data: "Test 2" },
      source_pid: 100,
      timestamp: Date.now(),
    };

    manager.addToHistory(entry2);
    const state2 = manager.getState();
    expect(state2.current).toEqual(entry2);
    expect(state2.history).toHaveLength(1);
    expect(state2.history[0]).toEqual(entry);
  });

  it("should cap history at 100 entries", () => {
    // Add 150 entries
    for (let i = 0; i < 150; i++) {
      manager.addToHistory({
        id: i,
        data: { type: "Text", data: `Entry ${i}` },
        source_pid: 100,
        timestamp: Date.now(),
      });
    }

    const state = manager.getState();
    expect(state.history.length).toBeLessThanOrEqual(100);
  });

  it("should clear all state", () => {
    manager.setCurrent({
      id: 1,
      data: { type: "Text", data: "Test" },
      source_pid: 100,
      timestamp: Date.now(),
    });
    manager.setSubscribed(true);

    manager.clearState();

    const state = manager.getState();
    expect(state.current).toBeNull();
    expect(state.history).toEqual([]);
    expect(state.stats).toBeNull();
    expect(state.subscribed).toBe(false);
  });
});

describe("useClipboard Hook", () => {
  let mockService: ReturnType<typeof createMockService>;
  let originalClipboard: Clipboard | undefined;

  beforeEach(() => {
    mockService = createMockService();
    originalClipboard = navigator.clipboard;
    Object.defineProperty(navigator, "clipboard", {
      value: createMockClipboardAPI(),
      writable: true,
    });
    clipboardManager.clearState();
  });

  afterEach(() => {
    Object.defineProperty(navigator, "clipboard", {
      value: originalClipboard,
      writable: true,
    });
  });

  it("should copy text successfully", async () => {
    mockService.execute.mockResolvedValue({
      success: true,
      data: { entry_id: 123, format: "text", global: false },
    });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      const entryId = await result.current.copy("Hello, World!");
      expect(entryId).toBe(123);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.copy", {
      data: "Hello, World!",
      format: "text",
      global: false,
    });
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Hello, World!");
  });

  it("should paste text successfully", async () => {
    const mockEntry: ClipboardEntry = {
      id: 123,
      data: { type: "Text", data: "Pasted text" },
      source_pid: 100,
      timestamp: Date.now(),
    };

    mockService.execute.mockResolvedValue({
      success: true,
      data: mockEntry,
    });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      const entry = await result.current.paste();
      expect(entry).toEqual(mockEntry);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.paste", { global: false });
  });

  it("should fallback to browser clipboard when service unavailable", async () => {
    const { result } = renderHook(() => useClipboard({ autoLoad: false }));

    await act(async () => {
      await result.current.copy("Fallback text");
    });

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Fallback text");

    await act(async () => {
      const entry = await result.current.paste();
      expect(entry?.data.data).toBe("Mocked text");
    });

    expect(navigator.clipboard.readText).toHaveBeenCalled();
  });

  it("should fetch history successfully", async () => {
    const mockHistory: ClipboardEntry[] = [
      { id: 1, data: { type: "Text", data: "First" }, source_pid: 100, timestamp: Date.now() },
      { id: 2, data: { type: "Text", data: "Second" }, source_pid: 100, timestamp: Date.now() },
    ];

    mockService.execute.mockResolvedValue({
      success: true,
      data: { entries: mockHistory },
    });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      const history = await result.current.getHistory({ limit: 10 });
      expect(history).toEqual(mockHistory);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.history", {
      limit: 10,
      global: false,
    });
  });

  it("should clear clipboard", async () => {
    mockService.execute.mockResolvedValue({ success: true, data: {} });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      await result.current.clear(false);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.clear", { global: false });
  });

  it("should subscribe to clipboard changes", async () => {
    mockService.execute.mockResolvedValue({ success: true, data: {} });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      await result.current.subscribe(["text", "html"]);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.subscribe", {
      formats: ["text", "html"],
    });

    await waitFor(() => {
      expect(result.current.subscribed).toBe(true);
    });
  });

  it("should unsubscribe from clipboard changes", async () => {
    mockService.execute.mockResolvedValue({ success: true, data: {} });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      await result.current.unsubscribe();
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.unsubscribe", {});

    await waitFor(() => {
      expect(result.current.subscribed).toBe(false);
    });
  });

  it("should fetch stats successfully", async () => {
    const mockStats = {
      total_entries: 10,
      total_size: 1024,
      process_count: 2,
      global_entries: 1,
      subscriptions: 0,
    };

    mockService.execute.mockResolvedValue({
      success: true,
      data: mockStats,
    });

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      const stats = await result.current.getStats();
      expect(stats).toEqual(mockStats);
    });

    expect(mockService.execute).toHaveBeenCalledWith("clipboard.stats", {});
  });

  it("should handle copy errors", async () => {
    mockService.execute.mockRejectedValue(new Error("Copy failed"));

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      await expect(result.current.copy("Test")).rejects.toThrow("Copy failed");
    });

    await waitFor(() => {
      expect(result.current.error).toBe("Copy failed");
    });
  });

  it("should handle paste errors", async () => {
    mockService.execute.mockRejectedValue(new Error("Paste failed"));
    (navigator.clipboard.readText as any).mockRejectedValue(new Error("Browser paste failed"));

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await act(async () => {
      const entry = await result.current.paste();
      expect(entry).toBeNull();
    });

    await waitFor(() => {
      expect(result.current.error).toBe("Paste failed");
    });
  });

  it("should auto-load history on mount when enabled", async () => {
    mockService.execute.mockResolvedValue({
      success: true,
      data: { entries: [] },
    });

    renderHook(() => useClipboard({ service: mockService, autoLoad: true }));

    await waitFor(() => {
      expect(mockService.execute).toHaveBeenCalledWith("clipboard.history", {
        limit: 10,
        global: false,
      });
    });
  });

  it("should not auto-load history when disabled", async () => {
    renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    await waitFor(() => {
      expect(mockService.execute).not.toHaveBeenCalled();
    });
  });

  it("should update loading state during operations", async () => {
    mockService.execute.mockImplementation(
      () => new Promise((resolve) => setTimeout(() => resolve({ success: true, data: {} }), 100))
    );

    const { result } = renderHook(() => useClipboard({ service: mockService, autoLoad: false }));

    expect(result.current.loading).toBe(false);

    act(() => {
      result.current.copy("Test");
    });

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });
});

