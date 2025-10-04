/**
 * Window Component
 * Draggable, resizable, and closeable window wrapper for apps
 */

import React, { useCallback } from "react";
import { Rnd } from "react-rnd";
import { X, Minus, Maximize2 } from "lucide-react";
import { useWindowActions, WindowState } from "../../store/windowStore";
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

  const handleDragStop = useCallback(
    (_e: any, d: { x: number; y: number }) => {
      updateWindowPosition(window.id, { x: d.x, y: d.y });
    },
    [window.id, updateWindowPosition]
  );

  const handleResizeStop = useCallback(
    (
      _e: any,
      _direction: any,
      ref: HTMLElement,
      _delta: any,
      position: { x: number; y: number }
    ) => {
      updateWindowSize(window.id, {
        width: ref.offsetWidth,
        height: ref.offsetHeight,
      });
      updateWindowPosition(window.id, position);
    },
    [window.id, updateWindowSize, updateWindowPosition]
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

