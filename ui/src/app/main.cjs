/**
 * Electron Main Process - Modern Implementation
 * Manages application lifecycle, window creation, and system integration
 *
 * Features:
 * - Window state persistence
 * - Single instance lock
 * - Enhanced security
 * - Native menu integration
 * - Crash recovery
 * - Performance optimizations
 */

const { app, BrowserWindow, ipcMain, Menu, shell, dialog, nativeTheme } = require('electron');
const path = require('path');
const fs = require('fs');
const log = require('electron-log');

// ============================================================================
// CONFIGURATION & CONSTANTS
// ============================================================================

const isDev = !app.isPackaged;
const WINDOW_STATE_FILE = path.join(app.getPath('userData'), 'window-state.json');

// Configure electron-log with rotation and better formatting
log.transports.file.level = isDev ? 'debug' : 'info';
log.transports.console.level = isDev ? 'debug' : 'warn';
log.transports.file.maxSize = 10 * 1024 * 1024; // 10MB
log.transports.file.format = '[{y}-{m}-{d} {h}:{i}:{s}.{ms}] [{level}] {text}';
log.catchErrors({
  showDialog: false,
  onError: (error) => {
    log.error('Unhandled error:', error);
  }
});

log.info('='.repeat(80));
log.info('AgentOS UI Starting');
log.info('Version:', app.getVersion());
log.info('Electron:', process.versions.electron);
log.info('Chrome:', process.versions.chrome);
log.info('Node:', process.versions.node);
log.info('Platform:', process.platform);
log.info('Architecture:', process.arch);
log.info('Development Mode:', isDev);
log.info('User Data Path:', app.getPath('userData'));
log.info('Log File:', log.transports.file.getFile().path);
log.info('='.repeat(80));

// ============================================================================
// WINDOW STATE MANAGEMENT
// ============================================================================

class WindowStateManager {
  constructor() {
    this.defaultState = {
      width: 1400,
      height: 900,
      x: undefined,
      y: undefined,
      isMaximized: false,
    };
    this.state = this.loadState();
  }

  loadState() {
    try {
      if (fs.existsSync(WINDOW_STATE_FILE)) {
        const data = fs.readFileSync(WINDOW_STATE_FILE, 'utf8');
        const state = JSON.parse(data);
        log.info('Loaded window state:', state);
        return { ...this.defaultState, ...state };
      }
    } catch (error) {
      log.warn('Failed to load window state:', error.message);
    }
    return { ...this.defaultState };
  }

  saveState(window) {
    try {
      if (!window) return;

      const bounds = window.getBounds();
      const state = {
        width: bounds.width,
        height: bounds.height,
        x: bounds.x,
        y: bounds.y,
        isMaximized: window.isMaximized(),
      };

      fs.writeFileSync(WINDOW_STATE_FILE, JSON.stringify(state, null, 2), 'utf8');
      log.debug('Saved window state');
    } catch (error) {
      log.error('Failed to save window state:', error);
    }
  }

  track(window) {
    // Save state on window events
    const saveHandler = () => this.saveState(window);

    window.on('resize', saveHandler);
    window.on('move', saveHandler);
    window.on('close', saveHandler);
    window.on('maximize', saveHandler);
    window.on('unmaximize', saveHandler);

    // Restore maximized state
    if (this.state.isMaximized) {
      window.maximize();
    }
  }

  getState() {
    return this.state;
  }
}

const windowStateManager = new WindowStateManager();

// ============================================================================
// APPLICATION MENU
// ============================================================================

