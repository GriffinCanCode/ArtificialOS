/**
 * SDK Type Definitions for Native Apps
 * Provided by the OS at runtime
 */

export interface NativeAppContext {
  appId: string;
  executor: ToolExecutor;
  window: WindowAPI;
}

export interface ToolExecutor {
  execute(toolId: string, params?: Record<string, any>): Promise<any>;
  cleanup(): void;
}

export interface WindowAPI {
  setTitle(title: string): void;
  setSize(width: number, height: number): void;
  close(): void;
  minimize(): void;
  maximize(): void;
  focus(): void;
  updateState(state: Partial<WindowState>): void;
}

export interface WindowState {
  position?: { x: number; y: number };
  size?: { width: number; height: number };
  minimized?: boolean;
  maximized?: boolean;
}

export interface NativeAppProps {
  context: NativeAppContext;
}

