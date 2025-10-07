/**
 * Native App SDK
 * API for native TypeScript/React apps to interact with the OS
 *
 * IMPORTANT: Native apps DO NOT use prebuilt Blueprint components.
 * They are full React applications with complete freedom to use any npm packages.
 */

import { ComponentState } from '../../features/dynamics/state/state';
import { ToolExecutor } from '../../features/dynamics/execution/executor';
import { logger } from '../utils/monitoring/logger';

// ============================================================================
// Type Definitions
// ============================================================================

/**
 * Native App Context
 * Provides access to OS APIs, state management, and window controls
 */
export interface NativeAppContext {
  appId: string;
  state: ComponentState;
  executor: ToolExecutor;
  window: AppWindow;
}

/**
 * Window controls for the app
 */
export interface AppWindow {
  id: string;
  setTitle: (title: string) => void;
  setIcon: (icon: string) => void;
  close: () => void;
  minimize: () => void;
  maximize: () => void;
  focus: () => void;
}

/**
 * Props passed to native app components
 */
export interface NativeAppProps {
  context: NativeAppContext;
}

// ============================================================================
// Native App Base Class (Optional Helper)
// ============================================================================

/**
 * Base class for native apps
 * Provides helper methods and lifecycle hooks
 *
 * Usage (optional):
 * ```typescript
 * class MyApp extends NativeApp {
 *   async onMount() {
 *     const data = await this.storageGet('mydata');
 *     this.setState('data', data);
 *   }
 * }
 * ```
 */
export abstract class NativeApp {
  protected context: NativeAppContext;

  constructor(context: NativeAppContext) {
    this.context = context;
  }

  // ============================================================================
  // State Management
  // ============================================================================

  /**
   * Set state value
   */
  protected setState(key: string, value: any): void {
    this.context.state.set(key, value);
  }

  /**
   * Get state value
   */
  protected getState<T = any>(key: string): T | undefined {
    return this.context.state.get<T>(key);
  }

  /**
   * Subscribe to state changes
   */
  protected subscribeState<T = any>(
    key: string,
    callback: (value: T) => void
  ): () => void {
    return this.context.state.subscribe(key, callback);
  }

  /**
   * Batch state updates
   */
  protected batchState(fn: () => void): void {
    this.context.state.batch(fn);
  }

  // ============================================================================
  // Service Calls
  // ============================================================================

  /**
   * Execute a backend service/tool
   */
  protected async callService(
    toolId: string,
    params: Record<string, any> = {}
  ): Promise<any> {
    try {
      return await this.context.executor.execute(toolId, params);
    } catch (error) {
      logger.error(`Service call failed: ${toolId}`, error as Error, { toolId, params });
      throw error;
    }
  }

  // ============================================================================
  // Filesystem APIs
  // ============================================================================

  /**
   * Read file contents
   */
  protected async readFile(path: string): Promise<string> {
    const result = await this.callService('filesystem.read', { path });
    return result?.content || '';
  }

  /**
   * Write file contents
   */
  protected async writeFile(path: string, content: string): Promise<void> {
    await this.callService('filesystem.write', { path, content });
  }

  /**
   * List directory contents
   */
  protected async listDirectory(path: string): Promise<any[]> {
    const result = await this.callService('filesystem.list', { path });
    return result?.entries || [];
  }

  /**
   * Create directory
   */
  protected async createDirectory(path: string): Promise<void> {
    await this.callService('filesystem.mkdir', { path });
  }

  /**
   * Delete file or directory
   */
  protected async deleteFile(path: string): Promise<void> {
    await this.callService('filesystem.delete', { path });
  }

  /**
   * Check if file exists
   */
  protected async fileExists(path: string): Promise<boolean> {
    const result = await this.callService('filesystem.exists', { path });
    return result?.exists || false;
  }

  // ============================================================================
  // Storage APIs (Persistent Key-Value)
  // ============================================================================

  /**
   * Get value from storage
   */
  protected async storageGet<T = any>(key: string): Promise<T | undefined> {
    const result = await this.callService('storage.get', { key });
    return result?.value;
  }

