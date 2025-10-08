/**
 * useDefaults Hook
 * Initialize default desktop icons on first load
 */

import { useEffect } from "react";
import { useActions, useIcons } from "../store/store";
import { getDefaultIcons, shouldInitializeDefaults } from "../utils/defaults";

// ============================================================================
// Defaults Hook
// ============================================================================

/**
 * Initialize default desktop icons if desktop is empty
 * Only runs once on first mount
 */
export function useDefaults() {
  const icons = useIcons();
  const { add } = useActions();

  useEffect(() => {
    // Only initialize if desktop is empty
    if (shouldInitializeDefaults(icons)) {
      const defaults = getDefaultIcons();

      // Add each default icon
      for (const iconData of defaults) {
        add(iconData);
      }
    }
  }, []); // Empty dependency array - only run once on mount

  return null;
}

