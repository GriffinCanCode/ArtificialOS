/**
 * Electron Preload Script - Modern Implementation
 *
 * Exposes safe, validated APIs to the renderer process with enhanced security.
 *
 * Features:
 * - IPC channel validation and whitelisting
 * - Type-safe API with JSDoc annotations
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

  /**
   * @typedef {Object} Logger
   * @property {(...args: any[]) => void} error
   * @property {(...args: any[]) => void} warn
   * @property {(...args: any[]) => void} info
   * @property {(...args: any[]) => void} debug
   * @property {(...args: any[]) => void} verbose
   */

  /** @type {Logger} */
  let log;

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
   * Only these channels can be invoked from the renderer process
   */
  const ALLOWED_CHANNELS = {
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
   * @param {string} channel - The IPC channel name
   * @returns {boolean} Whether the channel is allowed
   */
  const isChannelAllowed = (channel) => {
    const allowed = ALLOWED_CHANNELS[channel] === true;
    if (!allowed) {
      log.error(`[PRELOAD] ⛔ Blocked unauthorized IPC channel: ${channel}`);
    }
    return allowed;
  };

  /**
   * Safely invokes an IPC channel with validation and error handling
   * @param {string} channel - The IPC channel name
   * @param {...any} args - Arguments to pass to the IPC handler
   * @returns {Promise<any>} The result from the main process
   * @throws {Error} If the channel is not whitelisted or IPC fails
   */
  const safeInvoke = async (channel, ...args) => {
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
      throw new Error(`IPC call failed: ${error.message}`);
    }
  };

  // ============================================================================
  // WINDOW CONTROLS API
  // ============================================================================

  /**
   * @typedef {Object} WindowControlsAPI
   * @property {() => Promise<boolean>} minimize - Minimize the window
   * @property {() => Promise<boolean>} maximize - Toggle maximize/unmaximize
   * @property {() => Promise<boolean>} close - Close the window
   * @property {() => Promise<boolean>} isMaximized - Check if window is maximized
   */

  /** @type {WindowControlsAPI} */
  const windowControls = {
    /**
     * Minimizes the application window
     * @returns {Promise<boolean>} Success status
     */
    minimize: async () => {
      try {
        log.debug('[PRELOAD] Window minimize requested');
        return await safeInvoke('minimize-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to minimize window:', error);
        return false;
      }
    },

    /**
     * Toggles window between maximized and normal state
     * @returns {Promise<boolean>} New maximized state
     */
    maximize: async () => {
      try {
        log.debug('[PRELOAD] Window maximize toggle requested');
        return await safeInvoke('maximize-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to toggle maximize:', error);
        return false;
      }
    },

    /**
     * Closes the application window
     * @returns {Promise<boolean>} Success status
     */
    close: async () => {
      try {
        log.debug('[PRELOAD] Window close requested');
        return await safeInvoke('close-window');
      } catch (error) {
        log.error('[PRELOAD] Failed to close window:', error);
        return false;
      }
    },
  };

  // ============================================================================
  // SYSTEM INFO API
  // ============================================================================

  /**
   * @typedef {Object} SystemInfo
   * @property {string} platform - Operating system platform
   * @property {string} arch - CPU architecture
   * @property {string} version - Application version
   * @property {string} electron - Electron version
   * @property {string} chrome - Chrome version
   * @property {string} node - Node.js version
   * @property {boolean} isDev - Development mode flag
   */

  /**
   * @typedef {Object} SystemAPI
   * @property {() => Promise<SystemInfo>} getInfo - Get system information
   * @property {() => Promise<'dark'|'light'>} getTheme - Get native theme
   */

  /** @type {SystemAPI} */
  const system = {
    /**
     * Gets comprehensive system and application information
     * @returns {Promise<SystemInfo>} System information object
     */
    getInfo: async () => {
      try {
        return await safeInvoke('get-system-info');
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
     * @returns {Promise<'dark'|'light'>} Theme preference
     */
    getTheme: async () => {
      try {
        return await safeInvoke('get-native-theme');
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
   * All methods are validated and sandboxed through contextBridge
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
   * Provides consistent logging across main and renderer processes
   */
  contextBridge.exposeInMainWorld('electronLog', {
    /**
     * Log error message
     * @param {...any} args - Arguments to log
     */
    error: (...args) => {
      log.error('[RENDERER]', ...args);
    },

    /**
     * Log warning message
     * @param {...any} args - Arguments to log
     */
    warn: (...args) => {
      log.warn('[RENDERER]', ...args);
    },

    /**
     * Log info message
     * @param {...any} args - Arguments to log
     */
    info: (...args) => {
      log.info('[RENDERER]', ...args);
    },

    /**
     * Log debug message (dev only)
     * @param {...any} args - Arguments to log
     */
    debug: (...args) => {
      log.debug('[RENDERER]', ...args);
    },

    /**
     * Log verbose message (dev only)
     * @param {...any} args - Arguments to log
     */
    verbose: (...args) => {
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
  console.error('[PRELOAD] Stack trace:', error.stack);

  // Attempt to expose a minimal error API to renderer
  try {
    const { contextBridge } = require('electron');
    contextBridge.exposeInMainWorld('electron', {
      error: error.message,
      available: false,
    });
    contextBridge.exposeInMainWorld('electronLog', {
      error: (...args) => console.error('[RENDERER ERROR]', ...args),
      warn: (...args) => console.warn('[RENDERER WARN]', ...args),
      info: (...args) => console.info('[RENDERER INFO]', ...args),
      debug: (...args) => console.debug('[RENDERER DEBUG]', ...args),
      verbose: (...args) => console.log('[RENDERER VERBOSE]', ...args),
    });
  } catch (bridgeError) {
    console.error('[PRELOAD] ✗ Failed to expose fallback API:', bridgeError);
  }
}
