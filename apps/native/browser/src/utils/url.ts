/**
 * URL utilities for browser
 */

const SEARCH_ENGINES = {
  duckduckgo: 'https://duckduckgo.com/?q=',
  google: 'https://www.google.com/search?q=',
  bing: 'https://www.bing.com/search?q=',
};

/**
 * Check if input is a URL or search query
 */
export function isUrl(input: string): boolean {
  // Check for protocol
  if (input.startsWith('http://') || input.startsWith('https://')) {
    return true;
  }

  // Check for domain pattern (has dots and valid TLD)
  const domainPattern = /^[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(\/.*)?$/;
  if (domainPattern.test(input)) {
    return true;
  }

  // Check for localhost or IP
  if (input.startsWith('localhost') || /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/.test(input)) {
    return true;
  }

  return false;
}

/**
 * Normalize URL - add protocol if missing
 */
export function normalizeUrl(input: string): string {
  input = input.trim();

  if (!input) {
    return 'about:blank';
  }

  // Already has protocol
  if (input.startsWith('http://') || input.startsWith('https://')) {
    return input;
  }

  // Special URLs
  if (input.startsWith('about:')) {
    return input;
  }

  // Add https:// by default
  return `https://${input}`;
}

/**
 * Convert search query to URL
 */
export function searchToUrl(
  query: string,
  engine: 'duckduckgo' | 'google' | 'bing' = 'duckduckgo'
): string {
  const baseUrl = SEARCH_ENGINES[engine];
  return `${baseUrl}${encodeURIComponent(query)}`;
}

/**
 * Process input - determine if URL or search, return final URL
 */
export function processInput(
  input: string,
  searchEngine: 'duckduckgo' | 'google' | 'bing' = 'duckduckgo'
): string {
  input = input.trim();

  if (!input) {
    return 'about:blank';
  }

  if (isUrl(input)) {
    return normalizeUrl(input);
  } else {
    return searchToUrl(input, searchEngine);
  }
}

/**
 * Extract domain from URL
 */
export function getDomain(url: string): string {
  try {
    const urlObj = new URL(url);
    return urlObj.hostname;
  } catch {
    return '';
  }
}

/**
 * Extract title from URL (for display when title not available)
 */
export function getTitleFromUrl(url: string): string {
  if (url === 'about:blank') return 'New Tab';

  try {
    const urlObj = new URL(url);
    return urlObj.hostname;
  } catch {
    return url;
  }
}

/**
 * Check if URL is safe (basic validation)
 */
export function isSafeUrl(url: string): boolean {
  try {
    const urlObj = new URL(url);
    // Block dangerous protocols
    if (['javascript:', 'data:', 'vbscript:', 'file:'].includes(urlObj.protocol)) {
      return false;
    }
    return true;
  } catch {
    return false;
  }
}

