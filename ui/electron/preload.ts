/**
 * Electron Preload Script - Modern Implementation
 *
 * Exposes safe, validated APIs to the renderer process with enhanced security.
 *
 * Features:
 * - IPC channel validation and whitelisting
 * - Type-safe API with TypeScript
 * - Structured error handling with fallbacks
 * - Performance monitoring
 * - Secure contextBridge implementation
 * - Input/output validation
 *
 * @module preload
 * @version 2.0.0
 */

// ============================================================================
// PERFORMANCE TRACKING
// ============================================================================

const preloadStartTime = performance.now();
const isDev = process.env.NODE_ENV === 'development';

console.log('[PRELOAD] Starting preload script...');

// ============================================================================
// MODULE LOADING & SETUP
// ============================================================================

try {
  const { contextBridge, ipcRenderer } = require('electron');
  console.log('[PRELOAD] ✓ Electron modules loaded');

  // ============================================================================
  // LOGGING SETUP
  // ============================================================================

  interface Logger {
    error: (...args: any[]) => void;
    warn: (...args: any[]) => void;
    info: (...args: any[]) => void;
    debug: (...args: any[]) => void;
    verbose: (...args: any[]) => void;
  }

  let log: Logger;

  try {
    log = require('electron-log');
    log.info('[PRELOAD] ✓ electron-log loaded');
  } catch (logError) {
    console.warn('[PRELOAD] ⚠ electron-log not available, using console fallback');

    // Enhanced fallback logger with timestamp
    const timestamp = () => new Date().toISOString();
    log = {
      error: (...args) => console.error(`[${timestamp()}] [ERROR]`, ...args),
      warn: (...args) => console.warn(`[${timestamp()}] [WARN]`, ...args),
      info: (...args) => console.info(`[${timestamp()}] [INFO]`, ...args),
      debug: (...args) => isDev && console.debug(`[${timestamp()}] [DEBUG]`, ...args),
      verbose: (...args) => isDev && console.log(`[${timestamp()}] [VERBOSE]`, ...args),
    };
  }

  // ============================================================================
  // IPC SECURITY & VALIDATION
  // ============================================================================

  /**
   * Whitelist of allowed IPC channels for security
   */
  const ALLOWED_CHANNELS: Record<string, boolean> = {
    // Window controls
    'minimize-window': true,
    'maximize-window': true,
    'close-window': true,
    // System info
    'get-system-info': true,
    'get-native-theme': true,
  };

  /**
   * Validates if an IPC channel is whitelisted
   */
  const isChannelAllowed = (channel: string): boolean => {
    const allowed = ALLOWED_CHANNELS[channel] === true;
    if (!allowed) {
      log.error(`[PRELOAD] ⛔ Blocked unauthorized IPC channel: ${channel}`);
    }
    return allowed;
  };

  /**
   * Safely invokes an IPC channel with validation and error handling
   */
  const safeInvoke = async <T = any>(channel: string, ...args: any[]): Promise<T> => {
    if (!isChannelAllowed(channel)) {
      throw new Error(`IPC channel '${channel}' is not whitelisted`);
    }

    try {
      log.debug(`[PRELOAD] → IPC invoke: ${channel}`, args);
      const result = await ipcRenderer.invoke(channel, ...args);
      log.debug(`[PRELOAD] ← IPC result: ${channel}`, result);
      return result;
    } catch (error) {
      log.error(`[PRELOAD] ✗ IPC error on channel '${channel}':`, error);
      throw new Error(`IPC call failed: ${(error as Error).message}`);
    }
  };

  // ============================================================================
  // TYPE DEFINITIONS
  // ============================================================================

  interface SystemInfo {
    platform: string;
    arch: string;
    version: string;
    electron: string;
    chrome: string;
    node: string;
    isDev: boolean;
  }

  // ============================================================================
  // WINDOW CONTROLS API
  // ============================================================================

  const windowControls = {
    /**
     * Minimizes the application window
     */
    minimize: async (): Promise<boolean> => {
      try {
        log.debug('[PRELOAD] Window minimize requested');
        return await safeInvoke<boolean>('minimize-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to minimize window:', error);
        return false;
      }
    },

    /**
     * Toggles window between maximized and normal state
     */
    maximize: async (): Promise<boolean> => {
      try {
        log.debug('[PRELOAD] Window maximize toggle requested');
        return await safeInvoke<boolean>('maximize-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to toggle maximize:', error);
        return false;
      }
    },

    /**
     * Closes the application window
     */
    close: async (): Promise<boolean> => {
      try {
        log.debug('[PRELOAD] Window close requested');
        return await safeInvoke<boolean>('close-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to close window:', error);
        return false;
      }
    },
  };

  // ============================================================================
  // SYSTEM INFO API
  // ============================================================================

  const system = {
    /**
     * Gets comprehensive system and application information
     */
    getInfo: async (): Promise<SystemInfo> => {
      try {
        return await safeInvoke<SystemInfo>('get-system-info');
      } catch (error) {
        log.error('[PRELOAD] Failed to get system info:', error);
        // Return sensible defaults on error
        return {
          platform: 'unknown',
          arch: 'unknown',
          version: '0.0.0',
          electron: 'unknown',
          chrome: 'unknown',
          node: 'unknown',
          isDev: false,
        };
      }
    },

    /**
     * Gets the native system theme preference
     */
    getTheme: async (): Promise<'dark' | 'light'> => {
      try {
        return await safeInvoke<'dark' | 'light'>('get-native-theme');
      } catch (error) {
        log.error('[PRELOAD] Failed to get native theme:', error);
        return 'dark'; // Default fallback
      }
    },
  };

  // ============================================================================
  // EXPOSE SECURE APIs TO RENDERER
  // ============================================================================

  /**
   * Main Electron API exposed to renderer process
   */
  contextBridge.exposeInMainWorld('electron', {
    // Window controls
    ...windowControls,

    // System information
    system,

    // Version info
    versions: {
      electron: process.versions.electron,
      chrome: process.versions.chrome,
      node: process.versions.node,
    },

    // Environment
    env: {
      isDev,
      platform: process.platform,
    },
  });

  log.info('[PRELOAD] ✓ window.electron API exposed');

  // ============================================================================
  // EXPOSE LOGGER TO RENDERER
  // ============================================================================

  /**
   * Logger API exposed to renderer process
   */
  contextBridge.exposeInMainWorld('electronLog', {
    /**
     * Log error message
     */
    error: (...args: any[]): void => {
      log.error('[RENDERER]', ...args);
    },

    /**
     * Log warning message
     */
    warn: (...args: any[]): void => {
      log.warn('[RENDERER]', ...args);
    },

    /**
     * Log info message
     */
    info: (...args: any[]): void => {
      log.info('[RENDERER]', ...args);
    },

    /**
     * Log debug message (dev only)
     */
    debug: (...args: any[]): void => {
      log.debug('[RENDERER]', ...args);
    },

    /**
     * Log verbose message (dev only)
     */
    verbose: (...args: any[]): void => {
      log.verbose('[RENDERER]', ...args);
    },
  });

  log.info('[PRELOAD] ✓ window.electronLog API exposed');

  // ============================================================================
  // PERFORMANCE METRICS
  // ============================================================================

  const preloadEndTime = performance.now();
  const loadTime = (preloadEndTime - preloadStartTime).toFixed(2);

  log.info(`[PRELOAD] ✅ Preload script loaded successfully in ${loadTime}ms`);
  console.log(`[PRELOAD] ✅ Preload initialization complete (${loadTime}ms)`);

  // Expose performance info to renderer for debugging
  if (isDev) {
    contextBridge.exposeInMainWorld('__preloadMetrics', {
      loadTime: parseFloat(loadTime),
      timestamp: new Date().toISOString(),
    });
  }

} catch (error) {
  // ============================================================================
  // CRITICAL ERROR HANDLING
  // ============================================================================

  console.error('[PRELOAD] ✗ Critical error loading preload script:', error);
  console.error('[PRELOAD] Stack trace:', (error as Error).stack);

  // Attempt to expose a minimal error API to renderer
  try {
    const { contextBridge } = require('electron');
    contextBridge.exposeInMainWorld('electron', {
      error: (error as Error).message,
      available: false,
    });
    contextBridge.exposeInMainWorld('electronLog', {
      error: (...args: any[]) => console.error('[RENDERER ERROR]', ...args),
      warn: (...args: any[]) => console.warn('[RENDERER WARN]', ...args),
      info: (...args: any[]) => console.info('[RENDERER INFO]', ...args),
      debug: (...args: any[]) => console.debug('[RENDERER DEBUG]', ...args),
      verbose: (...args: any[]) => console.log('[RENDERER VERBOSE]', ...args),
    });
  } catch (bridgeError) {
    console.error('[PRELOAD] ✗ Failed to expose fallback API:', bridgeError);
  }
}