function createApplicationMenu() {
  const template = [
    {
      label: 'File',
      submenu: [
        {
          label: 'Reload',
          accelerator: 'CmdOrCtrl+R',
          click: (item, focusedWindow) => {
            if (focusedWindow) focusedWindow.reload();
          }
        },
        { type: 'separator' },
        {
          label: 'Quit',
          accelerator: process.platform === 'darwin' ? 'Cmd+Q' : 'Alt+F4',
          click: () => app.quit()
        }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' }
      ]
    },
    {
      label: 'View',
      submenu: [
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
        ...(isDev ? [
          { type: 'separator' },
          { role: 'toggleDevTools' }
        ] : [])
      ]
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'zoom' },
        ...(process.platform === 'darwin' ? [
          { type: 'separator' },
          { role: 'front' },
          { type: 'separator' },
          { role: 'window' }
        ] : [
          { role: 'close' }
        ])
      ]
    },
    {
      label: 'Help',
      submenu: [
        {
          label: 'Learn More',
          click: async () => {
            await shell.openExternal('https://github.com/yourusername/agentOS');
          }
        },
        { type: 'separator' },
        {
          label: 'Toggle Developer Tools',
          accelerator: process.platform === 'darwin' ? 'Alt+Command+I' : 'Ctrl+Shift+I',
          click: (item, focusedWindow) => {
            if (focusedWindow) focusedWindow.webContents.toggleDevTools();
          }
        }
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
  log.info('Application menu created');
}

// ============================================================================
// WINDOW CREATION
// ============================================================================

let mainWindow = null;

function createWindow() {
  log.info('Creating main window');

  const windowState = windowStateManager.getState();
  const preloadPath = path.join(__dirname, 'preload.cjs');

  log.info('Preload script path:', preloadPath);
  log.info('Preload script exists:', fs.existsSync(preloadPath));

  mainWindow = new BrowserWindow({
    width: windowState.width,
    height: windowState.height,
    x: windowState.x,
    y: windowState.y,
    minWidth: 800,
    minHeight: 600,
    backgroundColor: '#0a0a0a',
    show: false, // Don't show until ready-to-show
    frame: false,
    titleBarStyle: 'hidden',
    trafficLightPosition: { x: 10, y: 10 },
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true, // Enhanced security
      webSecurity: true,
      allowRunningInsecureContent: false,
      preload: preloadPath,
      // Performance optimizations
      backgroundThrottling: false,
      // Disable features not needed
      enableWebSQL: false,
      navigateOnDragDrop: false,
    },
  });

  // Track window state changes
  windowStateManager.track(mainWindow);

  // Show window when ready to prevent flickering
  mainWindow.once('ready-to-show', () => {
    log.info('Window ready to show');
    mainWindow.show();
  });

  // Load the React app
  const loadPromise = isDev
    ? mainWindow.loadURL('http://localhost:5173')
    : mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));

  loadPromise.then(() => {
    log.info(`Successfully loaded ${isDev ? 'dev server' : 'built files'}`);
    if (isDev) {
      mainWindow.webContents.openDevTools();
    }
  }).catch((error) => {
    log.error('Failed to load app:', error);
    dialog.showErrorBox('Load Error', `Failed to load application: ${error.message}`);
  });

  // ============================================================================
  // WINDOW EVENT HANDLERS
  // ============================================================================

  mainWindow.webContents.on('did-finish-load', () => {
    log.info('Content finished loading');
  });

  mainWindow.webContents.on('did-fail-load', (event, errorCode, errorDescription) => {
    log.error('Failed to load content:', { errorCode, errorDescription });
  });

  mainWindow.webContents.on('crashed', (event) => {
    log.error('Renderer process crashed');
    const options = {
      type: 'error',
      title: 'Renderer Crashed',
      message: 'The application has crashed. Would you like to reload?',
      buttons: ['Reload', 'Close']
    };

    dialog.showMessageBox(options).then(({ response }) => {
      if (response === 0) {
        mainWindow.reload();
      } else {
        app.quit();
      }
    });
  });

  mainWindow.webContents.on('unresponsive', () => {
    log.warn('Window became unresponsive');
  });

  mainWindow.webContents.on('responsive', () => {
    log.info('Window became responsive again');
  });

  // Security: Prevent navigation to external URLs
  mainWindow.webContents.on('will-navigate', (event, url) => {
    const appUrl = isDev ? 'http://localhost:5173' : 'file://';
    if (!url.startsWith(appUrl)) {
      event.preventDefault();
      log.warn('Prevented navigation to:', url);
    }
  });

  // Security: Handle new window requests
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    // Open external links in default browser
    if (url.startsWith('http://') || url.startsWith('https://')) {
      shell.openExternal(url);
    }
    return { action: 'deny' };
  });

  mainWindow.on('closed', () => {
    log.info('Main window closed');
    mainWindow = null;
  });

  // Performance: Clear cache periodically in dev mode
  if (isDev) {
    setInterval(() => {
      mainWindow?.webContents.session.clearCache().then(() => {
        log.debug('Cache cleared');
      });
    }, 5 * 60 * 1000); // Every 5 minutes
  }

  return mainWindow;
}

