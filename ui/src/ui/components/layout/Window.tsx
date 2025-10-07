/**
 * Window Component
 * Production-ready draggable, resizable window with animations and snap-to-edge
 */

import React, { useCallback, useRef, useState, useEffect } from "react";
import { Rnd } from "react-rnd";
import { X, Minus, Maximize2, Minimize2 } from "lucide-react";
import { useActions, useSnap, fadeIn, syncWindow, State } from "../../../features/windows";
import type { Window as WindowType } from "../../../features/windows";
import { useLogger } from "../../../core/utils/monitoring/useLogger";
import { Tooltip } from "../../../features/floating";
import "./Window.css";

interface WindowProps {
  window: WindowType;
  children: React.ReactNode;
}

export const Window: React.FC<WindowProps> = ({ window, children }) => {
  const {
    close,
    minimize,
    focus,
    toggle,
    updatePosition,
    updateSize,
    updateBounds,
    setAnimating,
    get,
  } = useActions();
  const log = useLogger("Window");
  const windowRef = useRef<Rnd>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const { preview, handleDrag, handleDragEnd, clearPreview } = useSnap();

  // Fade in animation on mount
  useEffect(() => {
    if (containerRef.current) {
      setAnimating(window.id, true);
      fadeIn(containerRef.current, { duration: 200 }).then(() => {
        setAnimating(window.id, false);
      });
    }
  }, []); // Only run once on mount

  const handleDragStart = useCallback(() => {
    setIsDragging(true);

    // If window is maximized, unmaximize it first
    if (window.state === State.MAXIMIZED) {
      toggle(window.id);
    }
  }, [window.id, window.state, toggle]);

  const handleDragMove = useCallback(
    (_e: any, d: { x: number; y: number }) => {
      // Show snap preview based on cursor position
      if (isDragging) {
        handleDrag(d.x, d.y);
      }
    },
    [isDragging, handleDrag]
  );

  const handleDragStop = useCallback(
    (_e: any, d: { x: number; y: number }) => {
      setIsDragging(false);

      // Check if we should snap to edge
      const snapBounds = handleDragEnd(d.x, d.y);

      if (snapBounds) {
        // Apply snap position
        updateBounds(window.id, snapBounds);
        log.debug("Window snapped to edge", { windowId: window.id, snapBounds });
      } else {
        // Normal position update
        updatePosition(window.id, { x: d.x, y: d.y });
      }

      clearPreview();

      // Sync to backend for session restoration - only if window still exists
      const currentWindow = get(window.id);
      if (currentWindow) {
        syncWindow(
          window.appId,
          window.id,
          snapBounds?.position || { x: d.x, y: d.y },
          snapBounds?.size || window.size
        );
      }
    },
    [
      window.id,
      window.appId,
      window.size,
      handleDragEnd,
      updatePosition,
      updateBounds,
      clearPreview,
      get,
      log,
    ]
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

      updateSize(window.id, newSize);
      updatePosition(window.id, position);

      // Sync to backend for session restoration - only if window still exists
      const currentWindow = get(window.id);
      if (currentWindow) {
        syncWindow(window.appId, window.id, position, newSize);
      }
    },
    [window.id, window.appId, updateSize, updatePosition, get]
  );

  const handleMouseDown = useCallback(() => {
    if (!window.isFocused) {
      focus(window.id);
    }
  }, [window.isFocused, window.id, focus]);

  const handleClose = useCallback(() => {
    close(window.id);
  }, [window.id, close]);

  const handleMinimize = useCallback(() => {
    minimize(window.id);
  }, [window.id, minimize]);

  const handleMaximize = useCallback(() => {
    toggle(window.id);
  }, [window.id, toggle]);

  // Double-click titlebar to maximize/restore
  const handleTitlebarDoubleClick = useCallback(() => {
    toggle(window.id);
  }, [window.id, toggle]);

  const isMaximized = window.state === State.MAXIMIZED;

  return (
    <>
      <Rnd
        ref={windowRef}
        default={{
          x: window.position.x,
          y: window.position.y,
          width: window.size.width,
          height: window.size.height,
        }}
        position={{
          x: window.position.x,
          y: window.position.y,
        }}
        size={{
          width: window.size.width,
          height: window.size.height,
        }}
        minWidth={400}
        minHeight={300}
        bounds="parent"
        dragHandleClassName="window-titlebar"
        onDragStart={handleDragStart}
        onDrag={handleDragMove}
        onDragStop={handleDragStop}
        onResizeStop={handleResizeStop}
        onMouseDown={handleMouseDown}
        disableDragging={isMaximized}
        enableResizing={!isMaximized}
        style={{
          zIndex: window.zIndex,
        }}
        className={`window ${window.isFocused ? "focused" : ""} ${isMaximized ? "maximized" : ""}`}
      >
        <div ref={containerRef} className="window-container">
          {/* Window Titlebar */}
          <div className="window-titlebar" onDoubleClick={handleTitlebarDoubleClick}>
            <div className="window-titlebar-left">
              {window.icon && <span className="window-icon">{window.icon}</span>}
              <span className="window-title">{window.title}</span>
            </div>
            <div className="window-titlebar-right">
              <Tooltip content="Minimize (⌘M)" delay={700}>
                <button
                  className="window-control-btn minimize"
                  onClick={handleMinimize}
                  aria-label="Minimize"
                >
                  <Minus size={14} />
                </button>
              </Tooltip>
              <Tooltip content={isMaximized ? "Restore" : "Maximize"} delay={700}>
                <button
                  className="window-control-btn maximize"
                  onClick={handleMaximize}
                  aria-label={isMaximized ? "Restore" : "Maximize"}
                >
                  {isMaximized ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
                </button>
              </Tooltip>
              <Tooltip content="Close (⌘W)" delay={700}>
                <button
                  className="window-control-btn close"
                  onClick={handleClose}
                  aria-label="Close"
                >
                  <X size={14} />
                </button>
              </Tooltip>
            </div>
          </div>

          {/* Window Content */}
          <div className="window-content">{children}</div>
        </div>
      </Rnd>

      {/* Snap Preview Overlay */}
      {preview && (
        <div
          className="snap-preview"
          style={{
            position: "fixed",
            left: `${preview.bounds.position.x}px`,
            top: `${preview.bounds.position.y}px`,
            width: `${preview.bounds.size.width}px`,
            height: `${preview.bounds.size.height}px`,
            pointerEvents: "none",
            zIndex: 9999,
            background: "rgba(139, 92, 246, 0.15)",
            border: "2px solid rgba(139, 92, 246, 0.5)",
            borderRadius: "8px",
          }}
        />
      )}
    </>
  );
};
