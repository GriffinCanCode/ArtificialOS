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
  const { add, clearAll } = useActions();

  useEffect(() => {
    // Check for duplicate native icons (corrupted localStorage)
    const nativeIcons = icons.filter((icon) => icon.type === "native");
    const terminalIcons = nativeIcons.filter(
      (icon) => icon.metadata.type === "native" && icon.metadata.packageId === "terminal"
    );
    const fileIcons = nativeIcons.filter(
      (icon) => icon.metadata.type === "native" && icon.metadata.packageId === "file-explorer"
    );

    // If we have duplicates, clear all and re-initialize
    if (terminalIcons.length > 1 || fileIcons.length > 1) {
      console.log("Detected duplicate icons, clearing and re-initializing");
      clearAll();
      const defaults = getDefaultIcons();
      for (const iconData of defaults) {
        add(iconData);
      }
      return;
    }

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

