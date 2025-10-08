/**
 * Keyboard Input Core
 * Pure functions for keyboard event handling and detection
 */

import type { KeyboardEventData, KeyboardOptions, ModifierKey } from "./types";

/**
 * Extract structured data from keyboard event
 */
export function extractKeyboardData(event: KeyboardEvent): KeyboardEventData {
  const modifiers: ModifierKey[] = [];

  if (event.altKey) modifiers.push("Alt");
  if (event.ctrlKey) modifiers.push("Control");
  if (event.metaKey) modifiers.push("Meta");
  if (event.shiftKey) modifiers.push("Shift");

  return {
    key: event.key,
    code: event.code,
    modifiers,
    target: event.target,
  };
}

/**
 * Check if keyboard event target is an input element
 */
export function isTypingInInput(event: KeyboardEvent): boolean {
  const target = event.target as HTMLElement;
  return target.tagName === "INPUT" || target.tagName === "TEXTAREA";
}

/**
 * Check if keyboard event target is contenteditable
 */
export function isTypingInContentEditable(event: KeyboardEvent): boolean {
  const target = event.target as HTMLElement;
  return target.isContentEditable;
}

/**
 * Check if keyboard event should be ignored for shortcuts
 */
export function shouldIgnoreKeyboardEvent(
  event: KeyboardEvent,
  options?: KeyboardOptions
): boolean {
  if (options?.allowedKeys?.includes(event.key)) {
    return false;
  }

  if (isTypingInInput(event)) {
    return true;
  }

  if (options?.includeContentEditable && isTypingInContentEditable(event)) {
    return true;
  }

  return false;
}

/**
 * Check if event matches keyboard shortcut
 */
export function matchesShortcut(
  event: KeyboardEvent,
  key: string,
  modifiers?: ModifierKey[]
): boolean {
  if (event.key !== key) return false;

  const requiredModifiers = modifiers || [];
  const hasAlt = requiredModifiers.includes("Alt");
  const hasCtrl = requiredModifiers.includes("Control");
  const hasMeta = requiredModifiers.includes("Meta");
  const hasShift = requiredModifiers.includes("Shift");

  return (
    event.altKey === hasAlt &&
    event.ctrlKey === hasCtrl &&
    event.metaKey === hasMeta &&
    event.shiftKey === hasShift
  );
}

/**
 * Check if key is navigation key
 */
export function isNavigationKey(key: string): boolean {
  const navigationKeys = [
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Tab",
    "Home",
    "End",
    "PageUp",
    "PageDown",
  ];
  return navigationKeys.includes(key);
}

/**
 * Check if key is action key
 */
export function isActionKey(key: string): boolean {
  const actionKeys = ["Enter", "Escape", "Space", "Delete", "Backspace"];
  return actionKeys.includes(key);
}

