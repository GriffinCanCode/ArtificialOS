/**
 * Local ID Generation for Terminal App
 * Simple, self-contained ID generation for isolated binary
 */

/**
 * Generate a prefixed ID with timestamp and random component
 */
export function generatePrefixed(prefix: string): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 9);
  return `${prefix}_${timestamp}_${random}`;
}

/**
 * Generate tab ID specifically
 */
export function generateTabId(): string {
  return generatePrefixed("tab");
}

/**
 * Generate session ID specifically (for local UI use, not backend session)
 */
export function generateLocalSessionId(): string {
  return generatePrefixed("session");
}
