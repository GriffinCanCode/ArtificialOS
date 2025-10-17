/**
 * Native App Loader
 * Dynamic module loading with caching and error handling
 *
 * Features:
 * - Dynamic ES module imports
 * - LRU cache with reference counting
 * - Development/production mode support
 * - Comprehensive error handling
 * - Memory-efficient cleanup
 */

/// <reference types="vite/client" />

import { logger } from "../../../core/utils/monitoring/logger";
import type { LoadedApp, CacheEntry } from "./types";
import { NativeAppError, ErrorCode } from "./types";

// ============================================================================
// Constants
// ============================================================================

const LOAD_TIMEOUT = 30000; // 30 seconds
const MAX_CACHE_SIZE = 20; // Maximum cached apps
const CACHE_TTL = 300000; // 5 minutes

// ============================================================================
// Loader Class
// ============================================================================

/**
 * Native App Loader
 * Handles dynamic loading and caching of native app modules
 */
class Loader {
  private cache = new Map<string, CacheEntry>();
  private loading = new Map<string, Promise<LoadedApp>>();

  /**
   * Load a native app module
   * Uses caching and reference counting
   */
  async load(packageId: string, bundlePath: string): Promise<LoadedApp> {
    // Check cache first
    const cached = this.cache.get(packageId);
    if (cached) {
      cached.refCount++;
      cached.lastAccessed = Date.now();
      logger.debug("Native app loaded from cache", {
        component: "NativeLoader",
        packageId,
        refCount: cached.refCount,
      });
      return cached.app;
    }

    // Check if already loading
    const existing = this.loading.get(packageId);
    if (existing) {
      logger.debug("Native app load already in progress", {
        component: "NativeLoader",
        packageId,
      });
      return existing;
    }

    // Start loading
    const promise = this.loadModule(packageId, bundlePath);
    this.loading.set(packageId, promise);

    try {
      const app = await promise;

      // Add to cache
      this.addToCache(packageId, app);

      return app;
    } finally {
      this.loading.delete(packageId);
    }
  }

  /**
   * Load module from bundle path
   */
  private async loadModule(packageId: string, bundlePath: string): Promise<LoadedApp> {
    logger.info("Loading native app module", {
      component: "NativeLoader",
      packageId,
      bundlePath,
    });

    const startTime = performance.now();

    try {
      // Load CSS first if it exists
      await this.loadCSS(packageId, bundlePath);

      // Add timeout
      const module = await Promise.race([
        this.importModule(bundlePath),
        this.timeout(LOAD_TIMEOUT),
      ]);

      // Validate module
      if (!module.default) {
        throw new NativeAppError(
          `App ${packageId} does not export a default component`,
          ErrorCode.NO_DEFAULT_EXPORT,
          packageId
        );
      }

      // Validate component is a function
      if (typeof module.default !== "function") {
        throw new NativeAppError(
          `Default export of ${packageId} is not a valid React component`,
          ErrorCode.INVALID_COMPONENT,
          packageId
        );
      }

      const loadTime = performance.now() - startTime;
      logger.info("Native app loaded successfully", {
        component: "NativeLoader",
        packageId,
        loadTime: `${loadTime.toFixed(2)}ms`,
      });

      return {
        id: packageId,
        component: module.default,
        cleanup: module.cleanup,
        loadedAt: Date.now(),
      };
    } catch (error) {
      const loadTime = performance.now() - startTime;
      logger.error("Failed to load native app", error as Error, {
        component: "NativeLoader",
        packageId,
        bundlePath,
        loadTime: `${loadTime.toFixed(2)}ms`,
      });

      if (error instanceof NativeAppError) {
        throw error;
      }

      throw new NativeAppError(
        `Failed to load app ${packageId}: ${(error as Error).message}`,
        ErrorCode.LOAD_FAILED,
        packageId
      );
    }
  }

  /**
   * Load CSS for native app
   */
  private async loadCSS(packageId: string, bundlePath: string): Promise<void> {
    // Derive CSS path from bundle path
    // bundlePath is like: /native-apps/file-explorer/index.js
    // CSS path is like: /native-apps/file-explorer/assets/index.css
    const basePath = bundlePath.substring(0, bundlePath.lastIndexOf("/"));
    const cssPath = `${basePath}/assets/index.css`;

    // Check if CSS already loaded
    const existingLink = document.querySelector(`link[data-app-id="${packageId}"]`);
    if (existingLink) {
      logger.debug("CSS already loaded for app", {
        component: "NativeLoader",
        packageId,
      });
      return;
    }

    // Try to fetch CSS to see if it exists
    try {
      const response = await fetch(cssPath, { method: "HEAD" });
      if (!response.ok) {
        // CSS doesn't exist, which is fine - not all apps need CSS
        logger.debug("No CSS file found for app (this is ok)", {
          component: "NativeLoader",
          packageId,
          cssPath,
        });
        return;
      }

      // CSS exists, inject it
      const link = document.createElement("link");
      link.rel = "stylesheet";
      link.href = cssPath;
      link.setAttribute("data-app-id", packageId);

      // Wait for CSS to load
      await new Promise<void>((resolve, _reject) => {
        link.onload = () => {
          logger.debug("CSS loaded successfully", {
            component: "NativeLoader",
            packageId,
            cssPath,
          });
          resolve();
        };
        link.onerror = () => {
          // CSS load failed, but don't fail the whole module load
          logger.warn("CSS load failed", {
            component: "NativeLoader",
            packageId,
            cssPath,
          });
          resolve();
        };
        document.head.appendChild(link);
      });
    } catch (error) {
      // Failed to check/load CSS, but don't fail the whole load
      logger.debug("CSS check/load failed (non-critical)", {
        component: "NativeLoader",
        packageId,
        error: (error as Error).message,
      });
    }
  }

