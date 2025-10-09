/**
 * Standardized Filesystem Paths
 * Centralized path constants for consistent filesystem structure across frontend
 *
 * This mirrors the kernel and backend path structure to ensure consistency.
 * Any changes should be synchronized with:
 * - kernel/src/vfs/paths.rs
 * - backend/internal/shared/paths/paths.go
 */

// Mount points
export const MOUNTS = {
  STORAGE: '/storage',
  TMP: '/tmp',
  CACHE: '/cache',
} as const;

// Storage subdirectories
export const STORAGE = {
  /** Prebuilt applications bundled with the OS */
  NATIVE_APPS: '/storage/native-apps',

  /** User-generated or AI-generated applications */
  APPS: '/storage/apps',

  /** User files and data */
  USER: '/storage/user',

  /** System configuration and data */
  SYSTEM: '/storage/system',

  /** Shared libraries and resources */
  LIB: '/storage/lib',
} as const;

// User subdirectories
export const USER = {
  DOCUMENTS: '/storage/user/documents',
  DOWNLOADS: '/storage/user/downloads',
  PROJECTS: '/storage/user/projects',
} as const;

/**
 * Get application-specific paths
 */
export class AppPaths {
  constructor(private readonly appId: string) {
    if (!appId) {
      throw new Error('App ID cannot be empty');
    }
  }

  /** Get app's data directory */
  dataDir(): string {
    return `${STORAGE.APPS}/${this.appId}/data`;
  }

  /** Get app's config directory */
  configDir(): string {
    return `${STORAGE.APPS}/${this.appId}/config`;
  }

  /** Get app's cache directory */
  cacheDir(): string {
    return `${MOUNTS.CACHE}/${this.appId}`;
  }

  /** Get app's temp directory */
  tempDir(): string {
    return `${MOUNTS.TMP}/${this.appId}`;
  }

  /** Resolve a relative path within the app */
  resolve(relativePath: string): string {
    return `${STORAGE.APPS}/${this.appId}/${relativePath}`;
  }
}

/**
 * Create app-specific path helper
 */
export function appPath(appId: string): AppPaths {
  return new AppPaths(appId);
}

/**
 * Check if path is within allowed userspace
 */
export function isUserspacePath(path: string): boolean {
  return (
    path.startsWith(STORAGE.APPS) ||
    path.startsWith(STORAGE.USER) ||
    path.startsWith(MOUNTS.TMP) ||
    path.startsWith(MOUNTS.CACHE)
  );
}

/**
 * Check if path is a native app
 */
export function isNativeAppPath(path: string): boolean {
  return path.startsWith(STORAGE.NATIVE_APPS);
}

/**
 * Check if path is system-protected
 */
export function isSystemPath(path: string): boolean {
  return path.startsWith(STORAGE.SYSTEM);
}

/**
 * Normalize path (remove trailing slashes, clean up)
 */
export function normalizePath(path: string): string {
  // Remove trailing slash except for root
  if (path.length > 1 && path.endsWith('/')) {
    path = path.slice(0, -1);
  }

  // Remove duplicate slashes
  path = path.replace(/\/+/g, '/');

  return path;
}

/**
 * Join path segments
 */
export function joinPath(...segments: string[]): string {
  return normalizePath(segments.join('/'));
}

/**
 * Get parent directory of path
 */
export function dirname(path: string): string {
  const normalized = normalizePath(path);
  const lastSlash = normalized.lastIndexOf('/');

  if (lastSlash === 0) {
    return '/';
  }

  if (lastSlash === -1) {
    return '.';
  }

  return normalized.slice(0, lastSlash);
}

/**
 * Get filename from path
 */
export function basename(path: string): string {
  const normalized = normalizePath(path);
  const lastSlash = normalized.lastIndexOf('/');

  if (lastSlash === -1) {
    return normalized;
  }

  return normalized.slice(lastSlash + 1);
}

/**
 * All standard directories
 */
export const STANDARD_DIRECTORIES = [
  STORAGE.NATIVE_APPS,
  STORAGE.APPS,
  STORAGE.USER,
  STORAGE.SYSTEM,
  STORAGE.LIB,
  USER.DOCUMENTS,
  USER.DOWNLOADS,
  USER.PROJECTS,
] as const;

/**
 * Path validation
 */
export function validatePath(path: string): { valid: boolean; error?: string } {
  if (!path) {
    return { valid: false, error: 'Path cannot be empty' };
  }

  if (path.includes('..')) {
    return { valid: false, error: 'Path cannot contain ..' };
  }

  if (path.includes('//')) {
    return { valid: false, error: 'Path cannot contain //' };
  }

  return { valid: true };
}