// ============================================================================
// IPC HANDLERS
// ============================================================================

function setupIpcHandlers() {
  // Window controls
  ipcMain.handle('minimize-window', () => {
    log.debug('IPC: minimize-window');
    mainWindow?.minimize();
    return true;
  });

  ipcMain.handle('maximize-window', () => {
    const isMaximized = mainWindow?.isMaximized();
    log.debug(`IPC: maximize-window (currently: ${isMaximized})`);

    if (mainWindow) {
      if (isMaximized) {
        mainWindow.unmaximize();
      } else {
        mainWindow.maximize();
      }
    }
    return !isMaximized;
  });

  ipcMain.handle('close-window', () => {
    log.debug('IPC: close-window');
    mainWindow?.close();
    return true;
  });

  // System information
  ipcMain.handle('get-system-info', () => {
    log.debug('IPC: get-system-info');
    return {
      platform: process.platform,
      arch: process.arch,
      version: app.getVersion(),
      electron: process.versions.electron,
      chrome: process.versions.chrome,
      node: process.versions.node,
      isDev,
    };
  });

  // Theme
  ipcMain.handle('get-native-theme', () => {
    return nativeTheme.shouldUseDarkColors ? 'dark' : 'light';
  });

  ipcMain.on('set-native-theme', (event, theme) => {
    nativeTheme.themeSource = theme;
  });

  log.info('IPC handlers registered');
}

// ============================================================================
// APP LIFECYCLE
// ============================================================================

// Single instance lock
const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
  log.warn('Another instance is already running. Quitting...');
  app.quit();
} else {
  app.on('second-instance', (event, commandLine, workingDirectory) => {
    log.info('Second instance detected, focusing existing window');

    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
    }
  });

  // Initialization
  app.whenReady().then(async () => {
    log.info('App ready, initializing...');

    // Create application menu
    createApplicationMenu();

    // Setup IPC handlers
    setupIpcHandlers();

    // Create main window
    createWindow();

    log.info('Initialization complete');
  });
}

// macOS: Re-activate
app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    log.info('Reactivating app - creating new window');
    createWindow();
  }
});

// Quit when all windows closed (except macOS)
app.on('window-all-closed', () => {
  log.info('All windows closed');
  if (process.platform !== 'darwin') {
    log.info('Quitting application');
    app.quit();
  }
});

// Cleanup before quit
app.on('before-quit', () => {
  log.info('Application quitting...');
});

app.on('will-quit', () => {
  log.info('Application will quit');
});

// ============================================================================
// ERROR HANDLING
// ============================================================================

process.on('uncaughtException', (error) => {
  log.error('Uncaught exception in main process:', error);
  // In production, you might want to send this to a crash reporting service
});

process.on('unhandledRejection', (reason, promise) => {
  log.error('Unhandled rejection in main process:', reason);
});

// Disable hardware acceleration if needed (helps with some GPU issues)
if (process.env.DISABLE_HARDWARE_ACCELERATION) {
  log.info('Hardware acceleration disabled');
  app.disableHardwareAcceleration();
}

log.info('Electron main process initialized');

