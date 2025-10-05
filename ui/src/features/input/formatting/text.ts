/**
 * Text Formatting Utilities
 * Pure functions for text transformation and formatting
 */

// ============================================================================
// Case Transformation
// ============================================================================

export function toTitleCase(text: string): string {
  return text
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function toCamelCase(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, char) => char.toUpperCase());
}

export function toKebabCase(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-zA-Z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function toSnakeCase(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-zA-Z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

export function toPascalCase(text: string): string {
  const camel = toCamelCase(text);
  return camel.charAt(0).toUpperCase() + camel.slice(1);
}

// ============================================================================
// String Manipulation
// ============================================================================

export function truncate(text: string, maxLength: number, ellipsis: string = "..."): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - ellipsis.length) + ellipsis;
}

export function truncateMiddle(text: string, maxLength: number, separator: string = "..."): string {
  if (text.length <= maxLength) return text;
  const charsToShow = maxLength - separator.length;
  const frontChars = Math.ceil(charsToShow / 2);
  const backChars = Math.floor(charsToShow / 2);
  return text.slice(0, frontChars) + separator + text.slice(-backChars);
}

export function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1).toLowerCase();
}

export function uncapitalize(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}

// ============================================================================
// Whitespace Management
// ============================================================================

export function normalizeWhitespace(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

export function removeWhitespace(text: string): string {
  return text.replace(/\s/g, "");
}

export function indentLines(text: string, spaces: number): string {
  const indent = " ".repeat(spaces);
  return text
    .split("\n")
    .map((line) => indent + line)
    .join("\n");
}

// ============================================================================
// Sanitization
// ============================================================================

export function sanitizeInput(text: string): string {
  return text
    .replace(/[<>]/g, "")
    .trim();
}

export function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  };
  return text.replace(/[&<>"']/g, (char) => map[char]);
}

export function unescapeHtml(text: string): string {
  const map: Record<string, string> = {
    "&amp;": "&",
    "&lt;": "<",
    "&gt;": ">",
    "&quot;": '"',
    "&#39;": "'",
  };
  return text.replace(/&amp;|&lt;|&gt;|&quot;|&#39;/g, (entity) => map[entity]);
}

// ============================================================================
// Word Operations
// ============================================================================

export function wordCount(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

export function extractWords(text: string): string[] {
  return text.match(/\b\w+\b/g) || [];
}

export function pluralize(word: string, count: number, suffix: string = "s"): string {
  return count === 1 ? word : word + suffix;
}

// ============================================================================
// Path & Slug
// ============================================================================

export function toSlug(text: string): string {
  return text
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function pathJoin(...parts: string[]): string {
  return parts
    .map((part, index) => {
      if (index === 0) return part.replace(/\/+$/, "");
      if (index === parts.length - 1) return part.replace(/^\/+/, "");
      return part.replace(/^\/+|\/+$/g, "");
    })
    .filter(Boolean)
    .join("/");
}
