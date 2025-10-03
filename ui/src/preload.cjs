/**
 * Electron Preload Script
 * Exposes safe APIs to the renderer process
 */

const { contextBridge, ipcRenderer } = require('electron');
const log = require('electron-log');

// Expose window controls
contextBridge.exposeInMainWorld('electron', {
  minimize: () => ipcRenderer.invoke('minimize-window'),
  maximize: () => ipcRenderer.invoke('maximize-window'),
  close: () => ipcRenderer.invoke('close-window'),
});

// Expose logger to renderer process
contextBridge.exposeInMainWorld('electronLog', {
  error: (...args) => log.error(...args),
  warn: (...args) => log.warn(...args),
  info: (...args) => log.info(...args),
  debug: (...args) => log.debug(...args),
  verbose: (...args) => log.verbose(...args),
});

log.info('✅ Preload script loaded');

