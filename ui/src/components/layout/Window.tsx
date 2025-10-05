/**
 * Window Component
 * Draggable, resizable, and closeable window wrapper for apps
 */

import React, { useCallback } from "react";
import { Rnd } from "react-rnd";
import { X, Minus, Maximize2 } from "lucide-react";
import { useWindowActions, WindowState } from "../../store/windowStore";
import { useLogger } from "../../utils/monitoring/useLogger";
import "./Window.css";

interface WindowProps {
  window: WindowState;
  children: React.ReactNode;
}

export const Window: React.FC<WindowProps> = ({ window, children }) => {
  const {
    closeWindow,
    minimizeWindow,
    focusWindow,
    updateWindowPosition,
    updateWindowSize,
  } = useWindowActions();
  const log = useLogger("Window");

  const handleDragStop = useCallback(
    (_e: any, d: { x: number; y: number }) => {
      updateWindowPosition(window.id, { x: d.x, y: d.y });
      
      // Sync to backend for session restoration (fire-and-forget)
      fetch(`http://localhost:8000/apps/${window.appId}/window`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          window_id: window.id,
          position: { x: d.x, y: d.y },
          size: window.size,
        }),
      }).catch((error) => {
        log.error("Failed to sync window position", error as Error);
      });
    },
    [window.id, window.appId, window.size, updateWindowPosition, log]
  );

  const handleResizeStop = useCallback(
    (
      _e: any,
      _direction: any,
      ref: HTMLElement,
      _delta: any,
      position: { x: number; y: number }
    ) => {
      const newSize = {
        width: ref.offsetWidth,
        height: ref.offsetHeight,
      };
      
      updateWindowSize(window.id, newSize);
      updateWindowPosition(window.id, position);
      
      // Sync to backend for session restoration (fire-and-forget)
      fetch(`http://localhost:8000/apps/${window.appId}/window`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          window_id: window.id,
          position: position,
          size: newSize,
        }),
      }).catch((error) => {
        log.error("Failed to sync window size", error as Error);
      });
    },
    [window.id, window.appId, updateWindowSize, updateWindowPosition, log]
  );

  const handleMouseDown = useCallback(() => {
    if (!window.isFocused) {
      focusWindow(window.id);
    }
  }, [window.isFocused, window.id, focusWindow]);

  const handleClose = useCallback(() => {
    closeWindow(window.id);
  }, [window.id, closeWindow]);

  const handleMinimize = useCallback(() => {
    minimizeWindow(window.id);
  }, [window.id, minimizeWindow]);

  return (
    <Rnd
      default={{
        x: window.position.x,
        y: window.position.y,
        width: window.size.width,
        height: window.size.height,
      }}
      minWidth={400}
      minHeight={300}
      bounds="parent"
      dragHandleClassName="window-titlebar"
      onDragStop={handleDragStop}
      onResizeStop={handleResizeStop}
      onMouseDown={handleMouseDown}
      style={{
        zIndex: window.zIndex,
      }}
      className={`window ${window.isFocused ? "focused" : ""}`}
    >
      <div className="window-container">
        {/* Window Titlebar */}
        <div className="window-titlebar">
          <div className="window-titlebar-left">
            {window.icon && <span className="window-icon">{window.icon}</span>}
            <span className="window-title">{window.title}</span>
          </div>
          <div className="window-titlebar-right">
            <button
              className="window-control-btn minimize"
              onClick={handleMinimize}
              title="Minimize"
            >
              <Minus size={14} />
            </button>
            <button
              className="window-control-btn maximize"
              title="Maximize (coming soon)"
            >
              <Maximize2 size={14} />
            </button>
            <button
              className="window-control-btn close"
              onClick={handleClose}
              title="Close"
            >
              <X size={14} />
            </button>
          </div>
        </div>

        {/* Window Content */}
        <div className="window-content">{children}</div>
      </div>
    </Rnd>
  );
};