  /**
   * Set value in storage
   */
  protected async storageSet(key: string, value: any): Promise<void> {
    await this.callService('storage.set', { key, value });
  }

  /**
   * Remove value from storage
   */
  protected async storageRemove(key: string): Promise<void> {
    await this.callService('storage.remove', { key });
  }

  /**
   * List all storage keys
   */
  protected async storageList(): Promise<string[]> {
    const result = await this.callService('storage.list', {});
    return result?.keys || [];
  }

  // ============================================================================
  // System APIs
  // ============================================================================

  /**
   * Get system information
   */
  protected async getSystemInfo(): Promise<any> {
    return await this.callService('system.info', {});
  }

  /**
   * Get current time
   */
  protected async getSystemTime(): Promise<number> {
    const result = await this.callService('system.time', {});
    return result?.timestamp || Date.now();
  }

  /**
   * Log message to system
   */
  protected async log(level: 'debug' | 'info' | 'warn' | 'error', message: string): Promise<void> {
    await this.callService('system.log', { level, message });
  }

  // ============================================================================
  // HTTP APIs
  // ============================================================================

  /**
   * Make HTTP GET request
   */
  protected async httpGet(url: string, headers?: Record<string, string>): Promise<any> {
    return await this.callService('http.get', { url, headers });
  }

  /**
   * Make HTTP POST request
   */
  protected async httpPost(
    url: string,
    body: any,
    headers?: Record<string, string>
  ): Promise<any> {
    return await this.callService('http.post', { url, body, headers });
  }

  // ============================================================================
  // UI APIs
  // ============================================================================

  /**
   * Show toast notification
   */
  protected async showToast(message: string, variant?: 'success' | 'error' | 'info' | 'warning'): Promise<void> {
    await this.callService('toast.show', { message, variant });
  }

  /**
   * Show system notification
   */
  protected async showNotification(title: string, body: string, icon?: string): Promise<void> {
    await this.callService('notification.show', { title, body, icon });
  }

  // ============================================================================
  // Window APIs
  // ============================================================================

  /**
   * Set window title
   */
  protected setWindowTitle(title: string): void {
    this.context.window.setTitle(title);
  }

  /**
   * Set window icon
   */
  protected setWindowIcon(icon: string): void {
    this.context.window.setIcon(icon);
  }

  /**
   * Close window
   */
  protected closeWindow(): void {
    this.context.window.close();
  }

  // ============================================================================
  // Lifecycle Hooks (Override in subclass)
  // ============================================================================

  /**
   * Called when app mounts
   */
  async onMount?(): Promise<void>;

  /**
   * Called when app unmounts
   */
  async onUnmount?(): Promise<void>;

  /**
   * Called when window gains focus
   */
  async onFocus?(): Promise<void>;

  /**
   * Called when window loses focus
   */
  async onBlur?(): Promise<void>;
}

// ============================================================================
// Context Factory
// ============================================================================

/**
 * Create native app context
 * Called by the app loader when initializing a native app
 *
 * @internal - Used by the framework, not by app developers
 */
export function createAppContext(
  appId: string,
  windowId: string,
  windowActions: any
): NativeAppContext {
  const state = new ComponentState();
  const executor = new ToolExecutor(state);
  executor.setAppId(appId);

  const window: AppWindow = {
    id: windowId,
    setTitle: (title: string) => {
      if (windowActions?.update) {
        windowActions.update(windowId, { title });
      }
    },
    setIcon: (icon: string) => {
      if (windowActions?.update) {
        windowActions.update(windowId, { icon });
      }
    },
    close: () => {
      if (windowActions?.close) {
        windowActions.close(windowId);
      }
    },
    minimize: () => {
      if (windowActions?.minimize) {
        windowActions.minimize(windowId);
      }
    },
    maximize: () => {
      if (windowActions?.maximize) {
        windowActions.maximize(windowId);
      }
    },
    focus: () => {
      if (windowActions?.focus) {
        windowActions.focus(windowId);
      }
    },
  };

  return { appId, state, executor, window };
}

// ============================================================================
// Exports
// ============================================================================

export type { ComponentState, ToolExecutor };
