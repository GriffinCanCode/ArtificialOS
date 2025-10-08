/**
 * Fuzzy Search Implementation
 * Simple but effective fuzzy matching for app search
 */

export interface FuzzyMatch {
  score: number;
  matches: number[];
}

/**
 * Calculate fuzzy match score
 * Returns score (higher = better) and matched indices
 */
export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
  const queryLower = query.toLowerCase();
  const textLower = text.toLowerCase();

  // Fast path: exact substring match
  if (textLower.includes(queryLower)) {
    const startIdx = textLower.indexOf(queryLower);
    return {
      score: 100 - startIdx, // Prefer matches at start
      matches: Array.from({ length: query.length }, (_, i) => startIdx + i),
    };
  }

  // Fuzzy match: all query chars must appear in order
  let queryIdx = 0;
  let textIdx = 0;
  const matches: number[] = [];

  while (queryIdx < queryLower.length && textIdx < textLower.length) {
    if (queryLower[queryIdx] === textLower[textIdx]) {
      matches.push(textIdx);
      queryIdx++;
    }
    textIdx++;
  }

  // All query chars must be found
  if (queryIdx !== queryLower.length) {
    return null;
  }

  // Score: fewer gaps = better
  const gaps = matches.reduce((sum, idx, i) => {
    if (i === 0) return 0;
    return sum + (idx - matches[i - 1] - 1);
  }, 0);

  return {
    score: 50 - gaps, // Penalize gaps
    matches,
  };
}

/**
 * Search apps with fuzzy matching
 */
export function fuzzySearch<T>(
  items: T[],
  query: string,
  keyFn: (item: T) => string[]
): Array<T & { __score: number }> {
  if (!query.trim()) {
    return items.map((item) => ({ ...item, __score: 0 }));
  }

  const results: Array<T & { __score: number }> = [];

  for (const item of items) {
    const keys = keyFn(item);
    let bestScore = -1;

    // Check all search keys
    for (const key of keys) {
      const match = fuzzyMatch(query, key);
      if (match && match.score > bestScore) {
        bestScore = match.score;
      }
    }

    if (bestScore > -1) {
      results.push({ ...item, __score: bestScore });
    }
  }

  // Sort by score descending
  return results.sort((a, b) => b.__score - a.__score);
}

