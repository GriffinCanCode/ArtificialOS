/**
 * useScope Hook
 * React hook for managing shortcut scopes
 */

import { useEffect } from "react";
import { useActions, useActiveScopes } from "../shortcuts/store/store";
import type { ShortcutScope } from "../shortcuts/core/types";

// ============================================================================
// Hook Implementation
// ============================================================================

/**
 * Activate a scope for the lifetime of the component
 */
export function useScope(scope: ShortcutScope, active: boolean = true): void {
  const { setScope } = useActions();

  useEffect(() => {
    // Activate scope on mount
    if (active) {
      setScope(scope, true);
    }

    // Deactivate scope on unmount
    return () => {
      if (active) {
        setScope(scope, false);
      }
    };
  }, [scope, active, setScope]);
}

/**
 * Activate multiple scopes
 */
export function useScopes(scopes: ShortcutScope[], active: boolean = true): void {
  const { setScope } = useActions();

  useEffect(() => {
    // Activate scopes on mount
    if (active) {
      for (const scope of scopes) {
        setScope(scope, true);
      }
    }

    // Deactivate scopes on unmount
    return () => {
      if (active) {
        for (const scope of scopes) {
          setScope(scope, false);
        }
      }
    };
  }, [scopes, active, setScope]);
}

/**
 * Check if a scope is active
 */
export function useScopeActive(scope: ShortcutScope): boolean {
  const activeScopes = useActiveScopes();
  return activeScopes.has(scope);
}

/**
 * Get all active scopes
 */
export function useActiveScopesArray(): ShortcutScope[] {
  const activeScopes = useActiveScopes();
  return Array.from(activeScopes);
}

/**
 * Toggle scope active state
 */
export function useToggleScope(scope: ShortcutScope): () => void {
  const { setScope } = useActions();
  const isActive = useScopeActive(scope);

  return () => {
    setScope(scope, !isActive);
  };
}

