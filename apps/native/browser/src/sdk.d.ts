/**
 * SDK Type Declarations
 * Export types for the app
 */

export interface NativeAppContext {
  appId: string;
  state: ComponentState;
  executor: ToolExecutor;
  window: AppWindow;
}

export interface AppWindow {
  id: string;
  setTitle: (title: string) => void;
  setIcon: (icon: string) => void;
  close: () => void;
  minimize: () => void;
  maximize: () => void;
  focus: () => void;
}

export interface NativeAppProps {
  context: NativeAppContext;
}

export interface ComponentState {
  get<T = any>(key: string, defaultValue?: T): T;
  set(key: string, value: any): void;
  subscribe<T = any>(key: string, callback: (value: T) => void): () => void;
  batch(fn: () => void): void;
}

export interface ToolExecutor {
  execute(toolId: string, params?: Record<string, any>): Promise<any>;
  setAppId(appId: string): void;
  cleanup(): void;
}

