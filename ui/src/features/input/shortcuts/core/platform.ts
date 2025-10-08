/**
 * Platform Detection and Utilities
 * Platform-specific keyboard shortcut handling
 */

import type { Platform, Modifier } from "./types";

// ============================================================================
// Platform Detection
// ============================================================================

/**
 * Detect current platform
 */
export function detectPlatform(): Platform {
  if (typeof window === "undefined") {
    return "unknown";
  }

  const ua = window.navigator.userAgent.toLowerCase();
  const platform = window.navigator.platform.toLowerCase();

  if (platform.includes("mac") || ua.includes("mac")) {
    return "mac";
  }

  if (platform.includes("win") || ua.includes("win")) {
    return "windows";
  }

  if (platform.includes("linux") || ua.includes("linux")) {
    return "linux";
  }

  return "unknown";
}

/**
 * Check if running on Mac
 */
export function isMac(): boolean {
  return detectPlatform() === "mac";
}

/**
 * Check if running on Windows
 */
export function isWindows(): boolean {
  return detectPlatform() === "windows";
}

/**
 * Check if running on Linux
 */
export function isLinux(): boolean {
  return detectPlatform() === "linux";
}

// ============================================================================
// Modifier Key Mapping
// ============================================================================

/**
 * Map platform-specific modifier symbols
 */
export const MODIFIER_SYMBOLS: Record<Platform, Record<string, string>> = {
  mac: {
    $mod: "⌘",
    Meta: "⌘",
    Control: "⌃",
    Alt: "⌥",
    Shift: "⇧",
  },
  windows: {
    $mod: "Ctrl",
    Meta: "Win",
    Control: "Ctrl",
    Alt: "Alt",
    Shift: "Shift",
  },
  linux: {
    $mod: "Ctrl",
    Meta: "Super",
    Control: "Ctrl",
    Alt: "Alt",
    Shift: "Shift",
  },
  unknown: {
    $mod: "Mod",
    Meta: "Meta",
    Control: "Ctrl",
    Alt: "Alt",
    Shift: "Shift",
  },
};

/**
 * Map platform-specific modifier names
 */
export const MODIFIER_NAMES: Record<Platform, Record<string, string>> = {
  mac: {
    $mod: "Command",
    Meta: "Command",
    Control: "Control",
    Alt: "Option",
    Shift: "Shift",
  },
  windows: {
    $mod: "Control",
    Meta: "Windows",
    Control: "Control",
    Alt: "Alt",
    Shift: "Shift",
  },
  linux: {
    $mod: "Control",
    Meta: "Super",
    Control: "Control",
    Alt: "Alt",
    Shift: "Shift",
  },
  unknown: {
    $mod: "Mod",
    Meta: "Meta",
    Control: "Control",
    Alt: "Alt",
    Shift: "Shift",
  },
};

/**
 * Get platform-specific modifier key
 */
export function getPlatformModifier(platform?: Platform): Modifier {
  const p = platform || detectPlatform();
  return p === "mac" ? "Meta" : "Control";
}

/**
 * Resolve $mod to platform-specific modifier
 */
export function resolveMod(modifier: string, platform?: Platform): string {
  if (modifier === "$mod") {
    return getPlatformModifier(platform);
  }
  return modifier;
}

// ============================================================================
// Key Mapping
// ============================================================================

/**
 * Map platform-specific key names
 */
export const KEY_SYMBOLS: Record<string, string> = {
  Enter: "↵",
  Return: "↵",
  Escape: "⎋",
  Backspace: "⌫",
  Delete: "⌦",
  Tab: "⇥",
  Space: "␣",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  PageUp: "⇞",
  PageDown: "⇟",
  Home: "↖",
  End: "↘",
};

/**
 * Normalize key name
 */
export function normalizeKey(key: string): string {
  // Map common variations
  const keyMap: Record<string, string> = {
    cmd: "Meta",
    command: "Meta",
    ctrl: "Control",
    control: "Control",
    opt: "Alt",
    option: "Alt",
    alt: "Alt",
    shift: "Shift",
    meta: "Meta",
    win: "Meta",
    windows: "Meta",
    super: "Meta",
    return: "Enter",
    esc: "Escape",
    backspace: "Backspace",
    delete: "Delete",
    del: "Delete",
    tab: "Tab",
    space: "Space",
  };

  const normalized = key.toLowerCase();
  return keyMap[normalized] || key;
}

// ============================================================================
// Compatibility
// ============================================================================

/**
 * Convert shortcut sequence to tinykeys format
 */
export function toTinykeysFormat(sequence: string, platform?: Platform): string {
  const p = platform || detectPlatform();

  // Replace $mod with platform-specific modifier
  let formatted = sequence.replace(/\$mod/gi, p === "mac" ? "Meta" : "Control");

  // Normalize modifier casing
  formatted = formatted
    .replace(/cmd\+/gi, "Meta+")
    .replace(/ctrl\+/gi, "Control+")
    .replace(/alt\+/gi, "Alt+")
    .replace(/shift\+/gi, "Shift+");

  return formatted;
}

/**
 * Check if sequence is platform-compatible
 */
export function isPlatformCompatible(
  sequence: string,
  platform?: Platform
): boolean {
  const p = platform || detectPlatform();

  // Check for platform-specific keys that don't exist on current platform
  if (p !== "mac" && sequence.includes("Command")) {
    return false;
  }

  if (p === "mac" && sequence.includes("Windows")) {
    return false;
  }

  return true;
}

