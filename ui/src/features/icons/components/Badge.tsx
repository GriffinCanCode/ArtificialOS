/**
 * Icon Badge Component
 * Displays notifications, status, and counts on icons
 */

import React from "react";
import type { IconBadge } from "../core/types";
import { getBadgeColor, getBadgePosition, formatBadgeContent, shouldShowBadge } from "../utils/badges";
import "./Badge.css";

// ============================================================================
// Badge Component Props
// ============================================================================

export interface BadgeProps {
  badge: IconBadge;
}

// ============================================================================
// Badge Component
// ============================================================================

export const Badge: React.FC<BadgeProps> = React.memo(({ badge }) => {
  if (!shouldShowBadge(badge)) {
    return null;
  }

  const position = getBadgePosition(badge.position || "top-right");
  const color = badge.color || getBadgeColor(badge.type);
  const content = formatBadgeContent(badge.content);

  return (
    <div
      className={`icon-badge icon-badge-${badge.type}`}
      style={{
        ...position,
        backgroundColor: color,
      }}
      title={badge.tooltip}
    >
      {content}
    </div>
  );
});

Badge.displayName = "IconBadge";

