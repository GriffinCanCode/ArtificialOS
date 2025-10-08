/**
 * Text Normalization Utilities
 * Functions for normalizing and tokenizing search text
 */

// ============================================================================
// Normalization
// ============================================================================

/**
 * Normalize text for search (lowercase, trim, remove diacritics)
 */
export function normalize(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, ""); // Remove diacritics
}

/**
 * Normalize with aggressive whitespace handling
 */
export function normalizeAggressive(text: string): string {
  return normalize(text).replace(/\s+/g, " ");
}

/**
 * Remove special characters
 */
export function removeSpecialChars(text: string): string {
  return text.replace(/[^a-zA-Z0-9\s]/g, "");
}

// ============================================================================
// Tokenization
// ============================================================================

/**
 * Tokenize text into words
 */
export function tokenize(text: string, separator: RegExp = /[\s\-_]+/): string[] {
  return text
    .split(separator)
    .map((token) => token.trim())
    .filter((token) => token.length > 0);
}

/**
 * Tokenize with normalization
 */
export function tokenizeNormalized(text: string, separator?: RegExp): string[] {
  return tokenize(normalize(text), separator);
}

/**
 * Get unique tokens
 */
export function uniqueTokens(text: string, separator?: RegExp): string[] {
  return Array.from(new Set(tokenize(text, separator)));
}

// ============================================================================
// String Distance Metrics
// ============================================================================

/**
 * Levenshtein distance (edit distance)
 */
export function levenshtein(a: string, b: string): number {
  const matrix: number[][] = [];

  for (let i = 0; i <= b.length; i++) {
    matrix[i] = [i];
  }

  for (let j = 0; j <= a.length; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= b.length; i++) {
    for (let j = 1; j <= a.length; j++) {
      if (b.charAt(i - 1) === a.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1];
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1, // substitution
          matrix[i][j - 1] + 1, // insertion
          matrix[i - 1][j] + 1 // deletion
        );
      }
    }
  }

  return matrix[b.length][a.length];
}

/**
 * Similarity ratio (0-1, higher is more similar)
 */
export function similarity(a: string, b: string): number {
  const distance = levenshtein(a, b);
  const maxLength = Math.max(a.length, b.length);
  return maxLength === 0 ? 1 : 1 - distance / maxLength;
}

/**
 * Check if strings are similar within threshold
 */
export function isSimilar(a: string, b: string, threshold: number = 0.7): boolean {
  return similarity(a, b) >= threshold;
}

