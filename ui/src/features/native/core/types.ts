/**
 * Native App Core Types
 * Type definitions for native TypeScript/React applications
 */

import type { ComponentState } from '../../dynamics/state/state';
import type { ToolExecutor } from '../../dynamics/execution/executor';
import type { NativeAppContext, NativeAppProps } from '../../../core/sdk';

// ============================================================================
// App Type Enum
// ============================================================================

/**
 * Application types supported by the OS
 */
export enum AppType {
  /** Blueprint apps (JSON-based, AI-generated) */
  BLUEPRINT = 'blueprint',
  /** Native web apps (TypeScript/React, custom components) */
  NATIVE = 'native_web',
  /** Native process apps (OS executables) */
  PROCESS = 'native_proc',
}

// ============================================================================
// Loaded App
// ============================================================================

/**
 * Loaded native app module
 */
export interface LoadedApp {
  /** Package ID */
  id: string;
  /** React component */
  component: React.ComponentType<NativeAppProps>;
  /** Optional cleanup function */
  cleanup?: () => void;
  /** Load timestamp */
  loadedAt: number;
}

// ============================================================================
// App Manifest
// ============================================================================

/**
 * Native web app manifest
 */
export interface NativeManifest {
  /** Entry point file */
  entryPoint: string;
  /** Exported component name */
  exports: {
    component: string;
  };
  /** Dev server URL (development only) */
  devServer?: string;
}

/**
 * Native process app manifest
 */
export interface ProcessManifest {
  /** Executable path or command */
  executable: string;
  /** Command arguments */
  args: string[];
  /** Working directory */
  workingDir: string;
  /** UI type */
  uiType: 'terminal' | 'headless' | 'custom';
  /** Environment variables */
  env: Record<string, string>;
}

// ============================================================================
// Window Metadata
// ============================================================================

/**
 * Native app window metadata
 * Extends base window metadata for native apps
 */
export interface NativeWindowMeta {
  /** App type */
  appType: AppType;
  /** Package ID in registry */
  packageId: string;
  /** Bundle path (for native web apps) */
  bundlePath?: string;
  /** Services required */
  services?: string[];
  /** Permissions granted */
  permissions?: string[];
  /** Process ID (for native proc apps) */
  pid?: number;
}

// ============================================================================
// Loader Cache Entry
// ============================================================================

/**
 * Cache entry for loaded apps
 */
export interface CacheEntry {
  /** Loaded app */
  app: LoadedApp;
  /** Reference count */
  refCount: number;
  /** Last accessed timestamp */
  lastAccessed: number;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Native app loading errors
 */
export class NativeAppError extends Error {
  constructor(
    message: string,
    public code: string,
    public appId?: string
  ) {
    super(message);
    this.name = 'NativeAppError';
  }
}

/**
 * Error codes
 */
export enum ErrorCode {
  LOAD_FAILED = 'LOAD_FAILED',
  NO_DEFAULT_EXPORT = 'NO_DEFAULT_EXPORT',
  INVALID_COMPONENT = 'INVALID_COMPONENT',
  BUNDLE_NOT_FOUND = 'BUNDLE_NOT_FOUND',
  TIMEOUT = 'TIMEOUT',
}

// ============================================================================
// Re-exports
// ============================================================================

export type { NativeAppContext, NativeAppProps, ComponentState, ToolExecutor };
