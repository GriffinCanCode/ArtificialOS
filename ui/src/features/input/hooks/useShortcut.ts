/**
 * useShortcut Hook
 * React hook for registering a single keyboard shortcut
 */

import { useEffect, useRef } from "react";
import { useActions } from "../shortcuts/store/store";
import type { ShortcutConfig, ShortcutHandler } from "../shortcuts/core/types";

// ============================================================================
// Hook Interface
// ============================================================================

export interface UseShortcutOptions {
  /** Shortcut sequence (e.g., "$mod+k") */
  sequence: string;

  /** Handler function */
  handler: ShortcutHandler;

  /** Scope where shortcut is active */
  scope?: ShortcutConfig["scope"];

  /** Priority for conflict resolution */
  priority?: ShortcutConfig["priority"];

  /** Category for organization */
  category?: ShortcutConfig["category"];

  /** Whether shortcut works in input fields */
  allowInInput?: boolean;

  /** Whether shortcut is enabled */
  enabled?: boolean;

  /** Human-readable label */
  label?: string;

  /** Description */
  description?: string;

  /** Tags for searching */
  tags?: string[];
}

// ============================================================================
// Hook Implementation
// ============================================================================

/**
 * Register a keyboard shortcut that's automatically cleaned up on unmount
 */
export function useShortcut(
  id: string,
  options: UseShortcutOptions
): void {
  const { register, unregister } = useActions();
  const handlerRef = useRef(options.handler);

  // Keep handler ref updated
  useEffect(() => {
    handlerRef.current = options.handler;
  }, [options.handler]);

  // Register shortcut on mount
  useEffect(() => {
    const config: ShortcutConfig = {
      id,
      sequence: options.sequence,
      handler: (event, context) => handlerRef.current(event, context),
      label: options.label || id,
      description: options.description,
      scope: options.scope,
      priority: options.priority,
      category: options.category,
      allowInInput: options.allowInInput,
      enabled: options.enabled,
      tags: options.tags,
    };

    register(config);

    // Cleanup on unmount
    return () => {
      unregister(id);
    };
  }, [
    id,
    options.sequence,
    options.scope,
    options.priority,
    options.category,
    options.allowInInput,
    options.enabled,
    options.label,
    options.description,
    register,
    unregister,
  ]);
}

/**
 * Register a simple keyboard shortcut (convenience wrapper)
 */
export function useSimpleShortcut(
  sequence: string,
  handler: () => void,
  enabled: boolean = true
): void {
  useShortcut(`shortcut-${sequence}`, {
    sequence,
    handler: (e) => {
      e.preventDefault();
      handler();
    },
    enabled,
  });
}

/**
 * Register a global keyboard shortcut
 */
export function useGlobalShortcut(
  id: string,
  sequence: string,
  handler: ShortcutHandler,
  options?: Partial<UseShortcutOptions>
): void {
  useShortcut(id, {
    sequence,
    handler,
    scope: "global",
    ...options,
  });
}

