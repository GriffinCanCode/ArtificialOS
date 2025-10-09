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
  const { add, clearAll, updatePositions, update } = useActions();

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

    // Check for overlapping icons at the same position
    const positionMap = new Map<string, string[]>();
    icons.forEach((icon) => {
      const key = `${icon.position.row}:${icon.position.col}`;
      const existing = positionMap.get(key) || [];
      existing.push(icon.id);
      positionMap.set(key, existing);
    });

    // Find positions with multiple icons
    const overlaps = Array.from(positionMap.entries()).filter(([_, ids]) => ids.length > 1);

    if (overlaps.length > 0) {
      console.warn("Detected overlapping icons, fixing positions:", overlaps);

      // Auto-fix: spread out overlapping icons
      const fixes = new Map<string, { row: number; col: number }>();

      overlaps.forEach(([posKey, iconIds]) => {
        // Keep first icon in place, move others to nearby positions
        iconIds.slice(1).forEach((iconId, index) => {
          const icon = icons.find((i) => i.id === iconId);
          if (icon) {
            // Try adjacent positions
            fixes.set(iconId, {
              row: icon.position.row,
              col: icon.position.col + index + 1,
            });
          }
        });
      });

      if (fixes.size > 0) {
        updatePositions(fixes);
      }
      return;
    }

    // Initialize if desktop is empty
    if (shouldInitializeDefaults(icons)) {
      const defaults = getDefaultIcons();

      // Add each default icon
      for (const iconData of defaults) {
        add(iconData);
      }
    } else {
      // Update existing native app icons if their definitions have changed
      const defaults = getDefaultIcons();

      defaults.forEach((defaultIcon) => {
        // Find existing icon with same packageId (only for native metadata)
        const existingIcon = icons.find(
          (icon) =>
            icon.type === "native" &&
            icon.metadata.type === "native" &&
            "packageId" in icon.metadata &&
            "packageId" in defaultIcon.metadata &&
            icon.metadata.packageId === defaultIcon.metadata.packageId
        );

        if (existingIcon && existingIcon.icon !== defaultIcon.icon) {
          const packageId = "packageId" in defaultIcon.metadata ? defaultIcon.metadata.packageId : "unknown";
          console.log(`Updating icon for ${packageId}: ${existingIcon.icon} -> ${defaultIcon.icon}`);
          update(existingIcon.id, { icon: defaultIcon.icon });
        }
      });
    }
  }, []); // Empty dependency array - only run once on mount

  return null;
}

