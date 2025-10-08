/**
 * Default Desktop Icons
 * Base system apps that appear on the desktop
 */

import type { Icon } from "../core/types";

// ============================================================================
// Default Icons
// ============================================================================

/**
 * Get default desktop icons for system apps
 */
export function getDefaultIcons(): Omit<Icon, "id" | "isSelected" | "isDragging" | "isHovered" | "zIndex" | "createdAt" | "updatedAt">[] {
  return [
    {
      type: "native",
      label: "Terminal",
      icon: "💻",
      position: { row: 0, col: 0 },
      metadata: {
        type: "native",
        packageId: "terminal",
        bundlePath: "/apps/native/terminal",
      },
    },
    {
      type: "native",
      label: "Files",
      icon: "📁",
      position: { row: 0, col: 1 },
      metadata: {
        type: "native",
        packageId: "file-explorer",
        bundlePath: "/apps/native/file-explorer",
      },
    },
  ];
}

/**
 * Check if default icons need to be initialized
 * Returns true if the desktop is empty
 */
export function shouldInitializeDefaults(existingIcons: Icon[]): boolean {
  return existingIcons.length === 0;
}

/**
 * Get default icon positions that are already occupied
 */
export function getOccupiedDefaultPositions(existingIcons: Icon[]): Set<string> {
  const defaults = getDefaultIcons();
  const defaultPositions = new Set(
    defaults.map((icon) => `${icon.position.row}:${icon.position.col}`)
  );

  const occupied = new Set<string>();
  for (const icon of existingIcons) {
    const key = `${icon.position.row}:${icon.position.col}`;
    if (defaultPositions.has(key)) {
      occupied.add(key);
    }
  }

  return occupied;
}

