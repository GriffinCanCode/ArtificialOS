/**
 * Electron Preload Script
 * Exposes safe APIs to the renderer process
 */

console.log('[PRELOAD] Starting preload script...');

try {
  const { contextBridge, ipcRenderer } = require('electron');
  console.log('[PRELOAD] Electron modules loaded');

  // Try to load electron-log, but don't fail if it's not available
  let log = null;
  try {
    log = require('electron-log');
    console.log('[PRELOAD] electron-log loaded');
  } catch (logError) {
    console.warn('[PRELOAD] electron-log not available, using console fallback');
    // Fallback to console logging
    log = {
      error: console.error.bind(console),
      warn: console.warn.bind(console),
      info: console.info.bind(console),
      debug: console.debug.bind(console),
      verbose: console.log.bind(console),
    };
  }

  // Expose window controls
  contextBridge.exposeInMainWorld('electron', {
    minimize: () => {
      console.log('[PRELOAD] minimize called');
      return ipcRenderer.invoke('minimize-window');
    },
    maximize: () => {
      console.log('[PRELOAD] maximize called');
      return ipcRenderer.invoke('maximize-window');
    },
    close: () => {
      console.log('[PRELOAD] close called');
      return ipcRenderer.invoke('close-window');
    },
  });
  console.log('[PRELOAD] window.electron exposed');

  // Expose logger to renderer process
  contextBridge.exposeInMainWorld('electronLog', {
    error: (...args) => log.error(...args),
    warn: (...args) => log.warn(...args),
    info: (...args) => log.info(...args),
    debug: (...args) => log.debug(...args),
    verbose: (...args) => log.verbose(...args),
  });
  console.log('[PRELOAD] window.electronLog exposed');

  log.info('✅ Preload script loaded successfully');
  console.log('[PRELOAD] ✅ Preload script loaded successfully');
} catch (error) {
  console.error('[PRELOAD] Error loading preload script:', error);
}

