/**
 * Shortcut Formatter
 * Format keyboard shortcuts for display
 */

import { detectPlatform, MODIFIER_SYMBOLS, MODIFIER_NAMES, KEY_SYMBOLS } from "./platform";
import { parseSequence } from "./parser";
import type { Platform, FormattedShortcut } from "./types";

// ============================================================================
// Formatting Functions
// ============================================================================

/**
 * Format shortcut for display
 */
export function formatShortcut(
  sequence: string,
  platform?: Platform
): FormattedShortcut {
  const p = platform || detectPlatform();
  const parsed = parseSequence(sequence, p);

  // Get symbol representations
  const modSymbols = parsed.modifiers.map(
    (mod) => MODIFIER_SYMBOLS[p][mod] || mod
  );
  const keySymbol = KEY_SYMBOLS[parsed.key] || parsed.key;

  // Get verbose names
  const modNames = parsed.modifiers.map(
    (mod) => MODIFIER_NAMES[p][mod] || mod
  );
  const keyName = parsed.key.charAt(0).toUpperCase() + parsed.key.slice(1);

  // Build display strings
  const symbols = p === "mac"
    ? [...modSymbols, keySymbol].join("")
    : [...modSymbols, keySymbol].join("+");

  const verbose = [...modNames, keyName].join(p === "mac" ? " " : "+");

  // Build key array for rendering
  const keys = [...parsed.modifiers, parsed.key];

  // Platform-specific display
  const display = p === "mac" ? symbols : verbose;

  return {
    sequence: parsed.original,
    display,
    keys,
    symbols,
    verbose,
  };
}

/**
 * Format multiple shortcuts
 */
export function formatShortcuts(
  sequences: string[],
  platform?: Platform
): FormattedShortcut[] {
  return sequences.map((seq) => formatShortcut(seq, platform));
}

/**
 * Format shortcut for HTML rendering
 */
export function formatShortcutHTML(
  sequence: string,
  platform?: Platform
): string {
  const formatted = formatShortcut(sequence, platform);

  const keys = formatted.keys.map((key) => {
    const display = MODIFIER_SYMBOLS[platform || detectPlatform()][key] || KEY_SYMBOLS[key] || key;
    return `<kbd class="shortcut-key">${display}</kbd>`;
  });

  const separator = platform === "mac" ? "" : '<span class="shortcut-separator">+</span>';

  return keys.join(separator);
}

/**
 * Format shortcut for accessibility
 */
export function formatShortcutAria(
  sequence: string,
  platform?: Platform
): string {
  const formatted = formatShortcut(sequence, platform);
  return formatted.verbose;
}

// ============================================================================
// Search and Filtering
// ============================================================================

/**
 * Create searchable text from shortcut
 */
export function createSearchText(sequence: string, platform?: Platform): string {
  const formatted = formatShortcut(sequence, platform);
  return [
    formatted.sequence,
    formatted.symbols,
    formatted.verbose,
    ...formatted.keys,
  ].join(" ").toLowerCase();
}

/**
 * Check if shortcut matches search query
 */
export function matchesSearch(
  sequence: string,
  query: string,
  platform?: Platform
): boolean {
  if (!query.trim()) return true;

  const searchText = createSearchText(sequence, platform);
  const queryLower = query.toLowerCase();

  return searchText.includes(queryLower);
}

// ============================================================================
// Display Utilities
// ============================================================================

/**
 * Get CSS class for shortcut key
 */
export function getKeyClass(key: string): string {
  const classes = ["shortcut-key"];

  if (["Control", "Meta", "Alt", "Shift"].includes(key)) {
    classes.push("shortcut-key--modifier");
  }

  if (KEY_SYMBOLS[key]) {
    classes.push("shortcut-key--symbol");
  }

  return classes.join(" ");
}

/**
 * Get display size category
 */
export function getDisplaySize(sequence: string): "small" | "medium" | "large" {
  const parsed = parseSequence(sequence);
  const totalKeys = parsed.modifiers.length + 1;

  if (totalKeys <= 2) return "small";
  if (totalKeys <= 3) return "medium";
  return "large";
}

/**
 * Truncate shortcut for compact display
 */
export function truncateShortcut(
  sequence: string,
  maxLength: number = 15,
  platform?: Platform
): string {
  const formatted = formatShortcut(sequence, platform);

  if (formatted.display.length <= maxLength) {
    return formatted.display;
  }

  // Try verbose first
  if (formatted.verbose.length <= maxLength) {
    return formatted.verbose;
  }

  // Truncate and add ellipsis
  return formatted.display.substring(0, maxLength - 1) + "…";
}

