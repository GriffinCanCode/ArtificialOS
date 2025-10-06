/**
 * Keyboard Input Hook
 * React hook for keyboard event handling with shortcuts
 */

import { useEffect, useRef } from "react";
import type { ShortcutConfig, KeyboardOptions } from "../core/types";
import { shouldIgnoreKeyboardEvent, matchesShortcut } from "../core/keyboard";

/**
 * Hook for handling keyboard shortcuts
 */
export function useKeyboard(shortcuts: ShortcutConfig[], options?: KeyboardOptions) {
  const shortcutsRef = useRef(shortcuts);

  useEffect(() => {
    shortcutsRef.current = shortcuts;
  }, [shortcuts]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Check if event should be ignored
      if (!shortcutsRef.current.some((s) => s.allowInInput)) {
        if (shouldIgnoreKeyboardEvent(event, options)) {
          return;
        }
      }

      // Find matching shortcut
      for (const shortcut of shortcutsRef.current) {
        if (matchesShortcut(event, shortcut.key, shortcut.modifiers)) {
          if (!shortcut.allowInInput && shouldIgnoreKeyboardEvent(event, options)) {
            continue;
          }

          if (options?.preventDefault ?? true) {
            event.preventDefault();
          }

          if (options?.stopPropagation) {
            event.stopPropagation();
          }

          shortcut.handler(event);
          break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [options]);
}

/**
 * Hook for single keyboard shortcut
 */
export function useKeyboardShortcut(
  key: string,
  handler: (event: KeyboardEvent) => void,
  modifiers?: string[],
  options?: KeyboardOptions
) {
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  const shortcuts: ShortcutConfig[] = [
    {
      key,
      modifiers: modifiers as any,
      handler: (e) => handlerRef.current(e),
    },
  ];

  useKeyboard(shortcuts, options);
}

/**
 * Hook for escape key handling
 */
export function useEscapeKey(handler: () => void) {
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useKeyboardShortcut("Escape", () => handlerRef.current(), undefined, {
    allowedKeys: ["Escape"],
  });
}

/**
 * Hook for Enter key handling
 */
export function useEnterKey(handler: () => void, options?: KeyboardOptions) {
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useKeyboardShortcut("Enter", () => handlerRef.current(), undefined, options);
}

/**
 * Hook for arrow key navigation
 */
export function useArrowKeys(handlers: {
  onUp?: () => void;
  onDown?: () => void;
  onLeft?: () => void;
  onRight?: () => void;
}) {
  const handlersRef = useRef(handlers);

  useEffect(() => {
    handlersRef.current = handlers;
  }, [handlers]);

  const shortcuts: ShortcutConfig[] = [
    ...(handlers.onUp ? [{ key: "ArrowUp", handler: () => handlersRef.current.onUp?.() }] : []),
    ...(handlers.onDown
      ? [{ key: "ArrowDown", handler: () => handlersRef.current.onDown?.() }]
      : []),
    ...(handlers.onLeft
      ? [{ key: "ArrowLeft", handler: () => handlersRef.current.onLeft?.() }]
      : []),
    ...(handlers.onRight
      ? [{ key: "ArrowRight", handler: () => handlersRef.current.onRight?.() }]
      : []),
  ];

  useKeyboard(shortcuts as ShortcutConfig[]);
}
