/**
 * Window Store Tests
 * Comprehensive tests for window management state
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useWindowStore } from "../../src/store/windowStore";
import { WindowState as WinState } from "../../src/types/windows";

describe("WindowStore", () => {
  beforeEach(() => {
    // Reset store before each test
    useWindowStore.getState().clearAllWindows();
  });

  describe("openWindow", () => {
    it("should create a new window with correct properties", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test App", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint, "🧪");

      const windows = store.windows;
      expect(windows).toHaveLength(1);
      expect(windows[0].id).toBe(windowId);
      expect(windows[0].appId).toBe("test-app");
      expect(windows[0].title).toBe("Test App");
      expect(windows[0].icon).toBe("🧪");
      expect(windows[0].isFocused).toBe(true);
      expect(windows[0].isMinimized).toBe(false);
      expect(windows[0].state).toBe(WinState.NORMAL);
    });

    it("should unfocus existing windows when opening new window", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      store.openWindow("app1", "App 1", mockBlueprint);
      store.openWindow("app2", "App 2", mockBlueprint);

      const windows = store.windows;
      expect(windows[0].isFocused).toBe(false);
      expect(windows[1].isFocused).toBe(true);
    });

    it("should cascade window positions", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      store.openWindow("app1", "App 1", mockBlueprint);
      store.openWindow("app2", "App 2", mockBlueprint);

      const windows = store.windows;
      expect(windows[1].position.x).toBeGreaterThan(windows[0].position.x);
      expect(windows[1].position.y).toBeGreaterThan(windows[0].position.y);
    });
  });

  describe("closeWindow", () => {
    it("should remove window from store", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      store.closeWindow(windowId);

      expect(store.windows).toHaveLength(0);
    });

    it("should focus topmost window after closing focused window", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const window1 = store.openWindow("app1", "App 1", mockBlueprint);
      const window2 = store.openWindow("app2", "App 2", mockBlueprint);

      store.closeWindow(window2);

      const windows = store.windows;
      expect(windows).toHaveLength(1);
      expect(windows[0].id).toBe(window1);
      expect(windows[0].isFocused).toBe(true);
    });
  });

  describe("minimizeWindow", () => {
    it("should minimize window and unfocus it", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      store.minimizeWindow(windowId);

      const window = store.windows.find((w) => w.id === windowId);
      expect(window?.isMinimized).toBe(true);
      expect(window?.isFocused).toBe(false);
    });
  });

  describe("restoreWindow", () => {
    it("should restore minimized window and focus it", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      store.minimizeWindow(windowId);
      store.restoreWindow(windowId);

      const window = store.windows.find((w) => w.id === windowId);
      expect(window?.isMinimized).toBe(false);
      expect(window?.isFocused).toBe(true);
    });
  });

  describe("maximizeWindow", () => {
    it("should maximize window and save previous bounds", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      const window = store.windows.find((w) => w.id === windowId);
      const originalBounds = {
        position: window!.position,
        size: window!.size,
      };

      store.maximizeWindow(windowId);

      const maximizedWindow = store.windows.find((w) => w.id === windowId);
      expect(maximizedWindow?.state).toBe(WinState.MAXIMIZED);
      expect(maximizedWindow?.metadata.lastNormalBounds).toEqual(originalBounds);
      // Maximized window should fill available space
      expect(maximizedWindow?.size.width).toBeGreaterThan(originalBounds.size.width);
      expect(maximizedWindow?.size.height).toBeGreaterThan(originalBounds.size.height);
    });
  });

  describe("unmaximizeWindow", () => {
    it("should restore window to previous bounds", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      const window = store.windows.find((w) => w.id === windowId);
      const originalBounds = {
        position: { ...window!.position },
        size: { ...window!.size },
      };

      store.maximizeWindow(windowId);
      store.unmaximizeWindow(windowId);

      const restoredWindow = store.windows.find((w) => w.id === windowId);
      expect(restoredWindow?.state).toBe(WinState.NORMAL);
      expect(restoredWindow?.position).toEqual(originalBounds.position);
      expect(restoredWindow?.size).toEqual(originalBounds.size);
    });
  });

  describe("toggleMaximize", () => {
    it("should toggle between maximized and normal state", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);

      store.toggleMaximize(windowId);
      expect(store.windows.find((w) => w.id === windowId)?.state).toBe(WinState.MAXIMIZED);

      store.toggleMaximize(windowId);
      expect(store.windows.find((w) => w.id === windowId)?.state).toBe(WinState.NORMAL);
    });
  });

  describe("focusWindow", () => {
    it("should focus window and increment zIndex", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const window1 = store.openWindow("app1", "App 1", mockBlueprint);
      const window2 = store.openWindow("app2", "App 2", mockBlueprint);

      const initialZIndex = store.windows.find((w) => w.id === window1)!.zIndex;
      store.focusWindow(window1);

      const focusedWindow = store.windows.find((w) => w.id === window1);
      expect(focusedWindow?.isFocused).toBe(true);
      expect(focusedWindow?.zIndex).toBeGreaterThan(initialZIndex);

      const otherWindow = store.windows.find((w) => w.id === window2);
      expect(otherWindow?.isFocused).toBe(false);
    });
  });

  describe("updateWindowPosition", () => {
    it("should update window position", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      const newPosition = { x: 200, y: 150 };

      store.updateWindowPosition(windowId, newPosition);

      const window = store.windows.find((w) => w.id === windowId);
      expect(window?.position).toEqual(newPosition);
    });
  });

  describe("updateWindowSize", () => {
    it("should update window size", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      const newSize = { width: 1000, height: 700 };

      store.updateWindowSize(windowId, newSize);

      const window = store.windows.find((w) => w.id === windowId);
      expect(window?.size).toEqual(newSize);
    });
  });

  describe("updateWindowBounds", () => {
    it("should update both position and size", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      const windowId = store.openWindow("test-app", "Test App", mockBlueprint);
      const newBounds = {
        position: { x: 250, y: 200 },
        size: { width: 900, height: 650 },
      };

      store.updateWindowBounds(windowId, newBounds);

      const window = store.windows.find((w) => w.id === windowId);
      expect(window?.position).toEqual(newBounds.position);
      expect(window?.size).toEqual(newBounds.size);
    });
  });

  describe("clearAllWindows", () => {
    it("should remove all windows", () => {
      const store = useWindowStore.getState();
      const mockBlueprint = { type: "app", title: "Test", components: [] };

      store.openWindow("app1", "App 1", mockBlueprint);
      store.openWindow("app2", "App 2", mockBlueprint);
      store.openWindow("app3", "App 3", mockBlueprint);

      store.clearAllWindows();

      expect(store.windows).toHaveLength(0);
    });
  });
});
