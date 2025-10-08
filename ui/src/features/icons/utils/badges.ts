/**
 * Badge System
 * Utilities for icon badges (notifications, status, counts)
 */

import type { IconBadge, BadgeType, BadgePosition } from "../core/types";

// ============================================================================
// Badge Factory Functions
// ============================================================================

/**
 * Create notification badge
 */
export function notificationBadge(count: number, position?: BadgePosition): IconBadge {
  return {
    type: "notification",
    content: count > 99 ? "99+" : count,
    position: position || "top-right",
    tooltip: `${count} notification${count !== 1 ? "s" : ""}`,
  };
}

/**
 * Create status badge
 */
export function statusBadge(status: string, color?: string, position?: BadgePosition): IconBadge {
  return {
    type: "status",
    content: status,
    position: position || "bottom-right",
    color,
    tooltip: `Status: ${status}`,
  };
}

/**
 * Create count badge
 */
export function countBadge(count: number, label?: string, position?: BadgePosition): IconBadge {
  return {
    type: "count",
    content: count,
    position: position || "top-right",
    tooltip: label ? `${count} ${label}` : `Count: ${count}`,
  };
}

/**
 * Create alert badge
 */
export function alertBadge(message?: string, position?: BadgePosition): IconBadge {
  return {
    type: "alert",
    content: "!",
    position: position || "top-right",
    tooltip: message || "Alert",
    color: "#ef4444",
  };
}

/**
 * Create success badge
 */
export function successBadge(message?: string, position?: BadgePosition): IconBadge {
  return {
    type: "success",
    content: "✓",
    position: position || "bottom-right",
    tooltip: message || "Success",
    color: "#10b981",
  };
}

/**
 * Create error badge
 */
export function errorBadge(message?: string, position?: BadgePosition): IconBadge {
  return {
    type: "error",
    content: "✕",
    position: position || "top-right",
    tooltip: message || "Error",
    color: "#ef4444",
  };
}

// ============================================================================
// Badge Color Mapping
// ============================================================================

/**
 * Get default color for badge type
 */
export function getBadgeColor(type: BadgeType): string {
  const colors: Record<BadgeType, string> = {
    notification: "#3b82f6",
    status: "#8b5cf6",
    count: "#3b82f6",
    alert: "#ef4444",
    success: "#10b981",
    error: "#ef4444",
  };

  return colors[type];
}

// ============================================================================
// Badge Position Utilities
// ============================================================================

/**
 * Get CSS position for badge
 */
export function getBadgePosition(position: BadgePosition): { top?: string; right?: string; bottom?: string; left?: string } {
  const positions: Record<BadgePosition, any> = {
    "top-right": { top: "-4px", right: "-4px" },
    "top-left": { top: "-4px", left: "-4px" },
    "bottom-right": { bottom: "-4px", right: "-4px" },
    "bottom-left": { bottom: "-4px", left: "-4px" },
  };

  return positions[position];
}

// ============================================================================
// Badge Validation
// ============================================================================

/**
 * Check if badge should be displayed
 */
export function shouldShowBadge(badge?: IconBadge): boolean {
  if (!badge) return false;
  if (badge.type === "count" && typeof badge.content === "number" && badge.content <= 0) {
    return false;
  }
  return true;
}

/**
 * Format badge content for display
 */
export function formatBadgeContent(content: string | number | undefined): string {
  if (content === undefined) return "";
  if (typeof content === "number") {
    return content > 999 ? "999+" : content.toString();
  }
  return content.toString();
}

