/**
 * Electron Main Process
 * Manages application lifecycle and window creation
 */

const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const log = require('electron-log');

// Configure electron-log
log.transports.file.level = 'debug';
log.transports.console.level = 'debug';
log.transports.file.maxSize = 10 * 1024 * 1024; // 10MB
log.transports.file.format = '[{y}-{m}-{d} {h}:{i}:{s}.{ms}] [{level}] {text}';

// Log file location
log.info('Log file location:', log.transports.file.getFile().path);

let mainWindow;

function createWindow() {
  log.info('Creating main window');
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 800,
    minHeight: 600,
    backgroundColor: '#0a0a0a',
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.cjs')
    },
    titleBarStyle: 'hiddenInset',
    frame: false,
  });

  // Load the React app
  // In development, load from Vite dev server
  const isDev = !app.isPackaged;
  
  if (isDev) {
    log.info('Loading app from Vite dev server: http://localhost:5173');
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    log.info('Loading app from built files');
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  mainWindow.webContents.on('did-finish-load', () => {
    log.info('Main window finished loading');
  });

  mainWindow.webContents.on('did-fail-load', (event, errorCode, errorDescription) => {
    log.error('Main window failed to load:', { errorCode, errorDescription });
  });

  mainWindow.on('closed', () => {
    log.info('Main window closed');
    mainWindow = null;
  });
}

// App lifecycle
app.whenReady().then(() => {
  log.info('🚀 AI-OS UI starting...');
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      log.info('Reactivating app - creating new window');
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  log.info('All windows closed');
  if (process.platform !== 'darwin') {
    log.info('Quitting application');
    app.quit();
  }
});

// IPC handlers
ipcMain.handle('minimize-window', () => {
  log.debug('Minimize window requested');
  mainWindow?.minimize();
});

ipcMain.handle('maximize-window', () => {
  const isMaximized = mainWindow?.isMaximized();
  log.debug(`Toggle maximize window - currently maximized: ${isMaximized}`);
  if (isMaximized) {
    mainWindow?.unmaximize();
  } else {
    mainWindow?.maximize();
  }
});

ipcMain.handle('close-window', () => {
  log.debug('Close window requested');
  mainWindow?.close();
});

// Error handling
process.on('uncaughtException', (error) => {
  log.error('Uncaught exception in main process:', error);
});

process.on('unhandledRejection', (reason, promise) => {
  log.error('Unhandled rejection in main process:', reason);
});

log.info('✅ Electron main process ready');

