/**
 * Hash Utilities
 *
 * Browser-based hashing using SubtleCrypto API.
 * Compatible with backend hashing (both use SHA-256).
 *
 * IMPORTANT:
 * - These are for CLIENT-SIDE hashing only (content verification, caching, etc.)
 * - Backend performs its own hashing for security-critical operations
 * - Uses Web Crypto API (SubtleCrypto) - modern browsers only
 *
 * Use cases:
 * - Content-based caching keys
 * - Detecting changes in UI state
 * - Client-side integrity checks
 * - Deterministic identifiers from content
 */

/**
 * Hash algorithm type
 */
export type HashAlgorithm = "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512";

/**
 * Default hash algorithm (matches backend)
 */
const DEFAULT_ALGORITHM: HashAlgorithm = "SHA-256";

/**
 * Checks if SubtleCrypto is available
 */
function isCryptoAvailable(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.crypto !== "undefined" &&
    typeof window.crypto.subtle !== "undefined"
  );
}

/**
 * Converts ArrayBuffer to hex string
 */
function bufferToHex(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Hash a string using the specified algorithm
 *
 * @param data - String to hash
 * @param algorithm - Hash algorithm (default: SHA-256)
 * @returns Promise<string> - Hex-encoded hash
 *
 * @example
 * const hash = await hashString("hello world");
 * // => "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
 */
export async function hashString(
  data: string,
  algorithm: HashAlgorithm = DEFAULT_ALGORITHM
): Promise<string> {
  if (!isCryptoAvailable()) {
    throw new Error("SubtleCrypto API not available in this environment");
  }

  const encoder = new TextEncoder();
  const dataBuffer = encoder.encode(data);
  const hashBuffer = await window.crypto.subtle.digest(algorithm, dataBuffer);
  return bufferToHex(hashBuffer);
}

/**
 * Hash an object (JSON-serializable) deterministically
 * Objects with same content produce same hash (keys are sorted)
 *
 * @param obj - Object to hash
 * @param algorithm - Hash algorithm (default: SHA-256)
 * @returns Promise<string> - Hex-encoded hash
 *
 * @example
 * const hash = await hashObject({ name: "Alice", age: 30 });
 */
export async function hashObject(
  obj: any,
  algorithm: HashAlgorithm = DEFAULT_ALGORITHM
): Promise<string> {
  // Sort keys for deterministic JSON
  const sorted = sortKeys(obj);
  const json = JSON.stringify(sorted);
  return hashString(json, algorithm);
}

/**
 * Hash multiple fields concatenated together
 * Fields are sorted for deterministic output
 *
 * @param fields - Array of strings to hash
 * @param algorithm - Hash algorithm (default: SHA-256)
 * @returns Promise<string> - Hex-encoded hash
 *
 * @example
 * const hash = await hashFields(["user123", "2024-01-10", "action"]);
 */
export async function hashFields(
  fields: string[],
  algorithm: HashAlgorithm = DEFAULT_ALGORITHM
): Promise<string> {
  // Sort fields for deterministic ordering (matches backend behavior)
  const sorted = [...fields].sort();
  const combined = sorted.join("|");
  return hashString(combined, algorithm);
}

/**
 * Generate a short hash (first N characters)
 * Useful for display or file names
 *
 * @param fullHash - Full hash string
 * @param length - Number of characters (default: 8)
 * @returns Shortened hash
 *
 * @example
 * const short = shortHash("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9", 8);
 * // => "b94d27b9"
 */
export function shortHash(fullHash: string, length = 8): string {
  return fullHash.substring(0, length);
}

/**
 * Verify that data matches a given hash
 *
 * @param data - Data to verify
 * @param expectedHash - Expected hash value
 * @param algorithm - Hash algorithm (default: SHA-256)
 * @returns Promise<boolean> - True if hashes match
 */
export async function verifyHash(
  data: string,
  expectedHash: string,
  algorithm: HashAlgorithm = DEFAULT_ALGORITHM
): Promise<boolean> {
  const actualHash = await hashString(data, algorithm);
  return actualHash === expectedHash;
}

/**
 * Sort object keys recursively for deterministic serialization
 * (Internal helper function)
 */
function sortKeys(obj: any): any {
  if (obj === null || typeof obj !== "object") {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map(sortKeys);
  }

  const sorted: Record<string, any> = {};
  Object.keys(obj)
    .sort()
    .forEach((key) => {
      sorted[key] = sortKeys(obj[key]);
    });

  return sorted;
}

/**
 * Hasher class for consistent hashing configuration
 * Similar to backend Hasher pattern
 */
export class Hasher {
  constructor(private algorithm: HashAlgorithm = DEFAULT_ALGORITHM) {}

  /**
   * Hash a string
   */
  async hash(data: string): Promise<string> {
    return hashString(data, this.algorithm);
  }

  /**
   * Hash an object
   */
  async hashObject(obj: any): Promise<string> {
    return hashObject(obj, this.algorithm);
  }

  /**
   * Hash multiple fields
   */
  async hashFields(...fields: string[]): Promise<string> {
    return hashFields(fields, this.algorithm);
  }

  /**
   * Verify hash
   */
  async verify(data: string, expectedHash: string): Promise<boolean> {
    return verifyHash(data, expectedHash, this.algorithm);
  }

  /**
   * Generate short hash
   */
  shortHash(fullHash: string, length = 8): string {
    return shortHash(fullHash, length);
  }
}

/**
 * Default hasher instance (SHA-256)
 * Use this for consistent hashing across your app
 */
export const defaultHasher = new Hasher();

/**
 * Simple non-cryptographic hash for fast cache keys
 * NOT secure - use only for non-security purposes
 *
 * @param str - String to hash
 * @returns number - 32-bit hash
 */
export function simpleHash(str: string): number {
  let hash = 0;
  if (str.length === 0) return hash;

  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32-bit integer
  }

  return Math.abs(hash);
}

/**
 * Generate a cache key from multiple values
 * Uses simple hash for performance
 *
 * @param values - Values to include in cache key
 * @returns string - Cache key
 */
export function cacheKey(...values: any[]): string {
  const combined = values.map((v) => String(v)).join(":");
  return `cache_${simpleHash(combined)}`;
}