  /**
   * Import module with vite ignore
   */
  private async importModule(bundlePath: string): Promise<any> {
    // In development, Vite handles HMR
    // In production, use built bundles
    if (import.meta.env.DEV) {
      // Development: Use Vite's dev server
      // bundlePath should be absolute or relative to public
      return await import(/* @vite-ignore */ bundlePath);
    } else {
      // Production: Use built bundle
      return await import(/* @vite-ignore */ bundlePath);
    }
  }

  /**
   * Timeout helper
   */
  private timeout(ms: number): Promise<never> {
    return new Promise((_, reject) => {
      setTimeout(() => {
        reject(new NativeAppError("Module load timeout", ErrorCode.TIMEOUT));
      }, ms);
    });
  }

  /**
   * Add to cache with LRU eviction
   */
  private addToCache(packageId: string, app: LoadedApp): void {
    // Evict if cache is full
    if (this.cache.size >= MAX_CACHE_SIZE) {
      this.evictLRU();
    }

    this.cache.set(packageId, {
      app,
      refCount: 1,
      lastAccessed: Date.now(),
    });

    logger.debug("Added to cache", {
      component: "NativeLoader",
      packageId,
      cacheSize: this.cache.size,
    });
  }

  /**
   * Evict least recently used entry
   */
  private evictLRU(): void {
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      // Don't evict if still in use
      if (entry.refCount > 0) continue;

      if (entry.lastAccessed < oldestTime) {
        oldestTime = entry.lastAccessed;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      const entry = this.cache.get(oldestKey)!;
      if (entry.app.cleanup) {
        try {
          entry.app.cleanup();
        } catch (error) {
          logger.error("Cleanup failed during eviction", error as Error, {
            component: "NativeLoader",
            packageId: oldestKey,
          });
        }
      }

      // Remove CSS
      this.removeCSS(oldestKey);

      this.cache.delete(oldestKey);

      logger.debug("Evicted from cache", {
        component: "NativeLoader",
        packageId: oldestKey,
      });
    }
  }

  /**
   * Release a loaded app
   * Decrements reference count
   */
  release(packageId: string): void {
    const entry = this.cache.get(packageId);
    if (!entry) return;

    entry.refCount = Math.max(0, entry.refCount - 1);
    logger.debug("Released app reference", {
      component: "NativeLoader",
      packageId,
      refCount: entry.refCount,
    });

    // Schedule cleanup if unused
    if (entry.refCount === 0) {
      this.scheduleCleanup(packageId);
    }
  }

  /**
   * Schedule cleanup of unused entry
   */
  private scheduleCleanup(packageId: string): void {
    setTimeout(() => {
      const entry = this.cache.get(packageId);
      if (!entry || entry.refCount > 0) return;

      // Check if stale
      if (Date.now() - entry.lastAccessed > CACHE_TTL) {
        if (entry.app.cleanup) {
          try {
            entry.app.cleanup();
          } catch (error) {
            logger.error("Cleanup failed", error as Error, {
              component: "NativeLoader",
              packageId,
            });
          }
        }

        // Remove CSS
        this.removeCSS(packageId);

        this.cache.delete(packageId);

        logger.debug("Cleaned up stale entry", {
          component: "NativeLoader",
          packageId,
        });
      }
    }, CACHE_TTL);
  }

  /**
   * Get loaded app from cache (without loading)
   */
  get(packageId: string): LoadedApp | undefined {
    return this.cache.get(packageId)?.app;
  }

  /**
   * Check if app is loaded
   */
  isLoaded(packageId: string): boolean {
    return this.cache.has(packageId);
  }

  /**
   * Remove CSS for a specific app
   */
  private removeCSS(packageId: string): void {
    const link = document.querySelector(`link[data-app-id="${packageId}"]`);
    if (link) {
      link.remove();
      logger.debug("CSS removed", {
        component: "NativeLoader",
        packageId,
      });
    }
  }

  /**
   * Clear all cache
   */
  clearCache(): void {
    for (const [packageId, entry] of this.cache.entries()) {
      if (entry.app.cleanup) {
        try {
          entry.app.cleanup();
        } catch (error) {
          logger.error("Cleanup failed during clear", error as Error, {
            component: "NativeLoader",
            packageId,
          });
        }
      }

      // Remove CSS
      this.removeCSS(packageId);
    }
    this.cache.clear();
    this.loading.clear();

    logger.info("Cache cleared", { component: "NativeLoader" });
  }

  /**
   * Get cache stats
   */
  getStats() {
    return {
      cacheSize: this.cache.size,
      loading: this.loading.size,
      entries: Array.from(this.cache.entries()).map(([id, entry]) => ({
        id,
        refCount: entry.refCount,
        lastAccessed: entry.lastAccessed,
        age: Date.now() - entry.app.loadedAt,
      })),
    };
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const loader = new Loader();
