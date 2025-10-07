/**
 * Electron Main Process - Modern Implementation
 *
 * Manages application lifecycle, window creation, and system integration with
 * enterprise-grade security, performance, and reliability features.
 *
 * Features:
 * - Window state persistence with automatic save/restore
 * - Single instance lock preventing multiple app instances
 * - Enhanced security with sandbox and IPC validation
 * - Native menu integration with platform-specific items
 * - Crash recovery with user dialog and auto-reload
 * - Performance optimizations and hardware acceleration control
 *
 * @module main
 * @version 2.0.0
 */

const { app, BrowserWindow, ipcMain, Menu, shell, dialog, nativeTheme } = require('electron');
const path = require('path');
const fs = require('fs');
const log = require('electron-log');

// ============================================================================
// CONFIGURATION & CONSTANTS
// ============================================================================

/**
 * @typedef {Object} WindowState
 * @property {number} width - Window width in pixels
 * @property {number} height - Window height in pixels
 * @property {number} [x] - Window x position (undefined for center)
 * @property {number} [y] - Window y position (undefined for center)
 * @property {boolean} isMaximized - Whether window is maximized
 */

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

/** @type {boolean} Development mode flag based on package status */
const isDev = !app.isPackaged;

/** @type {string} Path to window state persistence file */
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

/**
 * Manages window state persistence across application restarts.
 * Automatically saves window size, position, and maximized state to disk
 * and restores it on next launch.
 *
 * @class WindowStateManager
 */
class WindowStateManager {
  /**
   * Creates a new WindowStateManager instance
   * @constructor
   */
  constructor() {
    /** @type {WindowState} Default window state for first launch */
    this.defaultState = {
      width: 1400,
      height: 900,
      x: undefined,
      y: undefined,
      isMaximized: false,
    };
    /** @type {WindowState} Current window state */
    this.state = this.loadState();
  }

  /**
   * Loads window state from disk
   * Falls back to default state if file doesn't exist or is corrupted
   * @returns {WindowState} Loaded or default window state
   */
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

  /**
   * Saves current window state to disk
   * @param {BrowserWindow} window - The window to save state from
   */
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

  /**
   * Tracks window events and automatically saves state on changes
   * Also restores maximized state if window was maximized on last close
   * @param {BrowserWindow} window - The window to track
   */
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

  /**
   * Gets the current window state
   * @returns {WindowState} Current window state
   */
  getState() {
    return this.state;
  }
}

/** @type {WindowStateManager} Global window state manager instance */
const windowStateManager = new WindowStateManager();

// ============================================================================
// APPLICATION MENU
// ============================================================================

/**
 * Creates and sets the native application menu.
 * Includes File, Edit, View, Window, and Help menus with platform-specific
 * items (e.g., different accelerators for macOS vs Windows/Linux).
 *
 * Menu structure:
 * - File: Reload, Quit
 * - Edit: Undo, Redo, Cut, Copy, Paste, Select All
 * - View: Zoom controls, Toggle Fullscreen, Toggle DevTools (dev only)
 * - Window: Minimize, Zoom/Maximize, Close, macOS-specific items
 * - Help: Learn More, Toggle Developer Tools
 *
 * @function createApplicationMenu
 */
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

/**
 * @type {BrowserWindow | null}
 * Reference to the main application window
 */
let mainWindow = null;

/**
 * Creates the main application window with enhanced security and performance settings.
 *
 * Security features:
 * - Context isolation and sandbox enabled
 * - Node integration disabled
 * - Web security enforced
 * - Navigation and window opening restrictions
 *
 * Performance features:
 * - ready-to-show pattern to prevent flickering
 * - Background throttling disabled for UI responsiveness
 * - Periodic cache clearing in development mode
 *
 * Window features:
 * - Frameless with custom title bar
 * - Persistent state (size, position, maximized)
 * - Crash recovery with user dialog
 * - DevTools auto-open in development
 *
 * @function createWindow
 * @returns {BrowserWindow} The created main window instance
 */
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

/**
 * Registers all IPC handlers for communication between main and renderer processes.
 *
 * Registered handlers:
 * - minimize-window: Minimizes the main window
 * - maximize-window: Toggles window maximize/unmaximize state
 * - close-window: Closes the main window
 * - get-system-info: Returns system and application information
 * - get-native-theme: Returns current system theme (dark/light)
 * - set-native-theme: Sets the application theme
 *
 * All handlers are validated in the preload script via channel whitelist.
 *
 * @function setupIpcHandlers
 */
function setupIpcHandlers() {
  /**
   * IPC Handler: Minimize window
   * @returns {Promise<boolean>} Success status
   */
  ipcMain.handle('minimize-window', () => {
    log.debug('IPC: minimize-window');
    mainWindow?.minimize();
    return true;
  });

  /**
   * IPC Handler: Toggle maximize/unmaximize window
   * @returns {Promise<boolean>} New maximized state
   */
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

  /**
   * IPC Handler: Close window
   * @returns {Promise<boolean>} Success status
   */
  ipcMain.handle('close-window', () => {
    log.debug('IPC: close-window');
    mainWindow?.close();
    return true;
  });

  /**
   * IPC Handler: Get system information
   * @returns {Promise<SystemInfo>} System and application information
   */
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

  /**
   * IPC Handler: Get native system theme
   * @returns {Promise<'dark'|'light'>} Current system theme
   */
  ipcMain.handle('get-native-theme', () => {
    return nativeTheme.shouldUseDarkColors ? 'dark' : 'light';
  });

  /**
   * IPC Handler: Set application theme
   * @param {Electron.IpcMainEvent} event - IPC event object
   * @param {'system'|'light'|'dark'} theme - Theme to set
   */
  ipcMain.on('set-native-theme', (event, theme) => {
    nativeTheme.themeSource = theme;
  });

  log.info('IPC handlers registered');
}

// ============================================================================
// APP LIFECYCLE
// ============================================================================

/**
 * Single instance lock to prevent multiple instances of the application.
 * If another instance is already running, this instance will quit immediately
 * and focus the existing instance instead.
 *
 * @type {boolean}
 */
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

