/**
 * Search Highlighting
 * Utilities for highlighting search matches in text
 */

import type { HighlightedText, HighlightStyle, TextSegment } from "../core/types";

// ============================================================================
// Highlighting
// ============================================================================

/**
 * Default highlight style
 */
const DEFAULT_STYLE: Required<HighlightStyle> = {
  tag: "mark",
  className: "search-highlight",
  style: {},
  prefix: "",
  suffix: "",
};

/**
 * Highlight matches in text
 */
export function highlight(
  text: string,
  indices: Array<[number, number]>,
  style: HighlightStyle = {}
): HighlightedText {
  const mergedStyle = { ...DEFAULT_STYLE, ...style };

  if (indices.length === 0) {
    return {
      original: text,
      html: text,
      segments: [{ text, isMatch: false, start: 0, end: text.length }],
      matchCount: 0,
    };
  }

  // Sort and merge overlapping indices
  const sorted = mergeIndices(indices);
  const segments: TextSegment[] = [];

  let html = "";
  let lastIndex = 0;

  for (const [start, end] of sorted) {
    // Add unmatched text before this match
    if (lastIndex < start) {
      const unmatched = text.slice(lastIndex, start);
      html += unmatched;
      segments.push({
        text: unmatched,
        isMatch: false,
        start: lastIndex,
        end: start,
      });
    }

    // Add matched text with highlighting
    const matched = text.slice(start, end + 1);
    const styleStr = Object.entries(mergedStyle.style)
      .map(([k, v]) => `${k}: ${v}`)
      .join("; ");

    html += mergedStyle.prefix;
    html += `<${mergedStyle.tag} class="${mergedStyle.className}"${styleStr ? ` style="${styleStr}"` : ""}>`;
    html += matched;
    html += `</${mergedStyle.tag}>`;
    html += mergedStyle.suffix;

    segments.push({
      text: matched,
      isMatch: true,
      start,
      end: end + 1,
    });

    lastIndex = end + 1;
  }

  // Add remaining unmatched text
  if (lastIndex < text.length) {
    const remaining = text.slice(lastIndex);
    html += remaining;
    segments.push({
      text: remaining,
      isMatch: false,
      start: lastIndex,
      end: text.length,
    });
  }

  return {
    original: text,
    html,
    segments,
    matchCount: sorted.length,
  };
}

/**
 * Merge overlapping indices
 */
function mergeIndices(indices: Array<[number, number]>): Array<[number, number]> {
  if (indices.length === 0) return [];

  const sorted = [...indices].sort((a, b) => a[0] - b[0]);
  const merged: Array<[number, number]> = [sorted[0]];

  for (let i = 1; i < sorted.length; i++) {
    const current = sorted[i];
    const last = merged[merged.length - 1];

    if (current[0] <= last[1] + 1) {
      // Overlapping or adjacent - merge
      last[1] = Math.max(last[1], current[1]);
    } else {
      // Non-overlapping - add new
      merged.push(current);
    }
  }

  return merged;
}

/**
 * Highlight query in text (simple substring match)
 */
export function highlightQuery(query: string, text: string, style?: HighlightStyle): HighlightedText {
  if (!query.trim()) {
    return {
      original: text,
      html: text,
      segments: [{ text, isMatch: false, start: 0, end: text.length }],
      matchCount: 0,
    };
  }

  const indices: Array<[number, number]> = [];
  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();

  let index = 0;
  while (index < text.length) {
    const matchIndex = lowerText.indexOf(lowerQuery, index);
    if (matchIndex === -1) break;

    indices.push([matchIndex, matchIndex + query.length - 1]);
    index = matchIndex + query.length;
  }

  return highlight(text, indices, style);
}

/**
 * Extract text excerpt around first match
 */
export function excerpt(
  text: string,
  indices: Array<[number, number]>,
  maxLength: number = 150,
  ellipsis: string = "..."
): string {
  if (indices.length === 0 || text.length <= maxLength) {
    return text.slice(0, maxLength);
  }

  const firstMatch = indices[0];
  const matchStart = firstMatch[0];
  const matchEnd = firstMatch[1];

  const before = Math.floor((maxLength - (matchEnd - matchStart + 1)) / 2);
  const after = maxLength - before - (matchEnd - matchStart + 1);

  let start = Math.max(0, matchStart - before);
  let end = Math.min(text.length, matchEnd + 1 + after);

  // Try to break at word boundaries
  if (start > 0) {
    const spaceIndex = text.indexOf(" ", start);
    if (spaceIndex !== -1 && spaceIndex < matchStart) {
      start = spaceIndex + 1;
    }
  }

  if (end < text.length) {
    const spaceIndex = text.lastIndexOf(" ", end);
    if (spaceIndex !== -1 && spaceIndex > matchEnd) {
      end = spaceIndex;
    }
  }

  let result = text.slice(start, end);

  if (start > 0) result = ellipsis + result;
  if (end < text.length) result = result + ellipsis;

  return result;
}

