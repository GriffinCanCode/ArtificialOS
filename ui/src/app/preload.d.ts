/**
 * Type definitions for Electron preload script
 * Provides TypeScript support for window.electron and window.electronLog APIs
 */

/**
 * System information returned by the system API
 */
export interface SystemInfo {
  /** Operating system platform (darwin, win32, linux, etc.) */
  platform: string;
  /** CPU architecture (x64, arm64, etc.) */
  arch: string;
  /** Application version */
  version: string;
  /** Electron version */
  electron: string;
  /** Chromium version */
  chrome: string;
  /** Node.js version */
  node: string;
  /** Whether running in development mode */
  isDev: boolean;
}

/**
 * System API for getting system and environment information
 */
export interface SystemAPI {
  /** Get comprehensive system information */
  getInfo(): Promise<SystemInfo>;
  /** Get native system theme preference */
  getTheme(): Promise<"dark" | "light">;
}

/**
 * Version information exposed from process.versions
 */
export interface VersionInfo {
  /** Electron version */
  electron: string;
  /** Chromium version */
  chrome: string;
  /** Node.js version */
  node: string;
}

/**
 * Environment information
 */
export interface EnvInfo {
  /** Whether running in development mode */
  isDev: boolean;
  /** Operating system platform */
  platform: NodeJS.Platform;
}

/**
 * Preload performance metrics (dev mode only)
 */
export interface PreloadMetrics {
  /** Time taken to load preload script in milliseconds */
  loadTime: number;
  /** ISO timestamp when preload finished */
  timestamp: string;
}

/**
 * Main Electron API exposed to renderer process
 * Provides secure, validated access to main process functionality
 */
export interface ElectronAPI {
  /** Minimize the application window */
  minimize(): Promise<boolean>;

  /** Toggle window maximize/unmaximize state */
  maximize(): Promise<boolean>;

  /** Close the application window */
  close(): Promise<boolean>;

  /** System and environment information */
  system: SystemAPI;

  /** Version information for Electron, Chrome, and Node.js */
  versions: VersionInfo;

  /** Environment information */
  env: EnvInfo;

  /** Error message if preload failed (only present on error) */
  error?: string;

  /** Whether the API is available (false if preload failed) */
  available?: boolean;
}

/**
 * Logger API exposed to renderer process
 * Provides consistent logging that goes through electron-log
 */
export interface ElectronLogAPI {
  /** Log error message */
  error(...args: any[]): void;

  /** Log warning message */
  warn(...args: any[]): void;

  /** Log info message */
  info(...args: any[]): void;

  /** Log debug message (dev only) */
  debug(...args: any[]): void;

  /** Log verbose message (dev only) */
  verbose(...args: any[]): void;
}

/**
 * Global window extensions provided by the preload script
 */
declare global {
  interface Window {
    /** Electron API for interacting with the main process */
    electron: ElectronAPI;

    /** Logger API for consistent logging */
    electronLog: ElectronLogAPI;

    /** Performance metrics for preload script (dev mode only) */
    __preloadMetrics?: PreloadMetrics;
  }
}

export {};
