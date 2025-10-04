/**
 * Electron Preload Script
 * Exposes safe APIs to the renderer process
 */

console.log('[PRELOAD] Starting preload script...');

try {
  const { contextBridge, ipcRenderer } = require('electron');
  console.log('[PRELOAD] Electron modules loaded');
  
  const log = require('electron-log');
  console.log('[PRELOAD] electron-log loaded');

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

